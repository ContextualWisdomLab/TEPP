//! Embedded migration catalog boundary and PostgreSQL table-declaration canonicalization.
//!
//! The lexical facade normalizes comments and quoted regions before this module
//! runs. PostgreSQL permits persistence modifiers between `CREATE` and `TABLE`;
//! those modifiers are validation syntax, not part of the durable table name.
//! This boundary canonicalizes only those bounded declaration prefixes, then
//! delegates all naming, tenant, temporal, RLS, and table-body invariants to the
//! existing migration-core implementation.

#[path = "migration_core_impl.rs"]
mod implementation;
#[path = "migration_rls_table_state.rs"]
mod rls_table_state;
#[path = "migration_runtime_role_executor.rs"]
mod runtime_role_executor;
#[path = "migration_runtime_role_grantor.rs"]
mod runtime_role_grantor;
#[path = "migration_runtime_role_membership.rs"]
mod runtime_role_membership;
#[path = "migration_transaction_projection.rs"]
mod transaction_projection;

use crate::MigrationContractError;
pub use implementation::MigrationCatalog;

/// Project already-normalized SQL onto the statements that survive PostgreSQL transaction outcome.
///
/// This is the shared transaction authority for the facade lifecycle/RLS checks
/// and the structural core. Returning `None` means final durable state cannot be
/// proven locally because the input contains savepoint/two-phase ambiguity or an
/// unterminated explicit transaction.
pub(super) fn project_committed_sql(sql: &str) -> Option<String> {
    transaction_projection::project_committed_statements(sql)
}

/// Detect a committed policy-definition mutation not yet owned by the final-policy state model.
///
/// PostgreSQL `ALTER POLICY` can independently replace the role list, `USING`,
/// and `WITH CHECK` clauses while omitted clauses retain prior state. `DROP POLICY`
/// removes the policy definition entirely. Until this bounded validator owns the
/// policy identity/state fold, accepting historical `CREATE POLICY` evidence after
/// either mutation would be fail-open. The input is already lexically normalized
/// and transaction-projected, so statement-first token matching is sufficient and
/// comments/literals cannot manufacture these markers.
fn contains_unsupported_policy_mutation(sql: &str) -> bool {
    sql.split(';').any(|statement| {
        let mut tokens = statement.split_whitespace();
        let Some(verb) = tokens.next() else {
            return false;
        };
        matches!(verb.to_ascii_uppercase().as_str(), "ALTER" | "DROP")
            && tokens
                .next()
                .is_some_and(|token| token.eq_ignore_ascii_case("POLICY"))
    })
}

/// Return the next whitespace-delimited token span beginning at or after `from`.
///
/// Input has already crossed the shared PostgreSQL lexical authority. This
/// cursor exists only to retain the exact byte boundary after an ALTER TABLE
/// target; it does not interpret comments, quoted bodies, or identifiers again.
fn next_sql_token_span(sql: &str, from: usize) -> Option<(usize, usize)> {
    let tail = sql.get(from..)?;
    let mut token_start = None;

    for (offset, ch) in tail.char_indices() {
        if token_start.is_none() {
            if !ch.is_whitespace() {
                token_start = Some(from + offset);
            }
            continue;
        }
        if ch.is_whitespace() {
            if let Some(start) = token_start {
                return Some((start, from + offset));
            }
        }
    }

    token_start.map(|start| (start, sql.len()))
}

/// Compare one normalized token span with an ASCII PostgreSQL keyword.
fn token_span_eq(sql: &str, span: (usize, usize), keyword: &str) -> bool {
    sql.get(span.0..span.1)
        .is_some_and(|token| token.eq_ignore_ascii_case(keyword))
}

/// Return whether one ALTER TABLE action begins with destructive `DROP`.
fn alter_table_action_starts_with_drop(action: &str) -> bool {
    next_sql_token_span(action, 0).is_some_and(|span| token_span_eq(action, span, "DROP"))
}

/// Detect destructive DROP actions in PostgreSQL's comma-separated ALTER TABLE action list.
///
/// Action commas are recognized only outside expression parentheses and
/// array/subscript brackets. That keeps commas inside CHECK/function expressions
/// or `ARRAY[...]` from manufacturing action boundaries. Unbalanced delimiters
/// fail closed because malformed structure cannot prove the absence of a later
/// destructive action.
fn alter_table_actions_include_drop(actions: &str) -> bool {
    let mut parenthesis_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut action_start = 0usize;

    for (index, ch) in actions.char_indices() {
        match ch {
            '(' => parenthesis_depth += 1,
            ')' => {
                if parenthesis_depth == 0 {
                    return true;
                }
                parenthesis_depth -= 1;
            }
            '[' => bracket_depth += 1,
            ']' => {
                if bracket_depth == 0 {
                    return true;
                }
                bracket_depth -= 1;
            }
            ',' if parenthesis_depth == 0 && bracket_depth == 0 => {
                if alter_table_action_starts_with_drop(&actions[action_start..index]) {
                    return true;
                }
                action_start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    if parenthesis_depth != 0 || bracket_depth != 0 {
        return true;
    }
    alter_table_action_starts_with_drop(&actions[action_start..])
}

/// Return whether one committed statement mutates table final state beyond the bounded model.
///
/// `DROP TABLE` removes the durable relation. Standalone PostgreSQL `RENAME`
/// forms make historical table/column identities stale. The ordinary
/// `ALTER TABLE ... action [, ...]` form can also contain destructive `DROP`
/// actions such as `DROP COLUMN` or `DROP CONSTRAINT`; every top-level action is
/// inspected so an additive first action cannot hide a later destructive one.
/// Target parsing is positional, which keeps a table literally named `rename`
/// or `drop` from being confused with an action after lexical normalization.
fn statement_has_unsupported_table_final_state_mutation(statement: &str) -> bool {
    let Some(first) = next_sql_token_span(statement, 0) else {
        return false;
    };
    let Some(second) = next_sql_token_span(statement, first.1) else {
        return false;
    };

    if token_span_eq(statement, first, "DROP") && token_span_eq(statement, second, "TABLE") {
        return true;
    }
    if !token_span_eq(statement, first, "ALTER") || !token_span_eq(statement, second, "TABLE") {
        return false;
    }

    let mut cursor = second.1;
    let Some(mut target) = next_sql_token_span(statement, cursor) else {
        return false;
    };
    if token_span_eq(statement, target, "IF") {
        let Some(exists) = next_sql_token_span(statement, target.1) else {
            return true;
        };
        if !token_span_eq(statement, exists, "EXISTS") {
            return true;
        }
        target = match next_sql_token_span(statement, exists.1) {
            Some(span) => span,
            None => return true,
        };
    }
    if token_span_eq(statement, target, "ONLY") {
        target = match next_sql_token_span(statement, target.1) {
            Some(span) => span,
            None => return true,
        };
    }

    cursor = target.1;
    if let Some(star) = next_sql_token_span(statement, cursor) {
        if statement.get(star.0..star.1) == Some("*") {
            cursor = star.1;
        }
    }

    let Some(actions) = statement.get(cursor..) else {
        return true;
    };
    let Some(first_action) = next_sql_token_span(actions, 0) else {
        return true;
    };
    if token_span_eq(actions, first_action, "RENAME") {
        return true;
    }
    alter_table_actions_include_drop(actions)
}

/// Detect committed table removals or identity/destructive mutations not yet owned by final-table state.
///
/// The input has already crossed lexical normalization and transaction outcome,
/// so rolled-back mutations never reach this boundary. Until a first-class
/// table aggregate owns create/drop/rename/recreate plus column/constraint state,
/// accepting historical `CREATE TABLE` evidence after these mutations would be
/// fail-open and is therefore rejected explicitly.
fn contains_unsupported_table_final_state_mutation(sql: &str) -> bool {
    sql.split(';')
        .any(statement_has_unsupported_table_final_state_mutation)
}

/// Validate migration SQL after canonicalizing PostgreSQL table persistence modifiers.
///
/// `UNLOGGED`, `TEMP`/`TEMPORARY`, and PostgreSQL's compatibility
/// `GLOBAL`/`LOCAL TEMP[TEMPORARY]` spellings must traverse the same table-name
/// and table-body contracts as ordinary `CREATE TABLE`. The canonicalized copy
/// exists only for validation; executable migration SQL is never rewritten.
/// Runtime-role membership safety consumes only statements whose effects survive
/// explicit transaction outcome. Executor-relative grantor identity is projected
/// before that transaction filter so in-transaction `SET LOCAL ROLE` and session
/// authorization still identify the grantor that PostgreSQL recorded, while a
/// later rollback cannot donate false membership or revocation evidence. The
/// structural validator and facade-level RLS witnesses receive the same committed
/// final-state projection, so rolled-back DDL or policy composition cannot
/// satisfy naming, tenant, temporal, RLS, or governance contracts. RLS table
/// enablement is additionally folded in statement order so a committed trailing
/// `DISABLE` or `NO FORCE` cannot reuse stale positive evidence from earlier SQL.
/// Committed `ALTER POLICY` and `DROP POLICY` are temporarily rejected until
/// policy identity, clause replacement, and removal have their own final-state
/// authority. Committed `DROP TABLE`, standalone table/column rename forms, and
/// destructive ALTER TABLE DROP actions are likewise rejected until table,
/// column, constraint, removal/recreation, and dependent-object effects are
/// represented by a first-class final-table aggregate. Mutations removed by the
/// transaction projection never reach either bounded fail-closed boundary.
///
/// # Errors
///
/// Returns the same naming, tenant, temporal, RLS, or structural contract
/// errors as the underlying migration validator, plus `MissingAppRuntimeRole`
/// when the application runtime has an unsafe or unprovable PostgreSQL
/// membership path or transaction outcome.
pub fn validate_migration_catalog(
    catalog: &MigrationCatalog,
) -> Result<(), MigrationContractError> {
    let Some(grantor_sql) =
        runtime_role_executor::project_executor_relative_grantors(catalog.up_sql())
    else {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    };
    let Some(committed_up) = project_committed_sql(catalog.up_sql()) else {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    };
    let Some(committed_down) = project_committed_sql(catalog.down_sql()) else {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    };
    let Some(committed_grantor_sql) = project_committed_sql(&grantor_sql) else {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    };

    if contains_unsupported_table_final_state_mutation(&committed_up) {
        return Err(MigrationContractError::UnsupportedTableFinalStateMutation);
    }
    if contains_unsupported_policy_mutation(&committed_up) {
        return Err(MigrationContractError::MissingRlsPolicy);
    }
    let committed_requires_runtime_role =
        super::validation::declares_row_level_security(&committed_up);
    if committed_requires_runtime_role
        && !rls_table_state::final_rls_table_states_are_safe(&committed_up)
    {
        return Err(MigrationContractError::MissingRlsEnable);
    }
    if committed_requires_runtime_role && !super::declares_tenant_session_guc(&committed_up) {
        return Err(MigrationContractError::MissingTenantSessionGuc);
    }
    if committed_requires_runtime_role && !super::tenant_policies_bind_session_guc(&committed_up) {
        return Err(MigrationContractError::MissingRlsPolicy);
    }

    if !runtime_role_membership::runtime_membership_is_rls_safe(&committed_up)
        || !runtime_role_grantor::runtime_membership_grantors_are_rls_safe(
            &committed_grantor_sql,
        )
    {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    }

    let canonical_up = canonicalize_table_persistence_modifiers(&committed_up);
    let canonical_catalog = MigrationCatalog::from_sql(&canonical_up, &committed_down);
    implementation::validate_migration_catalog(&canonical_catalog)
}

/// Return whether `ch` can continue a PostgreSQL unquoted identifier.
///
/// This mirrors the migration-core token boundary so `CREATE` embedded in an
/// identifier cannot start a synthetic declaration while scanning modifiers.
fn is_postgresql_identifier_continuation(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || !ch.is_ascii()
}

/// Match one ASCII keyword at `start` with PostgreSQL identifier boundaries.
fn bounded_keyword_end(sql: &str, start: usize, keyword: &str) -> Option<usize> {
    let end = start.checked_add(keyword.len())?;
    let candidate = sql.get(start..end)?;
    if !candidate.eq_ignore_ascii_case(keyword) {
        return None;
    }
    if sql[..start]
        .chars()
        .next_back()
        .is_some_and(is_postgresql_identifier_continuation)
    {
        return None;
    }
    if sql[end..]
        .chars()
        .next()
        .is_some_and(is_postgresql_identifier_continuation)
    {
        return None;
    }
    Some(end)
}

/// Consume at least one SQL whitespace character before matching `keyword`.
fn keyword_after_required_whitespace(sql: &str, from: usize, keyword: &str) -> Option<usize> {
    let rest = sql.get(from..)?;
    let mut consumed = 0usize;
    for ch in rest.chars() {
        if !ch.is_whitespace() {
            break;
        }
        consumed += ch.len_utf8();
    }
    if consumed == 0 {
        return None;
    }
    bounded_keyword_end(sql, from + consumed, keyword)
}

/// Return the byte immediately after `TABLE` for one supported modifier-bearing declaration.
///
/// PostgreSQL 18 accepts `UNLOGGED`, `TEMP`/`TEMPORARY`, and compatibility
/// `GLOBAL`/`LOCAL TEMP[TEMPORARY]` prefixes. Ordinary `CREATE TABLE` is left
/// untouched so this helper cannot broaden the legacy parser's authority.
fn modifier_table_declaration_end(sql: &str, create_start: usize) -> Option<usize> {
    let create_end = bounded_keyword_end(sql, create_start, "CREATE")?;

    let first_start = {
        let rest = sql.get(create_end..)?;
        let mut consumed = 0usize;
        for ch in rest.chars() {
            if !ch.is_whitespace() {
                break;
            }
            consumed += ch.len_utf8();
        }
        if consumed == 0 {
            return None;
        }
        create_end + consumed
    };

    if let Some(unlogged_end) = bounded_keyword_end(sql, first_start, "UNLOGGED") {
        return keyword_after_required_whitespace(sql, unlogged_end, "TABLE");
    }
    if let Some(temp_end) = bounded_keyword_end(sql, first_start, "TEMP") {
        return keyword_after_required_whitespace(sql, temp_end, "TABLE");
    }
    if let Some(temporary_end) = bounded_keyword_end(sql, first_start, "TEMPORARY") {
        return keyword_after_required_whitespace(sql, temporary_end, "TABLE");
    }

    let scope_end = bounded_keyword_end(sql, first_start, "GLOBAL")
        .or_else(|| bounded_keyword_end(sql, first_start, "LOCAL"))?;
    let temp_end = keyword_after_required_whitespace(sql, scope_end, "TEMP")
        .or_else(|| keyword_after_required_whitespace(sql, scope_end, "TEMPORARY"))?;
    keyword_after_required_whitespace(sql, temp_end, "TABLE")
}

/// Canonicalize supported PostgreSQL table persistence modifiers for validation only.
///
/// The scanner changes only bounded declaration prefixes and copies every other
/// byte verbatim. This preserves declared table spelling, `IF NOT EXISTS`, table
/// bodies, policy evidence, and statement locality while making all supported
/// table forms visible to the one existing `CREATE TABLE` structural authority.
fn canonicalize_table_persistence_modifiers(sql: &str) -> String {
    let bytes = sql.as_bytes();
    let mut output = String::with_capacity(sql.len());
    let mut copied_through = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        if matches!(bytes[index], b'C' | b'c') {
            if let Some(declaration_end) = modifier_table_declaration_end(sql, index) {
                output.push_str(&sql[copied_through..index]);
                output.push_str("CREATE TABLE");
                copied_through = declaration_end;
                index = declaration_end;
                continue;
            }
        }
        index += 1;
    }

    if copied_through == 0 {
        return sql.to_owned();
    }
    output.push_str(&sql[copied_through..]);
    output
}

#[cfg(test)]
mod tests {
    use super::{
        canonicalize_table_persistence_modifiers, contains_unsupported_policy_mutation,
        contains_unsupported_table_final_state_mutation, project_committed_sql,
    };

    #[test]
    fn table_modifier_canonicalization_preserves_the_declared_name_and_body() {
        let sql = "CREATE\nGLOBAL\tTEMPORARY TABLE IF NOT EXISTS derived_cache (derived_cache_id uuid);";
        assert_eq!(
            canonicalize_table_persistence_modifiers(sql),
            "CREATE TABLE IF NOT EXISTS derived_cache (derived_cache_id uuid);"
        );
    }

    #[test]
    fn ordinary_and_identifier_attached_create_tokens_are_unchanged() {
        for sql in [
            "CREATE TABLE tenant_record (tenant_record_id uuid);",
            "prefixCREATE UNLOGGED TABLE derived_cache (derived_cache_id uuid);",
            "CREATE_UNLOGGED TABLE derived_cache (derived_cache_id uuid);",
        ] {
            assert_eq!(canonicalize_table_persistence_modifiers(sql), sql);
        }
    }

    #[test]
    fn rolled_back_structural_statement_is_absent_from_committed_projection() {
        let projected = project_committed_sql(
            "CREATE TABLE durable_record (durable_record_id uuid); BEGIN; CREATE TABLE rolled_back_record (rolled_back_record_id uuid); ROLLBACK;",
        )
        .expect("simple rollback outcome must project");
        assert!(projected.contains("CREATE TABLE durable_record"));
        assert!(!projected.contains("rolled_back_record"));
    }

    #[test]
    fn table_final_state_mutation_detection_is_statement_token_and_action_bounded() {
        for sql in [
            "DROP\nTABLE tenant_record ;",
            "DROP TABLE IF EXISTS tenant_record CASCADE ;",
            "ALTER TABLE tenant_record RENAME TO tenant_record_archive ;",
            "ALTER TABLE IF EXISTS ONLY tenant_record RENAME COLUMN tenant_record_id TO tenant_key ;",
            "ALTER TABLE tenant_record DROP COLUMN tenant_record_id CASCADE ;",
            "ALTER TABLE tenant_record ADD COLUMN auxiliary_flag boolean, DROP COLUMN tenant_record_id CASCADE ;",
        ] {
            assert!(contains_unsupported_table_final_state_mutation(sql));
        }
        for sql in [
            "SELECT drop_table_marker ; CREATE TABLE tenant_record ( tenant_record_id uuid ) ;",
            "ALTER TABLE tenant_record ENABLE ROW LEVEL SECURITY ;",
            "ALTER TABLE rename ENABLE ROW LEVEL SECURITY ;",
            "ALTER TABLE tenant_record ADD COLUMN drop_flag boolean ;",
            "ALTER TABLE tenant_record ADD CONSTRAINT tenant_record_shape CHECK (some_func(a, drop_flag)) ;",
            "ALTER TABLE tenant_record ADD CONSTRAINT tenant_record_array_shape CHECK (some_func(ARRAY[a, drop_flag])) ;",
        ] {
            assert!(!contains_unsupported_table_final_state_mutation(sql));
        }
    }

    #[test]
    fn policy_mutation_detection_is_statement_and_token_bounded() {
        assert!(contains_unsupported_policy_mutation(
            "ALTER\nPOLICY tenant_isolation ON tenant_record USING ( true ) ;"
        ));
        assert!(contains_unsupported_policy_mutation(
            "DROP\tPOLICY tenant_isolation ON tenant_record ;"
        ));
        assert!(!contains_unsupported_policy_mutation(
            "SELECT alter_policy_marker, drop_policy_marker ; CREATE POLICY tenant_isolation ON tenant_record USING ( true ) ;"
        ));
    }
}
