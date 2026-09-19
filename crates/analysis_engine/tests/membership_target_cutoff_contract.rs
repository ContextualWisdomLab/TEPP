//! Regression contract for leakage-safe membership-target admission.

use analysis_engine::{
    MEMBERSHIP_TARGET_MODEL_CONTRACT_VERSION, MEMBERSHIP_TARGET_OUTPUT_PROFILE,
    MembershipTargetDocument, execute_membership_target_run,
};
use membership_target::MembershipTargetKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn document(
    document_id: &str,
    kind: MembershipTargetKind,
    available_at: &str,
) -> MembershipTargetDocument {
    MembershipTargetDocument::new(
        document_id,
        kind,
        AvailableTime::parse_rfc3339(available_at).expect("valid availability time"),
    )
    .expect("valid membership-target document")
}

#[test]
fn future_duplicate_identity_cannot_change_historical_cutoff_result() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "membership-target-cutoff-duplicate".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-membership-target-cutoff-duplicate".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: MEMBERSHIP_TARGET_MODEL_CONTRACT_VERSION.into(),
        output_profile: MEMBERSHIP_TARGET_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new(
        "run-membership-target-cutoff-duplicate",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted receipt");
    let cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff).expect("cutoff");

    let visible = vec![
        document(
            "lang-a",
            MembershipTargetKind::Language,
            "2026-07-01T00:00:00Z",
        ),
        document(
            "entity-b",
            MembershipTargetKind::Entity,
            "2026-07-02T00:00:00Z",
        ),
    ];
    let baseline = execute_membership_target_run(
        &request,
        &accepted,
        &request.snapshot_id,
        cutoff,
        &visible,
        "2026-08-02T00:00:00Z",
    )
    .expect("historical baseline");

    let mut with_future_duplicate = vec![document(
        "lang-a",
        MembershipTargetKind::Project,
        "2026-08-02T00:00:00Z",
    )];
    with_future_duplicate.extend(visible);
    let replay = execute_membership_target_run(
        &request,
        &accepted,
        &request.snapshot_id,
        cutoff,
        &with_future_duplicate,
        "2026-08-02T00:00:00Z",
    )
    .expect("future-unavailable duplicate must not enter cutoff admission");

    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result, baseline.terminal_result);
}
