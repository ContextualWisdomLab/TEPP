//! Regression contracts for subevent-containment analysis-run boundaries.

use analysis_engine::{
    SUBEVENT_CONTAINMENT_MODEL_CONTRACT_VERSION, SUBEVENT_CONTAINMENT_OUTPUT_PROFILE,
    SubeventContainmentAssignment, execute_subevent_containment_run,
};
use subevent_containment::EventInterval;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn interval(start: i64, end: i64) -> EventInterval {
    EventInterval::new(start, end).expect("interval")
}

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available time")
}

fn request(cutoff: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "subevent-containment-regression".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-subevent-containment".into(),
        knowledge_cutoff: cutoff.into(),
        model_contract_version: SUBEVENT_CONTAINMENT_MODEL_CONTRACT_VERSION.into(),
        output_profile: SUBEVENT_CONTAINMENT_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-subevent-containment", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn assignments() -> Vec<SubeventContainmentAssignment> {
    let parent = interval(10, 40);
    vec![
        SubeventContainmentAssignment::new(
            "contained",
            parent,
            interval(15, 30),
            available("2026-07-01T00:00:00Z"),
        )
        .expect("contained assignment"),
        SubeventContainmentAssignment::new(
            "escaped",
            parent,
            interval(0, 20),
            available("2026-07-02T00:00:00Z"),
        )
        .expect("escaped assignment"),
    ]
}

#[test]
fn equivalent_rfc3339_cutoff_spellings_bind_to_the_same_instant() {
    let request = request("2026-08-01T09:00:00+09:00");
    let execution = execute_subevent_containment_run(
        &request,
        &accepted(&request),
        "snapshot-subevent-containment",
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        &assignments(),
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent cutoff instants must bind");
    assert_eq!(execution.artifact.assignment_count, 2);
}

#[test]
fn terminal_summary_keeps_validation_state_separate_from_inference_status() {
    let request = request("2026-08-01T00:00:00Z");
    let execution = execute_subevent_containment_run(
        &request,
        &accepted(&request),
        "snapshot-subevent-containment",
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        &assignments(),
        "2026-08-02T00:00:00Z",
    )
    .expect("execution");
    let summary = execution
        .terminal_result
        .summary
        .as_ref()
        .expect("succeeded summary");
    assert_eq!(summary.validation_status, "validated");
    assert_eq!(
        execution.artifact.inference_status,
        "subevent_interval_cannot_escape_parent_interval"
    );
}
