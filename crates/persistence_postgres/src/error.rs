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
    /// Live pool sizing or acquire options failed validation.
    PoolOptionsInvalid,
    /// Live `SQLx` wiring was requested without a configured transport URL.
    LiveAdapterNotConfigured,
    /// A membership assignment violated exactly-one, weight, window, or label contracts.
    InvalidMembershipAssignment,
    /// An entity record had an empty, oversized, or hostile type label.
    InvalidEntityRecord,
    /// A project record had an empty, oversized, or hostile status label.
    InvalidProjectRecord,
    /// An event relation violated the closed ERD transition vocabulary.
    InvalidEventRelation,
    /// An event mention reused an instance identity or had an invalid confidence.
    InvalidEventMention,
    /// An event instance had inverted windows or a hostile label.
    InvalidEventInstance,
    /// A source-artifact identity already exists with different immutable fields.
    ConflictingSourceArtifact,
    /// A source artifact had a non-canonical digest, negative size, or hostile label.
    InvalidSourceArtifact,
    /// An audit action code was empty, oversized, or hostile.
    InvalidAuditEvent,
    /// A concurrent writer won the open-row lock or serialization contest.
    ConcurrentWriteConflict,
    /// A restored snapshot failed integrity revalidation and is not usable.
    RestoreIntegrityFailed,
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
            Self::PoolOptionsInvalid => "pool options invalid",
            Self::LiveAdapterNotConfigured => "live adapter not configured",
            Self::InvalidMembershipAssignment => "invalid membership assignment",
            Self::InvalidEntityRecord => "invalid entity record",
            Self::InvalidProjectRecord => "invalid project record",
            Self::InvalidEventRelation => "invalid event relation",
            Self::InvalidEventMention => "invalid event mention",
            Self::InvalidEventInstance => "invalid event instance",
            Self::ConflictingSourceArtifact => "conflicting source artifact",
            Self::InvalidSourceArtifact => "invalid source artifact",
            Self::InvalidAuditEvent => "invalid audit event",
            Self::ConcurrentWriteConflict => "concurrent write conflict",
            Self::RestoreIntegrityFailed => "restore integrity failed",
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
    /// Tenant RLS was declared without enabling FORCE RLS on a table.
    MissingRlsEnable,
    /// Tenant RLS was declared without a multi-word isolation policy.
    MissingRlsPolicy,
    /// Tenant RLS was declared without the non-superuser application role.
    MissingAppRuntimeRole,
    /// Tenant RLS was declared without the session tenant GUC contract.
    MissingTenantSessionGuc,
    /// Append-only immutability triggers or revoke statements were incomplete.
    MissingAppendOnlyTrigger,
    /// Temporal interval ordering checks were incomplete when declared.
    MissingTemporalIntervalConstraint,
}

impl fmt::Display for MigrationContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SingleWordObjectName => "single-word database object name",
            Self::MissingTenantBoundary => "missing tenant boundary column",
            Self::MissingTemporalColumns => "missing temporal columns",
            Self::EmptyMigrationSql => "empty migration sql",
            Self::MissingRlsEnable => "missing row level security enable",
            Self::MissingRlsPolicy => "missing tenant isolation policy",
            Self::MissingAppRuntimeRole => "missing application runtime role",
            Self::MissingTenantSessionGuc => "missing tenant session guc",
            Self::MissingAppendOnlyTrigger => "missing append-only immutability trigger",
            Self::MissingTemporalIntervalConstraint => "missing temporal interval constraint",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for MigrationContractError {}

#[cfg(test)]
mod tests {
    use super::{MigrationContractError, PersistenceError};

    #[test]
    #[allow(clippy::too_many_lines)]
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
            PersistenceError::PoolOptionsInvalid.to_string(),
            "pool options invalid"
        );
        assert_eq!(
            PersistenceError::LiveAdapterNotConfigured.to_string(),
            "live adapter not configured"
        );
        assert_eq!(
            PersistenceError::InvalidMembershipAssignment.to_string(),
            "invalid membership assignment"
        );
        assert_eq!(
            PersistenceError::InvalidEntityRecord.to_string(),
            "invalid entity record"
        );
        assert_eq!(
            PersistenceError::InvalidProjectRecord.to_string(),
            "invalid project record"
        );
        assert_eq!(
            PersistenceError::ConcurrentWriteConflict.to_string(),
            "concurrent write conflict"
        );
        assert_eq!(
            PersistenceError::InvalidEventRelation.to_string(),
            "invalid event relation"
        );
        assert_eq!(
            PersistenceError::InvalidEventMention.to_string(),
            "invalid event mention"
        );
        assert_eq!(
            PersistenceError::InvalidEventInstance.to_string(),
            "invalid event instance"
        );
        assert_eq!(
            PersistenceError::ConflictingSourceArtifact.to_string(),
            "conflicting source artifact"
        );
        assert_eq!(
            PersistenceError::InvalidSourceArtifact.to_string(),
            "invalid source artifact"
        );
        assert_eq!(
            PersistenceError::InvalidAuditEvent.to_string(),
            "invalid audit event"
        );
        assert_eq!(
            PersistenceError::RestoreIntegrityFailed.to_string(),
            "restore integrity failed"
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
        assert_eq!(
            MigrationContractError::MissingRlsEnable.to_string(),
            "missing row level security enable"
        );
        assert_eq!(
            MigrationContractError::MissingRlsPolicy.to_string(),
            "missing tenant isolation policy"
        );
        assert_eq!(
            MigrationContractError::MissingAppRuntimeRole.to_string(),
            "missing application runtime role"
        );
        assert_eq!(
            MigrationContractError::MissingTenantSessionGuc.to_string(),
            "missing tenant session guc"
        );
        assert_eq!(
            MigrationContractError::MissingAppendOnlyTrigger.to_string(),
            "missing append-only immutability trigger"
        );
        assert_eq!(
            MigrationContractError::MissingTemporalIntervalConstraint.to_string(),
            "missing temporal interval constraint"
        );
    }
}
