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
/// satisfy naming, tenant, temporal, RLS, or governance contracts.
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

    let committed_requires_runtime_role =
        super::validation::declares_row_level_security(&committed_up);
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
    use super::{canonicalize_table_persistence_modifiers, project_committed_sql};

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
}
