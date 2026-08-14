//! Elevated re-identification must append redacted audit evidence for every decision.

use tepp_api::{
    AnalyticalPurpose, ApiError, IdentityMappingRecord, PurposeGrant,
    ReidentificationAuditOutcome, ReidentificationAuditRecord, ReidentificationAuditSink,
    disclose_identity_mapping,
};

#[derive(Default)]
struct RecordingAuditSink {
    records: Vec<ReidentificationAuditRecord>,
    fail_closed: bool,
}

impl ReidentificationAuditSink for RecordingAuditSink {
    fn append_reidentification_audit(
        &mut self,
        record: &ReidentificationAuditRecord,
    ) -> Result<(), ApiError> {
        if self.fail_closed {
            return Err(ApiError::LimitExceeded);
        }
        self.records.push(record.clone());
        Ok(())
    }
}

fn grant(reidentification_authorized: bool) -> PurposeGrant {
    PurposeGrant {
        tenant_workspace_id: "tenant-workspace".into(),
        principal_id: "principal-analyst".into(),
        purpose: AnalyticalPurpose::ScientificValidation,
        valid_from: "2026-01-01T00:00:00Z".into(),
        valid_to: Some("2026-12-31T23:59:59Z".into()),
        reidentification_authorized,
    }
}

fn mapping() -> IdentityMappingRecord {
    IdentityMappingRecord {
        tenant_workspace_id: "tenant-workspace".into(),
        opaque_analytical_id: "opaque-person-42".into(),
        direct_identity: "Pat Lee <pat.lee@example.test>".into(),
    }
}

#[test]
fn successful_reidentification_appends_redacted_audit_evidence_before_disclosure() {
    let mut sink = RecordingAuditSink::default();
    let (disclosed, audit) = disclose_identity_mapping(
        &grant(true),
        &mapping(),
        "2026-06-15T12:00:00Z",
        &mut sink,
    )
    .expect("audited elevated disclosure");

    assert_eq!(disclosed.direct_identity(), "Pat Lee <pat.lee@example.test>");
    assert_eq!(sink.records, vec![audit.clone()]);
    assert_eq!(audit.tenant_workspace_id(), "tenant-workspace");
    assert_eq!(audit.principal_id(), "principal-analyst");
    assert_eq!(audit.purpose_wire_name(), "scientific_validation");
    assert_eq!(audit.action_code(), "reidentify_identity_mapping");
    assert_eq!(audit.opaque_analytical_id(), "opaque-person-42");
    assert_eq!(audit.decision_time(), "2026-06-15T12:00:00Z");
    assert_eq!(audit.outcome(), ReidentificationAuditOutcome::Allowed);
    assert_eq!(audit.outcome().wire_name(), "allowed");
    assert!(audit.decision_digest().starts_with("sha256:"));
    assert_eq!(audit.decision_digest().len(), 71);
    assert!(!format!("{audit:?}").contains("Pat Lee"));
    assert!(!format!("{audit:?}").contains("pat.lee@example.test"));
}

#[test]
fn denied_reidentification_is_appended_and_replay_preserves_decision_order() {
    let mut sink = RecordingAuditSink::default();
    assert_eq!(
        disclose_identity_mapping(
            &grant(false),
            &mapping(),
            "2026-06-15T12:00:00Z",
            &mut sink,
        ),
        Err(ApiError::AuthorizationDenied),
    );
    let (_, allowed) = disclose_identity_mapping(
        &grant(true),
        &mapping(),
        "2026-06-15T12:00:01Z",
        &mut sink,
    )
    .expect("second audited decision");

    assert_eq!(sink.records.len(), 2);
    assert_eq!(
        sink.records[0].outcome(),
        ReidentificationAuditOutcome::Denied
    );
    assert_eq!(sink.records[0].outcome().wire_name(), "denied");
    assert_eq!(sink.records[1], allowed);
    assert_eq!(
        sink.records[1].outcome(),
        ReidentificationAuditOutcome::Allowed
    );
    assert_ne!(
        sink.records[0].decision_digest(),
        sink.records[1].decision_digest(),
        "allow and deny outcomes must be bound into distinct audit evidence"
    );
}

#[test]
fn disclosure_fails_closed_when_audit_append_fails() {
    let mut failed_sink = RecordingAuditSink {
        fail_closed: true,
        ..RecordingAuditSink::default()
    };
    assert_eq!(
        disclose_identity_mapping(
            &grant(true),
            &mapping(),
            "2026-06-15T12:00:00Z",
            &mut failed_sink,
        ),
        Err(ApiError::LimitExceeded),
    );
}

#[test]
fn audit_digest_is_deterministic_and_binds_protected_mapping_content() {
    let mut first_sink = RecordingAuditSink::default();
    let (_, first) = disclose_identity_mapping(
        &grant(true),
        &mapping(),
        "2026-06-15T12:00:00Z",
        &mut first_sink,
    )
    .expect("first disclosure");

    let mut repeat_sink = RecordingAuditSink::default();
    let (_, repeated) = disclose_identity_mapping(
        &grant(true),
        &mapping(),
        "2026-06-15T12:00:00Z",
        &mut repeat_sink,
    )
    .expect("repeat disclosure");
    assert_eq!(first.decision_digest(), repeated.decision_digest());

    let mut changed_mapping = mapping();
    changed_mapping.direct_identity = "Different Person".into();
    let mut changed_sink = RecordingAuditSink::default();
    let (_, changed) = disclose_identity_mapping(
        &grant(true),
        &changed_mapping,
        "2026-06-15T12:00:00Z",
        &mut changed_sink,
    )
    .expect("changed mapping disclosure");
    assert_ne!(first.decision_digest(), changed.decision_digest());
    assert!(!first.decision_digest().contains("Pat"));
    assert!(!changed.decision_digest().contains("Different"));
}
