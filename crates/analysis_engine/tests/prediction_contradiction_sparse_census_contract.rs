//! Regression contract: observed relation classes are data, not run prerequisites.
//!
//! A real prediction-versus-observation census can legitimately contain only
//! covered pairs or only refusals. Zero counts for absent Allen support classes
//! must remain observable evidence rather than making the whole run invalid.

use analysis_engine::{
    AnalysisEngineError, PREDICTION_CONTRADICTION_MODEL_CONTRACT_VERSION,
    PREDICTION_CONTRADICTION_OUTPUT_PROFILE, PredictionContradictionAssignment,
    execute_prediction_contradiction_run,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn available() -> AvailableTime {
    AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available")
}

fn event_at(second: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-01T00:00:{second:02}Z")).expect("event")
}

fn closed(start: u8, end: u8) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(event_at(start)),
        TemporalBoundary::Included(event_at(end)),
        TemporalPrecision::Second,
    )
    .expect("closed interval")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "prediction-contradiction-sparse-census".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-prediction-contradiction".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: PREDICTION_CONTRADICTION_MODEL_CONTRACT_VERSION.into(),
        output_profile: PREDICTION_CONTRADICTION_OUTPUT_PROFILE.into(),
    }
}

fn execute(
    assignments: &[PredictionContradictionAssignment],
) -> Result<analysis_engine::PredictionContradictionExecution, AnalysisEngineError> {
    let request = request();
    let accepted = AnalysisRunAccepted::new(
        "run-prediction-contradiction-sparse",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted");
    execute_prediction_contradiction_run(
        &request,
        &accepted,
        "snapshot-prediction-contradiction",
        cutoff(),
        assignments,
        "2026-08-02T00:00:00Z",
    )
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
fn covered_only_census_reports_zero_refusals_instead_of_rejecting_the_run() {
    let assignments = vec![
        assignment("covered-a", closed(0, 8), closed(0, 10)),
        assignment("covered-b", closed(2, 8), closed(0, 10)),
    ];

    let execution = execute(&assignments)
        .expect("absence of contradiction classes is valid observed evidence");
    assert_eq!(execution.artifact.assignment_count, 2);
    assert_eq!(execution.artifact.covered_count, 2);
    assert_eq!(execution.artifact.partial_overlap_count, 0);
    assert_eq!(execution.artifact.adjacent_count, 0);
    assert_eq!(execution.artifact.contradictory_count, 0);
    assert_eq!(execution.artifact.refused_promotion_count, 0);
}

#[test]
fn contradiction_only_census_reports_zero_coverage_instead_of_rejecting_the_run() {
    let assignments = vec![
        assignment("contradictory-a", closed(0, 10), closed(20, 30)),
        assignment("contradictory-b", closed(20, 30), closed(0, 10)),
    ];

    let execution = execute(&assignments)
        .expect("absence of covered pairs is valid observed evidence");
    assert_eq!(execution.artifact.assignment_count, 2);
    assert_eq!(execution.artifact.covered_count, 0);
    assert_eq!(execution.artifact.partial_overlap_count, 0);
    assert_eq!(execution.artifact.adjacent_count, 0);
    assert_eq!(execution.artifact.contradictory_count, 2);
    assert_eq!(execution.artifact.refused_promotion_count, 2);
}
