//! Replay contract for every well-formed elevated re-identification denial path.

use tepp_api::{
    AnalyticalPurpose, ApiError, IdentityMappingRecord, PurposeGrant,
    ReidentificationAuditOutcome, ReidentificationAuditRecord, ReidentificationAuditSink,
    disclose_identity_mapping,
};

const DECISION_DIGEST: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[derive(Default)]
struct RecordingAuditSink {
    records: Vec<ReidentificationAuditRecord>,
}

impl ReidentificationAuditSink for RecordingAuditSink {
    fn append_reidentification_audit(
        &mut self,
        record: &ReidentificationAuditRecord,
    ) -> Result<(), ApiError> {
        self.records.push(record.clone());
        Ok(())
    }
}

fn grant() -> PurposeGrant {
    PurposeGrant {
        tenant_workspace_id: "tenant-workspace".into(),
        principal_id: "principal-analyst".into(),
        purpose: AnalyticalPurpose::ScientificValidation,
        valid_from: "2026-01-01T00:00:00Z".into(),
        valid_to: Some("2026-12-31T23:59:59Z".into()),
        reidentification_authorized: true,
    }
}

fn mapping() -> IdentityMappingRecord {
    IdentityMappingRecord {
        tenant_workspace_id: "tenant-workspace".into(),
        opaque_analytical_id: "opaque-person-42".into(),
        direct_identity: "Pat Lee <pat.lee@example.test>".into(),
    }
}

fn assert_denied_and_audited(
    grant: &PurposeGrant,
    mapping: &IdentityMappingRecord,
    decision_time: &str,
    sink: &mut RecordingAuditSink,
) {
    assert_eq!(
        disclose_identity_mapping(grant, mapping, decision_time, DECISION_DIGEST, sink),
        Err(ApiError::AuthorizationDenied)
    );
    let record = sink.records.last().expect("denial audit record");
    assert_eq!(record.outcome(), ReidentificationAuditOutcome::Denied);
    assert_eq!(record.decision_digest(), DECISION_DIGEST);
    assert!(!format!("{record:?}").contains("Pat Lee"));
    assert!(!format!("{record:?}").contains("pat.lee@example.test"));
}

#[test]
fn all_well_formed_denial_paths_append_replayable_redacted_records() {
    let mut sink = RecordingAuditSink::default();

    let expired = grant();
    assert_denied_and_audited(
        &expired,
        &mapping(),
        "2027-01-01T00:00:00Z",
        &mut sink,
    );

    let not_yet_valid = grant();
    assert_denied_and_audited(
        &not_yet_valid,
        &mapping(),
        "2025-12-31T23:59:59Z",
        &mut sink,
    );

    let mut cross_tenant = mapping();
    cross_tenant.tenant_workspace_id = "other-tenant".into();
    assert_denied_and_audited(
        &grant(),
        &cross_tenant,
        "2026-06-15T12:00:00Z",
        &mut sink,
    );

    let mut wrong_purpose = grant();
    wrong_purpose.purpose = AnalyticalPurpose::PartnerDisclosure;
    assert_denied_and_audited(
        &wrong_purpose,
        &mapping(),
        "2026-06-15T12:00:00Z",
        &mut sink,
    );

    let mut missing_elevation = grant();
    missing_elevation.reidentification_authorized = false;
    assert_denied_and_audited(
        &missing_elevation,
        &mapping(),
        "2026-06-15T12:00:00Z",
        &mut sink,
    );

    assert_eq!(sink.records.len(), 5);
    assert!(sink
        .records
        .iter()
        .all(|record| record.outcome() == ReidentificationAuditOutcome::Denied));
}
