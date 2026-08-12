//! Fail-closed persistence validation and migration errors.

use std::fmt;

/// A fail-closed persistence-domain error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PersistenceError {
    /// A document identity already exists as an open system-time version.
    DuplicateDocumentRecord,
    /// An audit event identity was reused, violating append-only history.
    ImmutableAuditViolation,
    /// No historical version matches the requested as-of times.
    HistoricalVersionNotFound,
    /// A document digest failed closed validation.
    InvalidContentDigest,
    /// A live SQL batch was empty after comment/whitespace stripping.
    EmptySqlBatch,
    /// A live SQL statement or transport operation failed closed.
    SqlExecutionFailed,
    /// A live database URL failed scheme/host validation.
    DatabaseUrlInvalid,
    /// Live `SQLx` wiring was requested without a configured transport URL.
    LiveAdapterNotConfigured,
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::DuplicateDocumentRecord => "duplicate document record",
            Self::ImmutableAuditViolation => "immutable audit violation",
            Self::HistoricalVersionNotFound => "historical version not found",
            Self::InvalidContentDigest => "invalid content digest",
            Self::EmptySqlBatch => "empty sql batch",
            Self::SqlExecutionFailed => "sql execution failed",
            Self::DatabaseUrlInvalid => "database url invalid",
            Self::LiveAdapterNotConfigured => "live adapter not configured",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PersistenceError {}

/// Migration SQL contract violations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MigrationContractError {
    /// A table or constrained object used a single-word identifier.
    SingleWordObjectName,
    /// A core analytical table lacked a tenant boundary column.
    MissingTenantBoundary,
    /// A core analytical table lacked required temporal columns.
    MissingTemporalColumns,
    /// Embedded or supplied migration SQL was empty or unreadable.
    EmptyMigrationSql,
}

impl fmt::Display for MigrationContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SingleWordObjectName => "single-word database object name",
            Self::MissingTenantBoundary => "missing tenant boundary column",
            Self::MissingTemporalColumns => "missing temporal columns",
            Self::EmptyMigrationSql => "empty migration sql",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for MigrationContractError {}

#[cfg(test)]
mod tests {
    use super::{MigrationContractError, PersistenceError};

    #[test]
    fn error_messages_are_stable() {
        assert_eq!(
            PersistenceError::DuplicateDocumentRecord.to_string(),
            "duplicate document record"
        );
        assert_eq!(
            PersistenceError::ImmutableAuditViolation.to_string(),
            "immutable audit violation"
        );
        assert_eq!(
            PersistenceError::HistoricalVersionNotFound.to_string(),
            "historical version not found"
        );
        assert_eq!(
            PersistenceError::InvalidContentDigest.to_string(),
            "invalid content digest"
        );
        assert_eq!(
            PersistenceError::EmptySqlBatch.to_string(),
            "empty sql batch"
        );
        assert_eq!(
            PersistenceError::SqlExecutionFailed.to_string(),
            "sql execution failed"
        );
        assert_eq!(
            PersistenceError::DatabaseUrlInvalid.to_string(),
            "database url invalid"
        );
        assert_eq!(
            PersistenceError::LiveAdapterNotConfigured.to_string(),
            "live adapter not configured"
        );
        assert_eq!(
            MigrationContractError::SingleWordObjectName.to_string(),
            "single-word database object name"
        );
        assert_eq!(
            MigrationContractError::MissingTenantBoundary.to_string(),
            "missing tenant boundary column"
        );
        assert_eq!(
            MigrationContractError::MissingTemporalColumns.to_string(),
            "missing temporal columns"
        );
        assert_eq!(
            MigrationContractError::EmptyMigrationSql.to_string(),
            "empty migration sql"
        );
    }
}
