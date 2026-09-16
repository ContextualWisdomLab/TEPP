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
    let normalized_up = validation::normalize_migration_sql(catalog.up_sql())
        .ok_or(MigrationContractError::EmptyMigrationSql)?;
    let normalized_down = validation::normalize_migration_sql(catalog.down_sql())
        .ok_or(MigrationContractError::EmptyMigrationSql)?;
    let requires_runtime_role = validation::declares_row_level_security(&normalized_up);
    let normalized = MigrationCatalog::from_sql(&normalized_up, &normalized_down);
    core::validate_migration_catalog(&normalized)?;
    if requires_runtime_role && !runtime_role_declared {
        return Err(MigrationContractError::MissingAppRuntimeRole);
    }
    Ok(())
}
