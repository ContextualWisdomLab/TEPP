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
    let normalized = MigrationCatalog::from_sql(&normalized_up, &normalized_down);
    core::validate_migration_catalog(&normalized)?;
    if requires_runtime_role && !runtime_role_declared {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    }
    Ok(())
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
    use super::canonicalize_concurrent_index_modifier;

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
