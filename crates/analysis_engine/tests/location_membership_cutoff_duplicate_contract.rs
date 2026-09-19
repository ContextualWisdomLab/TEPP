//! Historical replay contract for location-membership evidence admission.

use analysis_engine::{
    LOCATION_MEMBERSHIP_MODEL_CONTRACT_VERSION, LOCATION_MEMBERSHIP_OUTPUT_PROFILE,
    LocationMembershipDocument, execute_location_membership_run,
};
use location_membership::LocationKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("valid availability")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("valid cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "location-membership-cutoff-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-location-membership-cutoff".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: LOCATION_MEMBERSHIP_MODEL_CONTRACT_VERSION.into(),
        output_profile: LOCATION_MEMBERSHIP_OUTPUT_PROFILE.into(),
    }
}

fn document(
    document_id: &str,
    kind: LocationKind,
    available_at: &str,
) -> LocationMembershipDocument {
    LocationMembershipDocument::new(document_id, kind, available(available_at))
        .expect("valid document")
}

fn execute(
    documents: &[LocationMembershipDocument],
) -> analysis_engine::LocationMembershipExecution {
    let request = request();
    let accepted = AnalysisRunAccepted::new(
        "run-location-membership-cutoff",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted run");
    execute_location_membership_run(
        &request,
        &accepted,
        "snapshot-location-membership-cutoff",
        cutoff(),
        documents,
        "2026-08-02T00:00:00Z",
    )
    .expect("historical execution")
}

#[test]
fn future_duplicate_identity_cannot_change_historical_result() {
    let visible = vec![
        document(
            "loc-a",
            LocationKind::Location,
            "2026-07-01T00:00:00Z",
        ),
        document(
            "ent-b",
            LocationKind::EntityIdentity,
            "2026-07-02T00:00:00Z",
        ),
        document(
            "lang-c",
            LocationKind::LanguageChannel,
            "2026-07-03T00:00:00Z",
        ),
    ];
    let baseline = execute(&visible);

    let mut with_future_duplicate = vec![document(
        "loc-a",
        LocationKind::Location,
        "2026-08-02T00:00:00Z",
    )];
    with_future_duplicate.extend(visible);
    let replay = execute(&with_future_duplicate);

    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result, baseline.terminal_result);
}
