//! Fail-closed persistence validation and migration errors.

use operational_log::OperationalLogError;
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
    /// Source text was supplied to an `audit_event` insert.
    SourceTextNotAuditable,
    /// Source identity was supplied to an `audit_event` insert.
    SourceIdentityNotAuditable,
    /// Blanket PII masking was treated as an `audit_event` insert grant.
    BlanketMaskIsNotAuditAuthorization,
    /// A concurrent writer won the open-row lock or serialization contest.
    ConcurrentWriteConflict,
    /// A restored snapshot failed integrity revalidation and is not usable.
    RestoreIntegrityFailed,
    /// A text segment had a negative or inverted UTF-8 byte span.
    InvalidTextSegment,
    /// A retention, hold, deletion, or tombstone record failed closed validation.
    InvalidRetentionLifecycle,
    /// An active legal hold blocked completed deletion.
    LegalHoldBlocksDeletion,
    /// Tombstoned evidence cannot be restored, and raw-source deletion cannot keep reproduction available.
    UngovernedEvidenceRestore,
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
            Self::SourceTextNotAuditable => "source text cannot appear in an audit event",
            Self::SourceIdentityNotAuditable => "source identity cannot appear in an audit event",
            Self::BlanketMaskIsNotAuditAuthorization => {
                "blanket PII masking is not audit-event authorization"
            }
            Self::ConcurrentWriteConflict => "concurrent write conflict",
            Self::RestoreIntegrityFailed => "restore integrity failed",
            Self::InvalidTextSegment => "invalid text segment",
            Self::InvalidRetentionLifecycle => "invalid retention lifecycle",
            Self::LegalHoldBlocksDeletion => "legal hold blocks deletion",
            Self::UngovernedEvidenceRestore => "ungoverned evidence restore",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PersistenceError {}

impl From<OperationalLogError> for PersistenceError {
    fn from(error: OperationalLogError) -> Self {
        match error {
            OperationalLogError::SourceTextNotLoggable => Self::SourceTextNotAuditable,
            OperationalLogError::SourceIdentityNotLoggable => Self::SourceIdentityNotAuditable,
            OperationalLogError::BlanketMaskIsNotAuthorization => {
                Self::BlanketMaskIsNotAuditAuthorization
            }
            _ => Self::InvalidAuditEvent,
        }
    }
}

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
    /// Retention/legal-hold migration declarations were incomplete.
    MissingRetentionLegalHold,
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
            Self::MissingRetentionLegalHold => "missing retention legal hold",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for MigrationContractError {}

#[cfg(test)]
mod tests {
    use super::{MigrationContractError, PersistenceError};
    use operational_log::OperationalLogError;

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
            PersistenceError::SourceTextNotAuditable.to_string(),
            "source text cannot appear in an audit event"
        );
        assert_eq!(
            PersistenceError::SourceIdentityNotAuditable.to_string(),
            "source identity cannot appear in an audit event"
        );
        assert_eq!(
            PersistenceError::BlanketMaskIsNotAuditAuthorization.to_string(),
            "blanket PII masking is not audit-event authorization"
        );
        assert_eq!(
            PersistenceError::RestoreIntegrityFailed.to_string(),
            "restore integrity failed"
        );
        assert_eq!(
            PersistenceError::InvalidTextSegment.to_string(),
            "invalid text segment"
        );
        assert_eq!(
            PersistenceError::InvalidRetentionLifecycle.to_string(),
            "invalid retention lifecycle"
        );
        assert_eq!(
            PersistenceError::LegalHoldBlocksDeletion.to_string(),
            "legal hold blocks deletion"
        );
        assert_eq!(
            PersistenceError::UngovernedEvidenceRestore.to_string(),
            "ungoverned evidence restore"
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
        assert_eq!(
            PersistenceError::from(OperationalLogError::SourceTextNotLoggable),
            PersistenceError::SourceTextNotAuditable
        );
        assert_eq!(
            PersistenceError::from(OperationalLogError::SourceIdentityNotLoggable),
            PersistenceError::SourceIdentityNotAuditable
        );
        assert_eq!(
            PersistenceError::from(OperationalLogError::BlanketMaskIsNotAuthorization),
            PersistenceError::BlanketMaskIsNotAuditAuthorization
        );
        assert_eq!(
            PersistenceError::from(OperationalLogError::InvalidLogPayload),
            PersistenceError::InvalidAuditEvent
        );
        assert_eq!(
            MigrationContractError::MissingRetentionLegalHold.to_string(),
            "missing retention legal hold"
        );
    }
}
