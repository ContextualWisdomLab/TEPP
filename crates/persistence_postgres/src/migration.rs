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
    let requires_runtime_role = validation::declares_row_level_security(&normalized_up);
    if requires_runtime_role && !declares_tenant_session_guc(&normalized_up) {
        return Err(MigrationContractError::MissingTenantSessionGuc);
    }
    if requires_runtime_role && !tenant_policies_bind_session_guc(&normalized_up) {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PolicyClauseSpan {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PolicyClause {
    Using,
    WithCheck,
}

fn is_sql_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

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

fn direct_tenant_operand(side: &str) -> bool {
    let compact = side
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    let trimmed = compact.trim_matches(|ch: char| matches!(ch, '(' | ')'));
    matches!(trimmed, "tenant_record_id" | "tenant_record_id::text")
}

fn previous_predicate_boundary(lower_sql: &str, end: usize) -> usize {
    const BOUNDARIES: [&str; 2] = [" and ", " or "];
    BOUNDARIES
        .iter()
        .filter_map(|boundary| {
            lower_sql[..end]
                .rfind(boundary)
                .map(|index| index + boundary.len())
        })
        .max()
        .unwrap_or(0)
}

fn next_predicate_boundary(lower_sql: &str, start: usize) -> usize {
    const BOUNDARIES: [&str; 2] = [" and ", " or "];
    BOUNDARIES
        .iter()
        .filter_map(|boundary| {
            lower_sql[start..]
                .find(boundary)
                .map(|index| start + index)
        })
        .min()
        .unwrap_or(lower_sql.len())
}

/// Require the tenant column and tenant session key to participate in the same
/// equality comparison. Co-presence in unrelated boolean terms is not tenant
/// isolation evidence. This bounded recognizer intentionally accepts only the
/// direct tenant identifier (optionally cast to text) on one side; the other
/// side may wrap the exact `current_setting(...)` call, as the shipped migration
/// does with `nullif`.
fn policy_binds_tenant_session_equality(policy_sql: &str) -> bool {
    let lower = policy_sql.to_ascii_lowercase();
    let bytes = policy_sql.as_bytes();

    for equality in 0..bytes.len() {
        if bytes[equality] != b'=' {
            continue;
        }
        let previous = equality.checked_sub(1).and_then(|index| bytes.get(index));
        let next = bytes.get(equality + 1);
        if previous.is_some_and(|byte| matches!(*byte, b'<' | b'>' | b'!' | b'='))
            || next.is_some_and(|byte| matches!(*byte, b'<' | b'>' | b'='))
        {
            continue;
        }

        let left_start = previous_predicate_boundary(&lower, equality);
        let right_end = next_predicate_boundary(&lower, equality + 1);
        let left = &policy_sql[left_start..equality];
        let right = &policy_sql[equality + 1..right_end];
        if (direct_tenant_operand(left) && declares_tenant_session_guc(right))
            || (declares_tenant_session_guc(left) && direct_tenant_operand(right))
        {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PolicyCommand {
    All,
    Select,
    Insert,
    Update,
    Delete,
}

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
/// independently admit rows. PostgreSQL permissive policies are OR-composed.
/// Command semantics determine which tenant-bound predicates are mandatory:
/// read-capable policies require `USING`, insert requires `WITH CHECK`, and
/// `ALL`/`UPDATE` reuse a valid `USING` for writes only when `WITH CHECK` is
/// omitted. Restrictive policies are AND-composed and may add narrower
/// conditions without duplicating the tenant predicate.
fn tenant_policies_bind_session_guc(normalized_sql: &str) -> bool {
    const CREATE_POLICY: &str = "create policy";

    let lower = normalized_sql.to_ascii_lowercase();
    let mut search_from = 0usize;
    let mut saw_policy = false;
    while let Some(relative) = lower[search_from..].find(CREATE_POLICY) {
        saw_policy = true;
        let start = search_from + relative;
        let statement_tail = &normalized_sql[start..];
        let statement_end = statement_tail.find(';').unwrap_or(statement_tail.len());
        let statement = &statement_tail[..statement_end];
        if !policy_is_restrictive(statement)
            && (!declares_tenant_session_guc(statement)
                || !policy_binds_tenant_identifier(statement)
                || !policy_row_predicates_bind_tenant_session(statement))
        {
            return false;
        }
        search_from = start + CREATE_POLICY.len();
    }
    saw_policy
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
        PolicyClause, PolicyCommand, canonicalize_concurrent_index_modifier,
        declares_tenant_session_guc, policy_binds_tenant_identifier,
        policy_binds_tenant_session_equality, policy_clause_span, policy_command,
        policy_is_restrictive, policy_row_predicates_bind_tenant_session,
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
    fn tenant_session_witness_must_be_relationally_bound() {
        assert!(policy_binds_tenant_session_equality(
            "tenant_record_id::text = nullif ( current_setting ( 'tepp.current_tenant_record_id' , true ) , )"
        ));
        assert!(policy_binds_tenant_session_equality(
            "current_setting ( 'tepp.current_tenant_record_id' , true ) = tenant_record_id::text"
        ));
        assert!(!policy_binds_tenant_session_equality(
            "tenant_record_id is not null and current_setting ( 'tepp.current_tenant_record_id' , true ) is not null"
        ));
        assert!(!policy_binds_tenant_session_equality(
            "tenant_record_id::text = document_record_id::text and current_setting ( 'tepp.current_tenant_record_id' , true ) is not null"
        ));
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
