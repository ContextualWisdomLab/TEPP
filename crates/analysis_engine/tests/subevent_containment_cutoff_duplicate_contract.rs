//! Regression contract for cutoff-safe subevent-containment duplicate admission.

use analysis_engine::{
    SUBEVENT_CONTAINMENT_MODEL_CONTRACT_VERSION, SUBEVENT_CONTAINMENT_OUTPUT_PROFILE,
    SubeventContainmentAssignment, execute_subevent_containment_run,
};
use subevent_containment::EventInterval;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn interval(start: i64, end: i64) -> EventInterval {
    EventInterval::new(start, end).expect("valid event interval")
}

fn assignment(
    assignment_id: &str,
    parent: EventInterval,
    child: EventInterval,
    available_at: &str,
) -> SubeventContainmentAssignment {
    SubeventContainmentAssignment::new(
        assignment_id,
        parent,
        child,
        AvailableTime::parse_rfc3339(available_at).expect("valid availability time"),
    )
    .expect("valid assignment")
}

#[test]
fn future_duplicate_identity_cannot_change_historical_cutoff_result() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "subevent-containment-cutoff-duplicate".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-subevent-containment-cutoff-duplicate".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: SUBEVENT_CONTAINMENT_MODEL_CONTRACT_VERSION.into(),
        output_profile: SUBEVENT_CONTAINMENT_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new(
        "run-subevent-containment-cutoff-duplicate",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted receipt");
    let cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff).expect("cutoff");
    let parent = interval(10, 40);

    let visible = vec![
        assignment(
            "contained-a",
            parent,
            interval(15, 30),
            "2026-07-01T00:00:00Z",
        ),
        assignment("escaped-b", parent, interval(0, 20), "2026-07-02T00:00:00Z"),
    ];
    let baseline = execute_subevent_containment_run(
        &request,
        &accepted,
        &request.snapshot_id,
        cutoff,
        &visible,
        "2026-08-02T00:00:00Z",
    )
    .expect("historical baseline");

    let mut with_future_duplicate = vec![assignment(
        "contained-a",
        parent,
        interval(16, 17),
        "2026-08-02T00:00:00Z",
    )];
    with_future_duplicate.extend(visible);
    let replay = execute_subevent_containment_run(
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
