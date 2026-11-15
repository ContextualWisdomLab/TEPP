//! Embedded migration catalog and fail-closed SQL contracts.

#[path = "migration_core.rs"]
mod core;
#[path = "migration_validation.rs"]
mod validation;

use crate::MigrationContractError;
pub use core::MigrationCatalog;

/// Validate migration SQL against TEPP persistence contracts through one
/// PostgreSQL-aware lexical boundary.
///
/// The lexical boundary removes comments and quoted SQL bodies from the
/// structural parser view, exposes quoted identifiers with their declared
/// spelling, and rejects unterminated lexical regions before contract parsing.
/// Both forward and rollback SQL pass through that boundary before structural
/// validation so malformed rollback text cannot bypass the catalog contract.
///
/// # Errors
///
/// Returns lexical, naming, tenant, temporal, RLS, or emptiness failures.
pub fn validate_migration_catalog(
    catalog: &MigrationCatalog,
) -> Result<(), MigrationContractError> {
    let runtime_role_declared =
        validation::declares_created_role(catalog.up_sql(), "tepp_app_runtime")
            .ok_or(MigrationContractError::EmptyMigrationSql)?;
    let normalized_up = normalize_catalog_sql(catalog.up_sql())
        .ok_or(MigrationContractError::EmptyMigrationSql)?;
    let normalized_down = normalize_catalog_sql(catalog.down_sql())
        .ok_or(MigrationContractError::EmptyMigrationSql)?;
    let committed_up = core::project_committed_sql(&normalized_up)
        .ok_or(MigrationContractError::MissingAppRuntimeRole)?;
    let requires_runtime_role = validation::declares_row_level_security(&committed_up);
    if requires_runtime_role && !declares_tenant_session_guc(&committed_up) {
        return Err(MigrationContractError::MissingTenantSessionGuc);
    }
    if requires_runtime_role && !tenant_policies_bind_session_guc(&committed_up) {
        return Err(MigrationContractError::MissingRlsPolicy);
    }
    let normalized = MigrationCatalog::from_sql(&normalized_up, &normalized_down);
    core::validate_migration_catalog(&normalized)?;
    if requires_runtime_role && !runtime_role_declared {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    }
    Ok(())
}

/// Require the tenant setting key to be the first argument of PostgreSQL's
/// unqualified `current_setting` call rather than accepting the same literal
/// anywhere in the migration text. Schema-qualified lookalikes fail closed so
/// application-defined functions cannot impersonate the built-in witness.
fn declares_tenant_session_guc(normalized_sql: &str) -> bool {
    const FUNCTION_NAME: &str = "current_setting";
    const TENANT_GUC: &str = "'tepp.current_tenant_record_id'";

    let mut search_from = 0usize;
    while let Some(relative) = normalized_sql[search_from..].find(FUNCTION_NAME) {
        let start = search_from + relative;
        let end = start + FUNCTION_NAME.len();
        let prefix = normalized_sql[..start].trim_end();
        let starts_at_boundary = normalized_sql[..start]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_');
        let is_unqualified = !prefix.ends_with('.');
        let ends_at_boundary = normalized_sql[end..]
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_');

        if starts_at_boundary && is_unqualified && ends_at_boundary {
            let after_name = normalized_sql[end..].trim_start();
            if let Some(arguments) = after_name.strip_prefix('(') {
                let first_argument = arguments.trim_start();
                if let Some(after_key) = first_argument.strip_prefix(TENANT_GUC) {
                    let delimiter = after_key.trim_start().chars().next();
                    if matches!(delimiter, Some(',' | ')')) {
                        return true;
                    }
                }
            }
        }
        search_from = end;
    }
    false
}

/// Byte span occupied by a top-level policy clause keyword.
///
/// Keeping start and end separately lets callers slice only the predicate body
/// while excluding headers such as policy names, target tables, commands, and
/// role lists from tenant-isolation evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PolicyClauseSpan {
    start: usize,
    end: usize,
}

/// Row-predicate clauses whose PostgreSQL command semantics differ.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PolicyClause {
    Using,
    WithCheck,
}

/// Return whether one ASCII byte can continue the bounded SQL identifiers used here.
///
/// This helper is intentionally narrower than the PostgreSQL lexer because it
/// is used only for ASCII keyword and role-list boundary checks after lexical
/// normalization; durable object-name parsing uses the core identifier authority.
fn is_sql_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Match an ASCII SQL keyword only when both sides are identifier boundaries.
///
/// The returned index is immediately after the keyword. Prefixes embedded in a
/// longer identifier are rejected so policy-clause and Boolean parsing cannot
/// manufacture structure from names such as `using_flag` or `orphan`.
fn bounded_ascii_keyword(bytes: &[u8], start: usize, keyword: &[u8]) -> Option<usize> {
    let end = start.checked_add(keyword.len())?;
    if end > bytes.len() || !bytes[start..end].eq_ignore_ascii_case(keyword) {
        return None;
    }
    if start > 0 && is_sql_identifier_byte(bytes[start - 1]) {
        return None;
    }
    if end < bytes.len() && is_sql_identifier_byte(bytes[end]) {
        return None;
    }
    Some(end)
}

/// Locate a policy clause at parenthesis depth zero after lexical normalization.
/// PostgreSQL keywords need token boundaries, not surrounding whitespace, so
/// `)WITH CHECK(` and `USING(` are valid clause boundaries. Comments have
/// already been converted to spacing by the lexical authority.
fn policy_clause_span(policy_sql: &str, clause: PolicyClause) -> Option<PolicyClauseSpan> {
    let bytes = policy_sql.as_bytes();
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'(' => {
                depth = depth.saturating_add(1);
                index += 1;
                continue;
            }
            b')' => {
                depth = depth.saturating_sub(1);
                index += 1;
                continue;
            }
            _ => {}
        }
        if depth != 0 {
            index += 1;
            continue;
        }

        match clause {
            PolicyClause::Using => {
                if let Some(end) = bounded_ascii_keyword(bytes, index, b"using") {
                    return Some(PolicyClauseSpan { start: index, end });
                }
            }
            PolicyClause::WithCheck => {
                if let Some(with_end) = bounded_ascii_keyword(bytes, index, b"with") {
                    let mut check_start = with_end;
                    let whitespace_start = check_start;
                    while check_start < bytes.len() && bytes[check_start].is_ascii_whitespace() {
                        check_start += 1;
                    }
                    if check_start > whitespace_start {
                        if let Some(check_end) =
                            bounded_ascii_keyword(bytes, check_start, b"check")
                        {
                            return Some(PolicyClauseSpan {
                                start: index,
                                end: check_end,
                            });
                        }
                    }
                }
            }
        }
        index += 1;
    }
    None
}

/// Return whether a row predicate in the normalized policy statement contains
/// the exact unquoted tenant key identifier. Header names, target tables, and
/// role lists are excluded so they cannot impersonate predicate evidence.
fn policy_binds_tenant_identifier(policy_sql: &str) -> bool {
    const TENANT_IDENTIFIER: &str = "tenant_record_id";

    let using_clause = policy_clause_span(policy_sql, PolicyClause::Using);
    let check_clause = policy_clause_span(policy_sql, PolicyClause::WithCheck);
    let Some(predicate_start) = [using_clause, check_clause]
        .into_iter()
        .flatten()
        .map(|span| span.end)
        .min()
    else {
        return false;
    };
    let predicate_sql = &policy_sql[predicate_start..];

    let mut search_from = 0usize;
    while let Some(relative) = predicate_sql[search_from..].find(TENANT_IDENTIFIER) {
        let start = search_from + relative;
        let end = start + TENANT_IDENTIFIER.len();
        let inside_atomic_literal = predicate_sql[..start]
            .bytes()
            .filter(|byte| *byte == b'\'')
            .count()
            % 2
            == 1;
        let starts_at_boundary = predicate_sql[..start]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_');
        let ends_at_boundary = predicate_sql[end..]
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_');
        if !inside_atomic_literal && starts_at_boundary && ends_at_boundary {
            return true;
        }
        search_from = end;
    }
    false
}

/// Return whether one equality operand is exactly TEPP's tenant row key.
///
/// Only the direct identifier and its shipped `::text` cast are admitted; more
/// complex expressions fail closed so computed values cannot masquerade as the
/// authoritative row tenant.
fn direct_tenant_operand(side: &str) -> bool {
    let compact = strip_enclosing_predicate_parentheses(side)
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    matches!(compact.as_str(), "tenant_record_id" | "tenant_record_id::text")
}

/// Accept only the bounded session-side expressions used by TEPP's tenant RLS
/// contract. Merely containing `current_setting(...)` is insufficient: wrappers
/// such as `coalesce(current_setting(...), tenant_record_id::text)` can fall
/// back to the row's own tenant value and turn the equality into a tautology.
/// Empty string literals are removed by lexical normalization, so the shipped
/// `nullif(current_setting(..., true), '')` form appears with an empty second
/// argument here.
fn direct_tenant_session_operand(side: &str) -> bool {
    let compact = strip_enclosing_predicate_parentheses(side)
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    matches!(
        compact.as_str(),
        "current_setting('tepp.current_tenant_record_id')"
            | "current_setting('tepp.current_tenant_record_id',true)"
            | "nullif(current_setting('tepp.current_tenant_record_id'),)"
            | "nullif(current_setting('tepp.current_tenant_record_id',true),)"
    )
}

/// Return whether a depth-zero equality directly binds row tenant to session tenant.
///
/// Comparison operators such as `<=`, `>=`, `!=`, and `==` are excluded. Both
/// operand orders are supported, but each side must satisfy the bounded direct
/// operand contracts rather than merely containing the relevant identifiers.
fn predicate_contains_top_level_tenant_session_equality(predicate_sql: &str) -> bool {
    let bytes = predicate_sql.as_bytes();
    let mut depth = 0usize;

    for equality in 0..bytes.len() {
        match bytes[equality] {
            b'(' => {
                depth = depth.saturating_add(1);
                continue;
            }
            b')' => {
                depth = depth.saturating_sub(1);
                continue;
            }
            b'=' if depth == 0 => {}
            _ => continue,
        }

        let previous = equality.checked_sub(1).and_then(|index| bytes.get(index));
        let next = bytes.get(equality + 1);
        if previous.is_some_and(|byte| matches!(*byte, b'<' | b'>' | b'!' | b'='))
            || next.is_some_and(|byte| matches!(*byte, b'<' | b'>' | b'='))
        {
            continue;
        }

        let left = &predicate_sql[..equality];
        let right = &predicate_sql[equality + 1..];
        if (direct_tenant_operand(left) && direct_tenant_session_operand(right))
            || (direct_tenant_session_operand(left) && direct_tenant_operand(right))
        {
            return true;
        }
    }
    false
}

/// Remove only parentheses that enclose the entire predicate expression.
///
/// Parentheses that close before trailing content are structural and therefore
/// retained. Malformed nesting also stops stripping so later checks fail closed
/// rather than accepting a widened or synthetically simplified expression.
fn strip_enclosing_predicate_parentheses(mut predicate_sql: &str) -> &str {
    loop {
        let trimmed = predicate_sql.trim();
        let bytes = trimmed.as_bytes();
        if bytes.first() != Some(&b'(') || bytes.last() != Some(&b')') {
            return trimmed;
        }

        let mut depth = 0usize;
        let mut encloses_entire_expression = true;
        for (index, byte) in bytes.iter().enumerate() {
            match *byte {
                b'(' => depth = depth.saturating_add(1),
                b')' => {
                    if depth == 0 {
                        return trimmed;
                    }
                    depth -= 1;
                    if depth == 0 && index + 1 != bytes.len() {
                        encloses_entire_expression = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth != 0 || !encloses_entire_expression {
            return trimmed;
        }
        predicate_sql = &trimmed[1..trimmed.len() - 1];
    }
}

/// Split a predicate on one bounded Boolean keyword only at depth zero.
///
/// Returning `None` means the keyword is absent at top level, not that the
/// predicate is malformed. Nested alternatives remain inside their owning
/// segment for recursive evaluation by the Boolean-path contract.
fn split_top_level_boolean<'a>(predicate_sql: &'a str, keyword: &[u8]) -> Option<Vec<&'a str>> {
    let bytes = predicate_sql.as_bytes();
    let mut depth = 0usize;
    let mut segment_start = 0usize;
    let mut index = 0usize;
    let mut segments = Vec::new();

    while index < bytes.len() {
        match bytes[index] {
            b'(' => {
                depth = depth.saturating_add(1);
                index += 1;
                continue;
            }
            b')' => {
                depth = depth.saturating_sub(1);
                index += 1;
                continue;
            }
            _ => {}
        }
        if depth == 0 {
            if let Some(keyword_end) = bounded_ascii_keyword(bytes, index, keyword) {
                segments.push(&predicate_sql[segment_start..index]);
                segment_start = keyword_end;
                index = keyword_end;
                continue;
            }
        }
        index += 1;
    }

    if segments.is_empty() {
        None
    } else {
        segments.push(&predicate_sql[segment_start..]);
        Some(segments)
    }
}

/// Evaluate the bounded Boolean structure of a normalized policy predicate.
/// `OR` requires every disjunct to carry the tenant/session equality, while one
/// tenant-bound `AND` conjunct guards the complete conjunction. Parentheses are
/// grouping only; arbitrary function-call wrappers remain opaque and therefore
/// cannot donate an equality hidden inside their argument list.
fn all_top_level_or_paths_bind_tenant_session(predicate_sql: &str) -> bool {
    let predicate_sql = strip_enclosing_predicate_parentheses(predicate_sql);

    if let Some(disjuncts) = split_top_level_boolean(predicate_sql, b"or") {
        return disjuncts
            .into_iter()
            .all(all_top_level_or_paths_bind_tenant_session);
    }
    if let Some(conjuncts) = split_top_level_boolean(predicate_sql, b"and") {
        return conjuncts
            .into_iter()
            .any(all_top_level_or_paths_bind_tenant_session);
    }

    predicate_contains_top_level_tenant_session_equality(predicate_sql)
}

/// Require the tenant column and tenant session key to participate in the same
/// equality comparison on every row-admitting Boolean path. A tenant-bound
/// conjunct may safely guard nested alternatives such as
/// `tenant_binding AND (role_a OR role_b)`, while `tenant_binding OR true` and
/// opaque wrappers around an unbound alternative fail closed.
fn policy_binds_tenant_session_equality(policy_sql: &str) -> bool {
    all_top_level_or_paths_bind_tenant_session(policy_sql)
}

/// PostgreSQL row-level-security commands represented by the bounded validator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PolicyCommand {
    All,
    Select,
    Insert,
    Update,
    Delete,
}

/// Parse the command scope of one normalized CREATE POLICY statement.
///
/// Omitted `FOR` defaults to PostgreSQL `ALL`; unsupported or malformed command
/// tokens return `None` so they cannot inherit permissive coverage accidentally.
fn policy_command(policy_sql: &str) -> Option<PolicyCommand> {
    let using_clause = policy_clause_span(policy_sql, PolicyClause::Using);
    let check_clause = policy_clause_span(policy_sql, PolicyClause::WithCheck);
    let header_end = [using_clause, check_clause]
        .into_iter()
        .flatten()
        .map(|span| span.start)
        .min()
        .unwrap_or(policy_sql.len());
    let tokens = policy_sql[..header_end].split_whitespace().collect::<Vec<_>>();
    let Some(for_index) = tokens
        .iter()
        .position(|token| token.eq_ignore_ascii_case("FOR"))
    else {
        return Some(PolicyCommand::All);
    };

    match tokens.get(for_index + 1).map(|token| token.to_ascii_uppercase()) {
        Some(command) if command == "ALL" => Some(PolicyCommand::All),
        Some(command) if command == "SELECT" => Some(PolicyCommand::Select),
        Some(command) if command == "INSERT" => Some(PolicyCommand::Insert),
        Some(command) if command == "UPDATE" => Some(PolicyCommand::Update),
        Some(command) if command == "DELETE" => Some(PolicyCommand::Delete),
        _ => None,
    }
}

/// Return the exact table token targeted by one normalized policy header.
///
/// Only the header before `USING`/`WITH CHECK` is searched, preventing `ON`
/// inside row expressions from donating a false policy target.
fn policy_target_table(policy_sql: &str) -> Option<&str> {
    let using_clause = policy_clause_span(policy_sql, PolicyClause::Using);
    let check_clause = policy_clause_span(policy_sql, PolicyClause::WithCheck);
    let header_end = [using_clause, check_clause]
        .into_iter()
        .flatten()
        .map(|span| span.start)
        .min()
        .unwrap_or(policy_sql.len());
    let tokens = policy_sql[..header_end].split_whitespace().collect::<Vec<_>>();
    let on_index = tokens
        .iter()
        .enumerate()
        .skip(2)
        .find_map(|(index, token)| token.eq_ignore_ascii_case("ON").then_some(index))?;
    tokens.get(on_index + 1).copied()
}

/// Parse the policy's explicit `TO` role list, defaulting omission to `PUBLIC`.
///
/// The grammar is intentionally `role (, role)*`; leading, trailing, adjacent,
/// or missing commas and unsupported role tokens return `None` rather than being
/// compacted into a different authorization scope.
fn policy_roles(policy_sql: &str) -> Option<Vec<String>> {
    let using_clause = policy_clause_span(policy_sql, PolicyClause::Using);
    let check_clause = policy_clause_span(policy_sql, PolicyClause::WithCheck);
    let header_end = [using_clause, check_clause]
        .into_iter()
        .flatten()
        .map(|span| span.start)
        .min()
        .unwrap_or(policy_sql.len());
    let tokenizable = policy_sql[..header_end].replace(',', " , ");
    let tokens = tokenizable.split_whitespace().collect::<Vec<_>>();
    let Some(to_index) = tokens
        .iter()
        .position(|token| token.eq_ignore_ascii_case("TO"))
    else {
        return Some(vec!["public".to_owned()]);
    };

    let mut roles = Vec::new();
    let mut expect_role = true;
    for token in tokens.iter().skip(to_index + 1) {
        if expect_role {
            if *token == "," || !token.bytes().all(is_sql_identifier_byte) {
                return None;
            }
            roles.push(token.to_ascii_lowercase());
            expect_role = false;
        } else {
            if *token != "," {
                return None;
            }
            expect_role = true;
        }
    }
    (!roles.is_empty() && !expect_role).then_some(roles)
}

/// Return whether one policy command covers a requested concrete command.
///
/// PostgreSQL `FOR ALL` is the only wildcard in this bounded model; otherwise
/// coverage requires exact command equality.
fn policy_command_applies(policy_command: PolicyCommand, requested_command: PolicyCommand) -> bool {
    policy_command == PolicyCommand::All || policy_command == requested_command
}

/// Validate the row-predicate clauses required by one policy command.
///
/// SELECT/DELETE require `USING`, INSERT requires only `WITH CHECK`, and
/// ALL/UPDATE require `USING` plus a tenant-bound `WITH CHECK` when that clause
/// is explicitly present. Reversed clause order or command-incompatible clauses
/// fail closed.
fn policy_row_predicates_bind_tenant_session(policy_sql: &str) -> bool {
    let Some(command) = policy_command(policy_sql) else {
        return false;
    };
    let using_clause = policy_clause_span(policy_sql, PolicyClause::Using);
    let check_clause = policy_clause_span(policy_sql, PolicyClause::WithCheck);
    if using_clause.is_some_and(|using_span| {
        check_clause.is_some_and(|check_span| check_span.start < using_span.end)
    }) {
        return false;
    }

    let using_sql = using_clause.map(|using_span| {
        let end = check_clause
            .filter(|check_span| check_span.start >= using_span.end)
            .map(|check_span| check_span.start)
            .unwrap_or(policy_sql.len());
        &policy_sql[using_span.end..end]
    });
    let check_sql = check_clause.map(|check_span| &policy_sql[check_span.end..]);
    let using_binds = using_sql.is_some_and(policy_binds_tenant_session_equality);
    let check_binds = check_sql.is_some_and(policy_binds_tenant_session_equality);

    match command {
        PolicyCommand::All | PolicyCommand::Update => {
            using_binds && (check_sql.is_none() || check_binds)
        }
        PolicyCommand::Select | PolicyCommand::Delete => using_binds && check_sql.is_none(),
        PolicyCommand::Insert => using_sql.is_none() && check_binds,
    }
}

/// Return whether the normalized policy header explicitly declares
/// PostgreSQL's restrictive policy composition mode. Only the grammar slot
/// immediately after `ON table_name` counts; `AS restrictive` inside a policy
/// expression is an SQL alias and must not change composition semantics.
fn policy_is_restrictive(policy_sql: &str) -> bool {
    let tokens = policy_sql.split_whitespace().collect::<Vec<_>>();
    let Some(on_index) = tokens
        .iter()
        .enumerate()
        .skip(2)
        .find_map(|(index, token)| token.eq_ignore_ascii_case("ON").then_some(index))
    else {
        return false;
    };

    tokens
        .get(on_index + 2)
        .is_some_and(|token| token.eq_ignore_ascii_case("AS"))
        && tokens
            .get(on_index + 3)
            .is_some_and(|token| token.eq_ignore_ascii_case("RESTRICTIVE"))
}

/// Bind tenant identity and tenant-session evidence to every policy that can
/// independently admit rows. PostgreSQL permissive policies are OR-composed;
/// restrictive policies are AND-composed only after a permissive policy grants
/// access. A restrictive policy therefore requires permissive coverage for each
/// command and target role it can constrain, otherwise PostgreSQL's default-deny
/// composition makes that policy path operationally inaccessible. `PUBLIC`
/// coverage is universal; otherwise role coverage is matched conservatively by
/// exact declared role because role-membership grants are outside this bounded
/// migration parser.
///
/// Command semantics determine which tenant-bound predicates are mandatory:
/// read-capable permissive policies require `USING`, insert requires
/// `WITH CHECK`, and `ALL`/`UPDATE` reuse a valid `USING` for writes only when
/// `WITH CHECK` is omitted. Restrictive policies may add narrower conditions
/// without duplicating the tenant predicate once matching permissive coverage
/// exists.
fn tenant_policies_bind_session_guc(normalized_sql: &str) -> bool {
    const CREATE_POLICY: &str = "create policy";
    const CONCRETE_COMMANDS: [PolicyCommand; 4] = [
        PolicyCommand::Select,
        PolicyCommand::Insert,
        PolicyCommand::Update,
        PolicyCommand::Delete,
    ];

    let lower = normalized_sql.to_ascii_lowercase();
    let mut search_from = 0usize;
    let mut saw_policy = false;
    let mut permissive_coverage = Vec::new();
    let mut restrictive_requirements = Vec::new();

    while let Some(relative) = lower[search_from..].find(CREATE_POLICY) {
        saw_policy = true;
        let start = search_from + relative;
        let statement_tail = &normalized_sql[start..];
        let statement_end = statement_tail.find(';').unwrap_or(statement_tail.len());
        let statement = &statement_tail[..statement_end];
        let Some(table) = policy_target_table(statement) else {
            return false;
        };
        let Some(command) = policy_command(statement) else {
            return false;
        };
        let Some(roles) = policy_roles(statement) else {
            return false;
        };

        if policy_is_restrictive(statement) {
            restrictive_requirements.push((table.to_ascii_lowercase(), command, roles));
        } else {
            if !declares_tenant_session_guc(statement)
                || !policy_binds_tenant_identifier(statement)
                || !policy_row_predicates_bind_tenant_session(statement)
            {
                return false;
            }
            permissive_coverage.push((table.to_ascii_lowercase(), command, roles));
        }
        search_from = start + CREATE_POLICY.len();
    }

    if !saw_policy {
        return false;
    }

    restrictive_requirements.into_iter().all(
        |(table, restrictive_command, restrictive_roles)| {
            CONCRETE_COMMANDS
                .into_iter()
                .filter(|command| policy_command_applies(restrictive_command, *command))
                .all(|command| {
                    if restrictive_roles.iter().any(|role| role == "public") {
                        return permissive_coverage.iter().any(
                            |(permissive_table, permissive_command, permissive_roles)| {
                                permissive_table == &table
                                    && policy_command_applies(*permissive_command, command)
                                    && permissive_roles.iter().any(|role| role == "public")
                            },
                        );
                    }

                    restrictive_roles.iter().all(|restrictive_role| {
                        permissive_coverage.iter().any(
                            |(permissive_table, permissive_command, permissive_roles)| {
                                permissive_table == &table
                                    && policy_command_applies(*permissive_command, command)
                                    && permissive_roles.iter().any(|permissive_role| {
                                        permissive_role == "public"
                                            || permissive_role == restrictive_role
                                    })
                            },
                        )
                    })
                })
        },
    )
}

/// Normalize SQL and then remove PostgreSQL's `CONCURRENTLY` index modifier
/// from the structural parser view without treating it as the index name.
///
/// The first pass owns lexical masking. A second pass is used only when the
/// modifier was removed so existing qualified-name and object-name guards see
/// the canonical `CREATE [UNIQUE] INDEX [IF NOT EXISTS] name` shape.
fn normalize_catalog_sql(sql: &str) -> Option<String> {
    let normalized = validation::normalize_migration_sql(sql)?;
    let canonical = canonicalize_concurrent_index_modifier(&normalized);
    if canonical == normalized {
        Some(normalized)
    } else {
        validation::normalize_migration_sql(&canonical)
    }
}

/// Remove PostgreSQL's `CONCURRENTLY` modifier from CREATE INDEX structural syntax.
///
/// The lexical pass has already masked quoted/commented semicolons, so exposing
/// real statement delimiters as tokens is safe. Only the modifier is removed;
/// `UNIQUE`, `IF NOT EXISTS`, and the declared index name retain their order for
/// downstream naming and qualified-name checks.
fn canonicalize_concurrent_index_modifier(sql: &str) -> String {
    // PostgreSQL does not require whitespace after a statement delimiter. The
    // lexical pass has already masked quoted/commented semicolons, so exposing
    // real delimiters as tokens here keeps `;CREATE INDEX CONCURRENTLY` on the
    // same structural path as its whitespace-separated form.
    let tokenizable = sql.replace(';', " ; ");
    let tokens = tokenizable.split_whitespace().collect::<Vec<_>>();
    let mut canonical = Vec::with_capacity(tokens.len());
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].eq_ignore_ascii_case("CREATE")
            && tokens
                .get(index + 1)
                .is_some_and(|token| token.eq_ignore_ascii_case("INDEX"))
            && tokens
                .get(index + 2)
                .is_some_and(|token| token.eq_ignore_ascii_case("CONCURRENTLY"))
        {
            canonical.push(tokens[index]);
            canonical.push(tokens[index + 1]);
            index += 3;
        } else if tokens[index].eq_ignore_ascii_case("CREATE")
            && tokens
                .get(index + 1)
                .is_some_and(|token| token.eq_ignore_ascii_case("UNIQUE"))
            && tokens
                .get(index + 2)
                .is_some_and(|token| token.eq_ignore_ascii_case("INDEX"))
            && tokens
                .get(index + 3)
                .is_some_and(|token| token.eq_ignore_ascii_case("CONCURRENTLY"))
        {
            canonical.push(tokens[index]);
            canonical.push(tokens[index + 1]);
            canonical.push(tokens[index + 2]);
            index += 4;
        } else {
            canonical.push(tokens[index]);
            index += 1;
        }
    }

    canonical.join(" ")
}

#[cfg(test)]
mod tests {
    use super::{
        PolicyClause, PolicyCommand, all_top_level_or_paths_bind_tenant_session,
        canonicalize_concurrent_index_modifier, declares_tenant_session_guc,
        direct_tenant_session_operand, policy_binds_tenant_identifier,
        policy_binds_tenant_session_equality, policy_clause_span, policy_command,
        policy_is_restrictive, policy_roles, policy_row_predicates_bind_tenant_session,
        tenant_policies_bind_session_guc,
    };

    #[test]
    fn tenant_guc_requires_a_current_setting_call() {
        assert!(declares_tenant_session_guc(
            "tenant_record_id = current_setting ( 'tepp.current_tenant_record_id' , true )"
        ));
        assert!(!declares_tenant_session_guc(
            "select 'tepp.current_tenant_record_id'"
        ));
        assert!(!declares_tenant_session_guc(
            "other_current_setting ( 'tepp.current_tenant_record_id' , true )"
        ));
        assert!(!declares_tenant_session_guc(
            "current_setting ( 'tepp.current_tenant_record_id_shadow' , true )"
        ));
        assert!(!declares_tenant_session_guc(
            "tenant_schema.current_setting ( 'tepp.current_tenant_record_id' , true )"
        ));
        assert!(!declares_tenant_session_guc(
            "tenant_schema . current_setting ( 'tepp.current_tenant_record_id' , true )"
        ));
    }

    #[test]
    fn policy_clauses_use_structural_token_boundaries() {
        let sql = "create policy tenant_policy on tenant_record for all using(tenant_record_id is not null)with\ncheck(tenant_record_id is not null)";
        let using_span = policy_clause_span(sql, PolicyClause::Using).expect("USING clause");
        let check_span =
            policy_clause_span(sql, PolicyClause::WithCheck).expect("WITH CHECK clause");
        assert_eq!(&sql[using_span.start..using_span.end], "using");
        assert_eq!(&sql[check_span.start..check_span.end], "with\ncheck");
        assert!(using_span.end < check_span.start);
        assert!(policy_clause_span("select confusing(1)", PolicyClause::Using).is_none());
    }

    #[test]
    fn tenant_identifier_must_be_structural_policy_evidence() {
        assert!(policy_binds_tenant_identifier(
            "create policy document_record_tenant_isolation on document_record using(tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true ))"
        ));
        assert!(!policy_binds_tenant_identifier(
            "create policy document_record_tenant_isolation on document_record using(document_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true ))"
        ));
        assert!(!policy_binds_tenant_identifier(
            "create policy tenant_record_id on document_record using(document_record_id is not null)"
        ));
        assert!(!policy_binds_tenant_identifier(
            "create policy document_record_tenant_isolation on document_record using(tenant_record_id_shadow is not null)"
        ));
    }

    #[test]
    fn tenant_session_operand_is_bounded_to_the_supported_contract_shape() {
        assert!(direct_tenant_session_operand(
            "current_setting ( 'tepp.current_tenant_record_id' , true )"
        ));
        assert!(direct_tenant_session_operand(
            "nullif ( current_setting ( 'tepp.current_tenant_record_id' , true ) , )"
        ));
        assert!(!direct_tenant_session_operand(
            "coalesce ( current_setting ( 'tepp.current_tenant_record_id' , true ) , tenant_record_id::text )"
        ));
        assert!(!direct_tenant_session_operand(
            "other_current_setting ( 'tepp.current_tenant_record_id' , true )"
        ));
    }

    #[test]
    fn tenant_session_witness_must_be_relationally_bound() {
        assert!(policy_binds_tenant_session_equality(
            "tenant_record_id::text = nullif ( current_setting ( 'tepp.current_tenant_record_id' , true ) , )"
        ));
        assert!(policy_binds_tenant_session_equality(
            "current_setting ( 'tepp.current_tenant_record_id' , true ) = tenant_record_id::text"
        ));
        assert!(!policy_binds_tenant_session_equality(
            "tenant_record_id::text = coalesce ( current_setting ( 'tepp.current_tenant_record_id' , true ) , tenant_record_id::text )"
        ));
        assert!(!policy_binds_tenant_session_equality(
            "tenant_record_id is not null and current_setting ( 'tepp.current_tenant_record_id' , true ) is not null"
        ));
        assert!(!policy_binds_tenant_session_equality(
            "tenant_record_id::text = document_record_id::text and current_setting ( 'tepp.current_tenant_record_id' , true ) is not null"
        ));
    }

    #[test]
    fn every_top_level_or_path_requires_tenant_equality() {
        let binding = "tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )";
        assert!(!all_top_level_or_paths_bind_tenant_session(&format!(
            "({binding} or true)"
        )));
        assert!(all_top_level_or_paths_bind_tenant_session(&format!(
            "(({binding} and document_record_id is not null) or ({binding} and document_record_id is null))"
        )));
        assert!(all_top_level_or_paths_bind_tenant_session(&format!(
            "({binding} and (document_record_id is null or document_record_id is not null))"
        )));
        assert!(!all_top_level_or_paths_bind_tenant_session(&format!(
            "coalesce({binding} or true, false)"
        )));
    }

    #[test]
    fn policy_command_defaults_to_all_and_rejects_unknown_commands() {
        assert_eq!(
            policy_command("create policy tenant_policy on tenant_record using(true)"),
            Some(PolicyCommand::All)
        );
        assert_eq!(
            policy_command("create policy tenant_policy on tenant_record for all using(true)"),
            Some(PolicyCommand::All)
        );
        assert_eq!(
            policy_command("create policy tenant_policy on tenant_record for select using(true)"),
            Some(PolicyCommand::Select)
        );
        assert_eq!(
            policy_command("create policy tenant_policy on tenant_record for insert with check(true)"),
            Some(PolicyCommand::Insert)
        );
        assert_eq!(
            policy_command("create policy tenant_policy on tenant_record for update using(true)"),
            Some(PolicyCommand::Update)
        );
        assert_eq!(
            policy_command("create policy tenant_policy on tenant_record for delete using(true)"),
            Some(PolicyCommand::Delete)
        );
        assert_eq!(
            policy_command("create policy tenant_policy on tenant_record for merge using(true)"),
            None
        );
    }

    #[test]
    fn policy_roles_default_to_public_and_preserve_explicit_targets() {
        assert_eq!(
            policy_roles("create policy tenant_policy on tenant_record using(true)"),
            Some(vec!["public".to_owned()])
        );
        assert_eq!(
            policy_roles(
                "create policy tenant_policy on tenant_record for select to Reader_Role, writer_role using(true)"
            ),
            Some(vec!["reader_role".to_owned(), "writer_role".to_owned()])
        );
        assert_eq!(
            policy_roles("create policy tenant_policy on tenant_record for select to using(true)"),
            None
        );
        assert_eq!(
            policy_roles(
                "create policy tenant_policy on tenant_record for select to reader-role using(true)"
            ),
            None
        );
        assert_eq!(
            policy_roles("create policy tenant_policy on tenant_record for select to ,reader_role using(true)"),
            None
        );
        assert_eq!(
            policy_roles("create policy tenant_policy on tenant_record for select to reader_role, using(true)"),
            None
        );
        assert_eq!(
            policy_roles(
                "create policy tenant_policy on tenant_record for select to reader_role,,writer_role using(true)"
            ),
            None
        );
        assert_eq!(
            policy_roles(
                "create policy tenant_policy on tenant_record for select to reader_role writer_role using(true)"
            ),
            None
        );
    }

    #[test]
    fn explicit_policy_command_requires_the_correct_tenant_predicate() {
        let binding = "tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )";
        assert!(policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for all using({binding})"
        )));
        assert!(!policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for all with check({binding})"
        )));
        assert!(policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for select using({binding})"
        )));
        assert!(policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for delete using({binding})"
        )));
        assert!(policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for insert with check({binding})"
        )));
        assert!(!policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for insert using({binding}) with check({binding})"
        )));
        assert!(policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for update using({binding})"
        )));
        assert!(!policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for update with check({binding})"
        )));
        assert!(!policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for all using({binding})with check(tenant_record_id is not null)"
        )));
        assert!(policy_row_predicates_bind_tenant_session(&format!(
            "create policy tenant_policy on tenant_record for all using({binding})with check({binding})"
        )));
    }

    #[test]
    fn tenant_guc_is_required_for_permissive_but_not_restrictive_policies() {
        assert!(tenant_policies_bind_session_guc(
            "create policy document_record_tenant_isolation on document_record using(tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true ));"
        ));
        assert!(!tenant_policies_bind_session_guc(
            "select current_setting ( 'tepp.current_tenant_record_id' , true ); create policy document_record_tenant_isolation on document_record using(tenant_record_id is not null);"
        ));
        assert!(!tenant_policies_bind_session_guc(
            "create policy document_record_tenant_isolation on document_record using(document_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )); select tenant_record_id from document_record;"
        ));
        assert!(!tenant_policies_bind_session_guc(
            "create policy document_record_tenant_isolation on document_record using(tenant_record_id is not null and current_setting ( 'tepp.current_tenant_record_id' , true ) is not null);"
        ));
        assert!(!tenant_policies_bind_session_guc(
            "create policy document_record_tenant_isolation on document_record for all using(tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )) with check(tenant_record_id is not null);"
        ));
        assert!(tenant_policies_bind_session_guc(
            "create policy document_record_tenant_isolation on document_record using(tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )); create policy document_record_visibility_guard on document_record as restrictive for select using(document_record_id is not null);"
        ));
        assert!(!tenant_policies_bind_session_guc(
            "create policy document_record_visibility_guard on document_record as restrictive for select using(document_record_id is not null);"
        ));
        assert!(!tenant_policies_bind_session_guc(
            "create policy document_record_insert_isolation on document_record as permissive for insert with check(tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )); create policy document_record_visibility_guard on document_record as restrictive for select using(document_record_id is not null);"
        ));
        assert!(!tenant_policies_bind_session_guc(
            "create policy document_record_writer_isolation on document_record as permissive for select to writer_role using(tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )); create policy document_record_reader_guard on document_record as restrictive for select to reader_role using(document_record_id is not null);"
        ));
        assert!(tenant_policies_bind_session_guc(
            "create policy document_record_reader_isolation on document_record as permissive for select to reader_role using(tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )); create policy document_record_reader_guard on document_record as restrictive for select to reader_role using(document_record_id is not null);"
        ));
        assert!(tenant_policies_bind_session_guc(
            "create policy document_record_public_isolation on document_record as permissive for select using(tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )); create policy document_record_reader_guard on document_record as restrictive for select to reader_role using(document_record_id is not null);"
        ));
        assert!(!tenant_policies_bind_session_guc(
            "create policy document_record_reader_isolation on document_record as permissive for select to reader_role using(tenant_record_id::text = current_setting ( 'tepp.current_tenant_record_id' , true )); create policy document_record_public_guard on document_record as restrictive for select using(document_record_id is not null);"
        ));
        assert!(policy_is_restrictive(
            "create policy document_record_visibility_guard on document_record AS RESTRICTIVE for select using(true)"
        ));
        assert!(!policy_is_restrictive(
            "create policy document_record_visibility_guard on document_record AS PERMISSIVE for select using(true)"
        ));
        assert!(!policy_is_restrictive(
            "create policy document_record_visibility_guard on document_record for select using(exists (select 1 AS restrictive))"
        ));
    }

    #[test]
    fn concurrent_index_modifier_is_removed_without_changing_the_declared_name() {
        assert_eq!(
            canonicalize_concurrent_index_modifier(
                "CREATE INDEX CONCURRENTLY tenant_record_lookup_index ON tenant_record"
            ),
            "CREATE INDEX tenant_record_lookup_index ON tenant_record"
        );
        assert_eq!(
            canonicalize_concurrent_index_modifier(
                "CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS tenant_record_lookup_index ON tenant_record"
            ),
            "CREATE UNIQUE INDEX IF NOT EXISTS tenant_record_lookup_index ON tenant_record"
        );
        assert_eq!(
            canonicalize_concurrent_index_modifier(
                "CREATE TABLE tenant_record (tenant_record_id uuid);CREATE INDEX CONCURRENTLY tenant_record_lookup_index ON tenant_record"
            ),
            "CREATE TABLE tenant_record (tenant_record_id uuid) ; CREATE INDEX tenant_record_lookup_index ON tenant_record"
        );
    }
}
