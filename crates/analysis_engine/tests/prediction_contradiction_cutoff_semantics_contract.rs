//! Regression contract for semantic knowledge-cutoff equality in prediction-contradiction runs.

use analysis_engine::{
    PREDICTION_CONTRADICTION_MODEL_CONTRACT_VERSION, PREDICTION_CONTRADICTION_OUTPUT_PROFILE,
    PredictionContradictionAssignment, execute_prediction_contradiction_run,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available() -> AvailableTime {
    AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available time")
}

fn event_at(second: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-01T00:00:{second:02}Z")).expect("event time")
}

fn closed(start: u8, end: u8) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(event_at(start)),
        TemporalBoundary::Included(event_at(end)),
        TemporalPrecision::Second,
    )
    .expect("closed interval")
}

fn assignment(
    assignment_id: &str,
    predicted: TemporalInterval<EventTime>,
    observed: TemporalInterval<EventTime>,
) -> PredictionContradictionAssignment {
    PredictionContradictionAssignment::new(assignment_id, predicted, observed, available())
        .expect("assignment")
}

#[test]
fn equivalent_rfc3339_cutoff_offsets_are_admitted() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "prediction-contradiction-equivalent-cutoff".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-prediction-contradiction".into(),
        knowledge_cutoff: "2026-08-01T09:00:00+09:00".into(),
        model_contract_version: PREDICTION_CONTRADICTION_MODEL_CONTRACT_VERSION.into(),
        output_profile: PREDICTION_CONTRADICTION_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new(
        "run-prediction-contradiction",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted");
    let execution_cutoff =
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("execution cutoff");
    let assignments = vec![
        assignment("covered-a", closed(0, 8), closed(0, 10)),
        assignment("partial-b", closed(0, 10), closed(5, 15)),
        assignment("adjacent-c", closed(0, 10), closed(10, 20)),
        assignment("contradictory-d", closed(0, 10), closed(20, 30)),
    ];

    let execution = execute_prediction_contradiction_run(
        &request,
        &accepted,
        "snapshot-prediction-contradiction",
        execution_cutoff,
        &assignments,
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent RFC 3339 spellings denote the same cutoff instant");

    assert_eq!(
        execution.artifact.knowledge_cutoff,
        execution_cutoff.to_rfc3339()
    );
}
