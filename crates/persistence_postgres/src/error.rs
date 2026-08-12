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
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::DuplicateDocumentRecord => "duplicate document record",
            Self::ImmutableAuditViolation => "immutable audit violation",
            Self::HistoricalVersionNotFound => "historical version not found",
            Self::InvalidContentDigest => "invalid content digest",
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
