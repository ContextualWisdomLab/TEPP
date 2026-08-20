//! SQL contracts for retention policy, legal hold, deletion, and tombstones.

use crate::PersistenceError;
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

const DATA_CLASSES: [&str; 6] = [
    "raw_source",
    "identity_mapping",
    "derived_result",
    "provider_payload",
    "audit_evidence",
    "export_cache",
];
const PURPOSES: [&str; 4] = [
    "psychometric_analysis",
    "legal_preservation",
    "operations_audit",
    "export_fulfillment",
];
const POLICY_STATUSES: [&str; 2] = ["active", "superseded"];
const HOLD_SCOPES: [&str; 2] = ["document", "tenant"];
const HOLD_STATUSES: [&str; 2] = ["active", "released"];
const DELETION_KINDS: [&str; 3] = [
    "logical_revocation",
    "cache_export_removal",
    "identity_tombstone",
];
const REQUEST_STATUSES: [&str; 4] = [
    "requested",
    "completed",
    "blocked_by_hold",
    "reproduction_limited",
];
const REPRODUCTION_STATUSES: [&str; 3] = ["unavailable", "limited", "unaffected"];

/// Tenant-scoped retention policy for one data class and purpose.
#[derive(Clone, Debug, PartialEq)]
pub struct RetentionPolicyRecord {
    /// Primary key for this policy identity.
    pub retention_policy_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Governed data class (`raw_source`, `identity_mapping`, ...).
    pub data_class_code: String,
    /// Declared processing purpose bound to the retention period.
    pub processing_purpose_code: String,
    /// Positive retention period in whole days.
    pub retention_period_days: i32,
    /// Policy lifecycle token (`active` or `superseded`).
    pub policy_status_code: String,
    /// Authority citation recorded without raw source text.
    pub authority_citation: String,
    /// System/record time when the policy was asserted.
    pub system_time: SystemTime,
    /// Closed system-time upper bound; required when `policy_status_code` is
    /// `superseded`, and must remain open (`None`) for `active` policies.
    pub system_to: Option<SystemTime>,
    /// Availability time of the policy assertion.
    pub available_time: AvailableTime,
}

impl RetentionPolicyRecord {
    /// Fail-closed vocabulary, period, window, and label validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidRetentionLifecycle`] when the period
    /// is not positive, a label is empty/hostile/unknown, or the status/window
    /// pair is inconsistent with migration `0007`.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        if self.retention_period_days <= 0 {
            return Err(PersistenceError::InvalidRetentionLifecycle);
        }
        validate_vocab(&self.data_class_code, &DATA_CLASSES)?;
        validate_vocab(&self.processing_purpose_code, &PURPOSES)?;
        validate_vocab(&self.policy_status_code, &POLICY_STATUSES)?;
        validate_lifecycle_label(&self.authority_citation)?;
        let open = self.system_to.is_none();
        let active = self.policy_status_code == "active";
        if active != open {
            return Err(PersistenceError::InvalidRetentionLifecycle);
        }
        if let Some(until) = self.system_to.as_ref()
            && until.instant() < self.system_time.instant()
        {
            return Err(PersistenceError::InvalidRetentionLifecycle);
        }
        Ok(())
    }
}

/// Legal or contractual hold that prevents completed deletion.
#[derive(Clone, Debug, PartialEq)]
pub struct LegalHoldRecord {
    /// Primary key for this hold identity.
    pub legal_hold_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Hold scope (`document` or `tenant`).
    pub hold_scope_code: String,
    /// Document identity when `hold_scope_code` is `document`.
    pub held_document_id: Option<Uuid>,
    /// Authority class that imposed the hold.
    pub hold_authority_code: String,
    /// Hold lifecycle token (`active` or `released`).
    pub hold_status_code: String,
    /// Authority citation recorded without raw source text.
    pub authority_citation: String,
    /// System/record time when the hold was asserted.
    pub system_time: SystemTime,
    /// Closed system-time upper bound; required when `hold_status_code` is
    /// `released`, and must remain open (`None`) for `active` holds.
    pub system_to: Option<SystemTime>,
    /// Availability time of the hold assertion.
    pub available_time: AvailableTime,
}

impl LegalHoldRecord {
    /// Fail-closed scope, vocabulary, window, and label validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidRetentionLifecycle`] when document
    /// scope is missing an identity, tenant scope carries one, a label is
    /// hostile/unknown, or the status/window pair is inconsistent.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_vocab(&self.hold_scope_code, &HOLD_SCOPES)?;
        validate_vocab(&self.hold_status_code, &HOLD_STATUSES)?;
        validate_lifecycle_label(&self.hold_authority_code)?;
        validate_lifecycle_label(&self.authority_citation)?;
        let document_scope = self.hold_scope_code == "document";
        if document_scope != self.held_document_id.is_some() {
            return Err(PersistenceError::InvalidRetentionLifecycle);
        }
        let open = self.system_to.is_none();
        let active = self.hold_status_code == "active";
        if active != open {
            return Err(PersistenceError::InvalidRetentionLifecycle);
        }
        if let Some(until) = self.system_to.as_ref()
            && until.instant() < self.system_time.instant()
        {
            return Err(PersistenceError::InvalidRetentionLifecycle);
        }
        Ok(())
    }

    /// True when this hold blocks a completed deletion of `document_id`.
    #[must_use]
    pub fn blocks_deletion(&self, tenant_record_id: Uuid, document_id: Uuid) -> bool {
        self.hold_status_code == "active"
            && self.system_to.is_none()
            && self.tenant_record_id == tenant_record_id
            && (self.hold_scope_code == "tenant" || self.held_document_id == Some(document_id))
    }
}

/// Auditable deletion request that never silently DELETE-s evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct DeletionRequestRecord {
    /// Primary key for this request identity.
    pub deletion_request_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Retention policy that authorized the lifecycle action.
    pub retention_policy_id: Uuid,
    /// Document identity being revoked, tombstoned, or cache-cleared.
    pub target_document_id: Uuid,
    /// Data class being acted on.
    pub target_data_class_code: String,
    /// Purpose used to select the retention policy.
    pub processing_purpose_code: String,
    /// Deletion kind (`logical_revocation`, `cache_export_removal`, `identity_tombstone`).
    pub deletion_kind_code: String,
    /// Request status (`requested`, `completed`, `blocked_by_hold`, `reproduction_limited`).
    pub request_status_code: String,
    /// Optional hold that blocked completion.
    pub legal_hold_id: Option<Uuid>,
    /// System/record time when the request was asserted.
    pub system_time: SystemTime,
    /// Availability time of the request assertion.
    pub available_time: AvailableTime,
}

impl DeletionRequestRecord {
    /// Fail-closed vocabulary and label validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidRetentionLifecycle`] when a code is
    /// unknown or a label is empty/hostile.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_vocab(&self.target_data_class_code, &DATA_CLASSES)?;
        validate_vocab(&self.processing_purpose_code, &PURPOSES)?;
        validate_vocab(&self.deletion_kind_code, &DELETION_KINDS)?;
        validate_vocab(&self.request_status_code, &REQUEST_STATUSES)
    }

    /// Bind this request to the cited retention policy's tenant, class, and purpose.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidRetentionLifecycle`] when vocabulary
    /// fails or the request tenant, policy identity, data class, or purpose
    /// does not match the cited policy.
    pub fn bind_cited_policy(
        &self,
        policy: &RetentionPolicyRecord,
    ) -> Result<(), PersistenceError> {
        self.validate()?;
        if self.retention_policy_id != policy.retention_policy_id
            || self.tenant_record_id != policy.tenant_record_id
            || self.target_data_class_code != policy.data_class_code
            || self.processing_purpose_code != policy.processing_purpose_code
        {
            return Err(PersistenceError::InvalidRetentionLifecycle);
        }
        Ok(())
    }
}

/// Append-only tombstone that records deletion without raw source text.
#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceTombstoneRecord {
    /// Primary key for this tombstone identity.
    pub evidence_tombstone_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Document identity that may no longer be restored.
    pub tombstoned_document_id: Uuid,
    /// Deletion request that produced the tombstone.
    pub deletion_request_id: Uuid,
    /// SHA-256 digest of the deletion action, not of raw source bytes.
    pub evidence_digest: String,
    /// Data class that was tombstoned.
    pub target_data_class_code: String,
    /// Deletion kind that produced the tombstone.
    pub deletion_kind_code: String,
    /// Reproduction consequence (`unavailable`, `limited`, `unaffected`).
    pub reproduction_status_code: String,
    /// System/record time when the tombstone was asserted.
    pub system_time: SystemTime,
    /// Availability time of the tombstone assertion.
    pub available_time: AvailableTime,
}

impl EvidenceTombstoneRecord {
    /// Fail-closed digest, vocabulary, and reproduction-override validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidRetentionLifecycle`] for hostile or
    /// unknown labels and [`PersistenceError::UngovernedEvidenceRestore`] when
    /// raw-source deletion would silently keep reproduction available.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_sha256_hex(&self.evidence_digest)?;
        validate_vocab(&self.target_data_class_code, &DATA_CLASSES)?;
        validate_vocab(&self.deletion_kind_code, &DELETION_KINDS)?;
        validate_vocab(&self.reproduction_status_code, &REPRODUCTION_STATUSES)?;
        let source_revoked = self.target_data_class_code == "raw_source"
            && (self.deletion_kind_code == "identity_tombstone"
                || self.deletion_kind_code == "logical_revocation");
        if source_revoked && self.reproduction_status_code != "unavailable" {
            return Err(PersistenceError::UngovernedEvidenceRestore);
        }
        Ok(())
    }
}

/// Render insert SQL for a validated retention policy.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidRetentionLifecycle`] before any SQL is
/// produced when the record fails validation.
pub fn insert_retention_policy_sql(
    record: &RetentionPolicyRecord,
) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO retention_policy (\
            retention_policy_id, tenant_record_id, data_class_code, \
            processing_purpose_code, retention_period_days, policy_status_code, \
            authority_citation, system_time, system_to, available_time\
        ) VALUES (\
            '{policy}'::uuid, '{tenant}'::uuid, '{class}', \
            '{purpose}', {days}, '{status}', \
            '{citation}', '{system}'::timestamptz, {system_to}, '{available}'::timestamptz\
        )",
        policy = record.retention_policy_id,
        tenant = record.tenant_record_id,
        class = record.data_class_code,
        purpose = record.processing_purpose_code,
        days = record.retention_period_days,
        status = record.policy_status_code,
        citation = record.authority_citation,
        system = record.system_time.to_rfc3339(),
        system_to = optional_timestamptz(record.system_to.as_ref()),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render SQL that atomically supersedes one active retention policy.
///
/// The statement calls the tenant-bound `supersede_retention_policy` function
/// so the predecessor is closed and the successor is inserted in one
/// database transaction. Application code never issues a standalone UPDATE.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidRetentionLifecycle`] before any SQL is
/// produced when the replacement is not an active distinct policy or fails
/// vocabulary/period/label validation.
pub fn supersede_retention_policy_sql(
    predecessor_retention_policy_id: Uuid,
    replacement: &RetentionPolicyRecord,
) -> Result<String, PersistenceError> {
    replacement.validate()?;
    if replacement.policy_status_code != "active"
        || replacement.retention_policy_id == predecessor_retention_policy_id
    {
        return Err(PersistenceError::InvalidRetentionLifecycle);
    }
    Ok(format!(
        "SELECT supersede_retention_policy(\
            '{predecessor}'::uuid, '{replacement}'::uuid, {days}, \
            '{citation}', '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        predecessor = predecessor_retention_policy_id,
        replacement = replacement.retention_policy_id,
        days = replacement.retention_period_days,
        citation = replacement.authority_citation,
        system = replacement.system_time.to_rfc3339(),
        available = replacement.available_time.to_rfc3339(),
    ))
}

/// Render insert SQL for a validated legal hold.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidRetentionLifecycle`] before any SQL is
/// produced when the record fails validation.
pub fn insert_legal_hold_sql(record: &LegalHoldRecord) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO legal_hold (\
            legal_hold_id, tenant_record_id, hold_scope_code, held_document_id, \
            hold_authority_code, hold_status_code, authority_citation, \
            system_time, system_to, available_time\
        ) VALUES (\
            '{hold}'::uuid, '{tenant}'::uuid, '{scope}', {document}, \
            '{authority}', '{status}', '{citation}', \
            '{system}'::timestamptz, {system_to}, '{available}'::timestamptz\
        )",
        hold = record.legal_hold_id,
        tenant = record.tenant_record_id,
        scope = record.hold_scope_code,
        document = optional_uuid(record.held_document_id),
        authority = record.hold_authority_code,
        status = record.hold_status_code,
        citation = record.authority_citation,
        system = record.system_time.to_rfc3339(),
        system_to = optional_timestamptz(record.system_to.as_ref()),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render SQL that releases one active legal hold under the session tenant.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidRetentionLifecycle`] when the hold
/// identity is nil (caller must pass a real open hold id).
pub fn release_legal_hold_sql(
    legal_hold_id: Uuid,
    release_system_time: &SystemTime,
) -> Result<String, PersistenceError> {
    if legal_hold_id.is_nil() {
        return Err(PersistenceError::InvalidRetentionLifecycle);
    }
    Ok(format!(
        "SELECT release_legal_hold('{hold}'::uuid, '{release}'::timestamptz)",
        hold = legal_hold_id,
        release = release_system_time.to_rfc3339(),
    ))
}

/// Render insert SQL for a validated deletion request that is not a completion.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidRetentionLifecycle`] when the record is
/// invalid.
pub fn insert_deletion_request_sql(
    record: &DeletionRequestRecord,
) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(render_deletion_request_sql(record))
}

/// Render insert SQL for a completed deletion after legal-hold evaluation.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidRetentionLifecycle`] when the record is
/// invalid and [`PersistenceError::LegalHoldBlocksDeletion`] when an active
/// hold covers the target.
pub fn insert_completed_deletion_request_sql(
    record: &DeletionRequestRecord,
    holds: &[LegalHoldRecord],
) -> Result<String, PersistenceError> {
    record.validate()?;
    if holds
        .iter()
        .any(|hold| hold.blocks_deletion(record.tenant_record_id, record.target_document_id))
    {
        return Err(PersistenceError::LegalHoldBlocksDeletion);
    }
    Ok(render_deletion_request_sql(record))
}

/// Render insert SQL for a validated evidence tombstone.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidRetentionLifecycle`] or
/// [`PersistenceError::UngovernedEvidenceRestore`] before any SQL is produced.
pub fn insert_evidence_tombstone_sql(
    record: &EvidenceTombstoneRecord,
) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO evidence_tombstone (\
            evidence_tombstone_id, tenant_record_id, tombstoned_document_id, \
            deletion_request_id, evidence_digest, target_data_class_code, \
            deletion_kind_code, reproduction_status_code, system_time, available_time\
        ) VALUES (\
            '{tombstone}'::uuid, '{tenant}'::uuid, '{document}'::uuid, \
            '{request}'::uuid, '{digest}', '{class}', \
            '{kind}', '{reproduction}', '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        tombstone = record.evidence_tombstone_id,
        tenant = record.tenant_record_id,
        document = record.tombstoned_document_id,
        request = record.deletion_request_id,
        digest = record.evidence_digest,
        class = record.target_data_class_code,
        kind = record.deletion_kind_code,
        reproduction = record.reproduction_status_code,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render selection SQL that excludes revoked or identity-tombstoned documents.
///
/// A `cache_export_removal` tombstone or completed request does not drop the
/// document from active analysis; only `logical_revocation` and
/// `identity_tombstone` kinds remove analysis eligibility.
#[must_use]
pub fn select_active_analysis_document_sql(document_record_id: Uuid) -> String {
    format!(
        "SELECT document_record_id FROM document_record \
         WHERE document_record_id = '{document_record_id}'::uuid \
           AND system_to IS NULL \
           AND NOT EXISTS (\
                SELECT 1 FROM evidence_tombstone \
                WHERE tombstoned_document_id = document_record.document_record_id \
                  AND deletion_kind_code IN ('logical_revocation', 'identity_tombstone')\
           ) \
           AND NOT EXISTS (\
                SELECT 1 FROM deletion_request \
                WHERE target_document_id = document_record.document_record_id \
                  AND request_status_code = 'completed' \
                  AND deletion_kind_code IN ('logical_revocation', 'identity_tombstone')\
           )"
    )
}

/// Map a lifecycle SQL failure message onto a typed persistence error.
///
/// Trigger text `legal hold blocks deletion` is
/// [`PersistenceError::LegalHoldBlocksDeletion`]. Other messages stay
/// unmapped so the transport can fail closed as a generic execution error.
#[must_use]
pub fn classify_lifecycle_sql_failure(message: &str) -> Option<PersistenceError> {
    if message.contains("legal hold blocks deletion") {
        Some(PersistenceError::LegalHoldBlocksDeletion)
    } else {
        None
    }
}

fn render_deletion_request_sql(record: &DeletionRequestRecord) -> String {
    format!(
        "INSERT INTO deletion_request (\
            deletion_request_id, tenant_record_id, retention_policy_id, \
            target_document_id, target_data_class_code, processing_purpose_code, \
            deletion_kind_code, request_status_code, legal_hold_id, \
            system_time, available_time\
        ) VALUES (\
            '{request}'::uuid, '{tenant}'::uuid, '{policy}'::uuid, \
            '{document}'::uuid, '{class}', '{purpose}', \
            '{kind}', '{status}', {hold}, \
            '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        request = record.deletion_request_id,
        tenant = record.tenant_record_id,
        policy = record.retention_policy_id,
        document = record.target_document_id,
        class = record.target_data_class_code,
        purpose = record.processing_purpose_code,
        kind = record.deletion_kind_code,
        status = record.request_status_code,
        hold = optional_uuid(record.legal_hold_id),
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    )
}

fn optional_timestamptz(value: Option<&SystemTime>) -> String {
    match value {
        Some(time) => format!("'{}'::timestamptz", time.to_rfc3339()),
        None => "NULL".into(),
    }
}

fn optional_uuid(value: Option<Uuid>) -> String {
    match value {
        Some(id) => format!("'{id}'::uuid"),
        None => "NULL".to_owned(),
    }
}

fn validate_vocab(value: &str, allowed: &[&str]) -> Result<(), PersistenceError> {
    validate_lifecycle_label(value)?;
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(PersistenceError::InvalidRetentionLifecycle)
    }
}

fn validate_lifecycle_label(value: &str) -> Result<(), PersistenceError> {
    if value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_control() || ch == '\'' || ch == ';' || ch == '\\')
    {
        return Err(PersistenceError::InvalidRetentionLifecycle);
    }
    Ok(())
}

fn validate_sha256_hex(value: &str) -> Result<(), PersistenceError> {
    let valid_len = value.len() == 64;
    let valid_hex = value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid_len && valid_hex {
        Ok(())
    } else {
        Err(PersistenceError::InvalidRetentionLifecycle)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DeletionRequestRecord, EvidenceTombstoneRecord, LegalHoldRecord, RetentionPolicyRecord,
        classify_lifecycle_sql_failure, insert_completed_deletion_request_sql,
        insert_deletion_request_sql, insert_evidence_tombstone_sql, insert_legal_hold_sql,
        insert_retention_policy_sql, release_legal_hold_sql, select_active_analysis_document_sql,
        supersede_retention_policy_sql,
    };
    use crate::PersistenceError;
    use temporal_core::{AvailableTime, SystemTime};
    use uuid::Uuid;

    fn times() -> (AvailableTime, SystemTime) {
        (
            AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
            SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
        )
    }

    fn policy() -> RetentionPolicyRecord {
        let (available, system) = times();
        RetentionPolicyRecord {
            retention_policy_id: Uuid::nil(),
            tenant_record_id: Uuid::nil(),
            data_class_code: "raw_source".into(),
            processing_purpose_code: "psychometric_analysis".into(),
            retention_period_days: 365,
            policy_status_code: "active".into(),
            authority_citation: "adr-0009".into(),
            system_time: system,
            system_to: None,
            available_time: available,
        }
    }

    fn hold() -> LegalHoldRecord {
        let (available, system) = times();
        LegalHoldRecord {
            legal_hold_id: Uuid::from_u128(2),
            tenant_record_id: Uuid::nil(),
            hold_scope_code: "document".into(),
            held_document_id: Some(Uuid::from_u128(3)),
            hold_authority_code: "contract".into(),
            hold_status_code: "active".into(),
            authority_citation: "hold-authority".into(),
            system_time: system,
            system_to: None,
            available_time: available,
        }
    }

    fn request() -> DeletionRequestRecord {
        let (available, system) = times();
        DeletionRequestRecord {
            deletion_request_id: Uuid::from_u128(4),
            tenant_record_id: Uuid::nil(),
            retention_policy_id: Uuid::nil(),
            target_document_id: Uuid::from_u128(3),
            target_data_class_code: "raw_source".into(),
            processing_purpose_code: "psychometric_analysis".into(),
            deletion_kind_code: "identity_tombstone".into(),
            request_status_code: "completed".into(),
            legal_hold_id: None,
            system_time: system,
            available_time: available,
        }
    }

    fn tombstone() -> EvidenceTombstoneRecord {
        let (available, system) = times();
        EvidenceTombstoneRecord {
            evidence_tombstone_id: Uuid::from_u128(5),
            tenant_record_id: Uuid::nil(),
            tombstoned_document_id: Uuid::from_u128(3),
            deletion_request_id: Uuid::from_u128(4),
            evidence_digest: "ab".repeat(32),
            target_data_class_code: "raw_source".into(),
            deletion_kind_code: "identity_tombstone".into(),
            reproduction_status_code: "unavailable".into(),
            system_time: system,
            available_time: available,
        }
    }

    #[test]
    fn insert_sql_renders_policy_hold_request_and_tombstone() {
        let policy_sql = insert_retention_policy_sql(&policy()).expect("policy");
        assert!(policy_sql.contains("INSERT INTO retention_policy"));
        assert!(policy_sql.contains("retention_period_days"));

        let hold_sql = insert_legal_hold_sql(&hold()).expect("hold");
        assert!(hold_sql.contains("INSERT INTO legal_hold"));
        assert!(hold_sql.contains("held_document_id"));

        let mut requested = request();
        requested.request_status_code = "requested".into();
        let request_sql = insert_deletion_request_sql(&requested).expect("request");
        assert!(request_sql.contains("INSERT INTO deletion_request"));
        assert!(request_sql.contains("NULL"));

        let tombstone_sql = insert_evidence_tombstone_sql(&tombstone()).expect("tombstone");
        assert!(tombstone_sql.contains("INSERT INTO evidence_tombstone"));
        assert!(tombstone_sql.contains("reproduction_status_code"));
        assert!(!tombstone_sql.contains("content_sha256"));
    }

    #[test]
    fn succession_sql_calls_function_and_refuses_same_identity() {
        let predecessor = Uuid::from_u128(1);
        let sql = supersede_retention_policy_sql(predecessor, &policy()).expect("succession");
        assert!(sql.contains("SELECT supersede_retention_policy("));
        assert!(sql.contains(&predecessor.to_string()));

        let mut same = policy();
        same.retention_policy_id = predecessor;
        assert_eq!(
            supersede_retention_policy_sql(predecessor, &same),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut superseded = policy();
        superseded.policy_status_code = "superseded".into();
        superseded.system_to =
            Some(SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("closed"));
        assert_eq!(
            supersede_retention_policy_sql(predecessor, &superseded),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        assert!(insert_retention_policy_sql(&superseded).is_ok());
        assert!(
            insert_retention_policy_sql(&superseded)
                .expect("historical superseded")
                .contains("system_to")
        );

        let release = release_legal_hold_sql(
            Uuid::from_u128(2),
            &SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("release"),
        )
        .expect("release sql");
        assert!(release.contains("SELECT release_legal_hold("));
        assert_eq!(
            release_legal_hold_sql(
                Uuid::nil(),
                &SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("r")
            ),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
    }

    #[test]
    fn tenant_hold_renders_null_document_and_lookup_excludes_tombstones() {
        let mut tenant_hold = hold();
        tenant_hold.hold_scope_code = "tenant".into();
        tenant_hold.held_document_id = None;
        let sql = insert_legal_hold_sql(&tenant_hold).expect("tenant hold");
        assert!(sql.contains("NULL"));
        let lookup = select_active_analysis_document_sql(Uuid::from_u128(3));
        assert!(lookup.contains("evidence_tombstone"));
        assert!(lookup.contains("logical_revocation"));
        let tombstone_clause = lookup
            .split("FROM evidence_tombstone")
            .nth(1)
            .expect("tombstone exclusion");
        assert!(
            tombstone_clause
                .contains("deletion_kind_code IN ('logical_revocation', 'identity_tombstone')"),
            "cache_export_removal tombstones must stay analysis-eligible"
        );
    }

    #[test]
    fn deletion_request_must_match_cited_retention_policy() {
        assert_eq!(request().bind_cited_policy(&policy()), Ok(()));

        let mut mismatched_purpose = request();
        mismatched_purpose.processing_purpose_code = "export_fulfillment".into();
        assert_eq!(
            mismatched_purpose.bind_cited_policy(&policy()),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut mismatched_class = request();
        mismatched_class.target_data_class_code = "export_cache".into();
        assert_eq!(
            mismatched_class.bind_cited_policy(&policy()),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut mismatched_tenant = request();
        mismatched_tenant.tenant_record_id = Uuid::from_u128(9);
        assert_eq!(
            mismatched_tenant.bind_cited_policy(&policy()),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut mismatched_policy_id = request();
        mismatched_policy_id.retention_policy_id = Uuid::from_u128(11);
        assert_eq!(
            mismatched_policy_id.bind_cited_policy(&policy()),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
    }

    #[test]
    fn lifecycle_sql_message_maps_legal_hold_block() {
        assert_eq!(
            classify_lifecycle_sql_failure("ERROR: legal hold blocks deletion"),
            Some(PersistenceError::LegalHoldBlocksDeletion)
        );
        assert_eq!(classify_lifecycle_sql_failure("sql execution failed"), None);
    }

    #[test]
    fn status_window_pair_and_inverted_system_bounds_fail_closed() {
        let closed =
            SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("closed system bound");
        let before =
            SystemTime::parse_rfc3339("2025-01-01T00:00:00Z").expect("before system bound");

        let mut active_closed = policy();
        active_closed.system_to = Some(closed);
        assert_eq!(
            insert_retention_policy_sql(&active_closed),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut superseded_open = policy();
        superseded_open.policy_status_code = "superseded".into();
        assert_eq!(
            insert_retention_policy_sql(&superseded_open),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut inverted = policy();
        inverted.policy_status_code = "superseded".into();
        inverted.system_to = Some(before);
        assert_eq!(
            insert_retention_policy_sql(&inverted),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut hold_active_closed = hold();
        hold_active_closed.system_to = Some(closed);
        assert_eq!(
            insert_legal_hold_sql(&hold_active_closed),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        assert!(
            !hold_active_closed.blocks_deletion(Uuid::nil(), Uuid::from_u128(3)),
            "active status with a closed system_to must not block deletion"
        );

        let mut hold_released_open = hold();
        hold_released_open.hold_status_code = "released".into();
        assert_eq!(
            insert_legal_hold_sql(&hold_released_open),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut hold_inverted = hold();
        hold_inverted.hold_status_code = "released".into();
        hold_inverted.system_to = Some(before);
        assert_eq!(
            insert_legal_hold_sql(&hold_inverted),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut released_closed = hold();
        released_closed.hold_status_code = "released".into();
        released_closed.system_to = Some(closed);
        insert_legal_hold_sql(&released_closed).expect("historical released hold");
        assert!(
            !released_closed.blocks_deletion(Uuid::nil(), Uuid::from_u128(3)),
            "released/closed holds must not block deletion"
        );
    }

    #[test]
    fn non_positive_period_unknown_vocab_and_hostile_labels_fail_closed() {
        let mut zero = policy();
        zero.retention_period_days = 0;
        assert_eq!(
            insert_retention_policy_sql(&zero),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        zero.retention_period_days = -1;
        assert_eq!(
            insert_retention_policy_sql(&zero),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut unknown = policy();
        unknown.data_class_code = "notes".into();
        assert_eq!(
            insert_retention_policy_sql(&unknown),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut hostile = policy();
        hostile.authority_citation = "adr'; DROP TABLE".into();
        assert_eq!(
            insert_retention_policy_sql(&hostile),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        hostile.authority_citation.clear();
        assert_eq!(
            insert_retention_policy_sql(&hostile),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        hostile.authority_citation = "adr\\cite".into();
        assert_eq!(
            insert_retention_policy_sql(&hostile),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        hostile.authority_citation = "adr\ncite".into();
        assert_eq!(
            insert_retention_policy_sql(&hostile),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
    }

    #[test]
    fn hold_scope_mismatch_and_unknown_codes_fail_closed() {
        let mut missing_document = hold();
        missing_document.held_document_id = None;
        assert_eq!(
            insert_legal_hold_sql(&missing_document),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut tenant_with_document = hold();
        tenant_with_document.hold_scope_code = "tenant".into();
        assert_eq!(
            insert_legal_hold_sql(&tenant_with_document),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut unknown_scope = hold();
        unknown_scope.hold_scope_code = "project".into();
        assert_eq!(
            insert_legal_hold_sql(&unknown_scope),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        unknown_scope.hold_scope_code = "document".into();
        unknown_scope.hold_status_code = "pending".into();
        assert_eq!(
            insert_legal_hold_sql(&unknown_scope),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        unknown_scope.hold_status_code = "active".into();
        unknown_scope.hold_authority_code = "court;hold".into();
        assert_eq!(
            insert_legal_hold_sql(&unknown_scope),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
    }

    #[test]
    fn deletion_request_unknown_codes_fail_closed() {
        let mut bad = request();
        bad.deletion_kind_code = "hard_delete".into();
        assert_eq!(
            insert_deletion_request_sql(&bad),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        bad.deletion_kind_code = "identity_tombstone".into();
        bad.request_status_code = "approved".into();
        assert_eq!(
            insert_deletion_request_sql(&bad),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        bad.request_status_code = "completed".into();
        bad.processing_purpose_code = "curiosity".into();
        assert_eq!(
            insert_deletion_request_sql(&bad),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
    }

    #[test]
    fn completed_deletion_fails_closed_under_matching_hold() {
        assert_eq!(
            insert_completed_deletion_request_sql(&request(), &[hold()]),
            Err(PersistenceError::LegalHoldBlocksDeletion)
        );

        let mut tenant_hold = hold();
        tenant_hold.hold_scope_code = "tenant".into();
        tenant_hold.held_document_id = None;
        assert_eq!(
            insert_completed_deletion_request_sql(&request(), &[tenant_hold]),
            Err(PersistenceError::LegalHoldBlocksDeletion)
        );

        let mut released = hold();
        released.hold_status_code = "released".into();
        insert_completed_deletion_request_sql(&request(), &[released]).expect("released");

        let mut other_document = hold();
        other_document.held_document_id = Some(Uuid::from_u128(99));
        insert_completed_deletion_request_sql(&request(), &[other_document]).expect("other doc");

        let mut other_tenant = hold();
        other_tenant.tenant_record_id = Uuid::from_u128(8);
        insert_completed_deletion_request_sql(&request(), &[other_tenant]).expect("other tenant");

        insert_completed_deletion_request_sql(&request(), &[]).expect("no holds");
    }

    #[test]
    fn tombstone_refuses_invalid_digest_and_reproduction_override() {
        let mut digest = tombstone();
        digest.evidence_digest = "zz".repeat(32);
        assert_eq!(
            insert_evidence_tombstone_sql(&digest),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        digest.evidence_digest = "ab".repeat(16);
        assert_eq!(
            insert_evidence_tombstone_sql(&digest),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        // 64-char non-hex and digit-only paths exercise every SHA-256 arm.
        digest.evidence_digest = format!("{}G", "a".repeat(63));
        assert_eq!(
            insert_evidence_tombstone_sql(&digest),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
        digest.evidence_digest = "0".repeat(64);
        insert_evidence_tombstone_sql(&digest).expect("digit digest");

        let mut override_status = tombstone();
        override_status.reproduction_status_code = "unaffected".into();
        assert_eq!(
            insert_evidence_tombstone_sql(&override_status),
            Err(PersistenceError::UngovernedEvidenceRestore)
        );
        override_status.deletion_kind_code = "logical_revocation".into();
        assert_eq!(
            insert_evidence_tombstone_sql(&override_status),
            Err(PersistenceError::UngovernedEvidenceRestore)
        );

        let mut cache = tombstone();
        cache.deletion_kind_code = "cache_export_removal".into();
        cache.reproduction_status_code = "unaffected".into();
        insert_evidence_tombstone_sql(&cache).expect("cache removal");

        let mut derived = tombstone();
        derived.target_data_class_code = "derived_result".into();
        derived.reproduction_status_code = "limited".into();
        insert_evidence_tombstone_sql(&derived).expect("derived limited");
    }
}
