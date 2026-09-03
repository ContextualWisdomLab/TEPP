//! End-to-end contract for cutoff-safe prediction-contradiction refusals.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION,
    PREDICTION_CONTRADICTION_MODEL_CONTRACT_VERSION, PREDICTION_CONTRADICTION_OUTPUT_PROFILE,
    PredictionContradictionArtifact, PredictionContradictionAssignment,
    execute_prediction_contradiction_run,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
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

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "prediction-contradiction-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-prediction-contradiction".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: PREDICTION_CONTRADICTION_MODEL_CONTRACT_VERSION.into(),
        output_profile: PREDICTION_CONTRADICTION_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-prediction-contradiction",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn assignment(
    assignment_id: &str,
    predicted: TemporalInterval<EventTime>,
    observed: TemporalInterval<EventTime>,
    stamp: &str,
) -> PredictionContradictionAssignment {
    PredictionContradictionAssignment::new(assignment_id, predicted, observed, available(stamp))
        .expect("assignment")
}

fn mixed_assignments() -> Vec<PredictionContradictionAssignment> {
    vec![
        assignment(
            "covered-a",
            closed(0, 8),
            closed(0, 10),
            "2026-07-01T00:00:00Z",
        ),
        assignment(
            "partial-b",
            closed(0, 10),
            closed(5, 15),
            "2026-07-02T00:00:00Z",
        ),
        assignment(
            "adjacent-c",
            closed(0, 10),
            closed(10, 20),
            "2026-07-03T00:00:00Z",
        ),
        assignment(
            "contradictory-d",
            closed(0, 10),
            closed(20, 30),
            "2026-07-04T00:00:00Z",
        ),
        assignment(
            "covered-e",
            closed(0, 10),
            closed(0, 10),
            "2026-07-05T00:00:00Z",
        ),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    assignments: &[PredictionContradictionAssignment],
) -> Result<analysis_engine::PredictionContradictionExecution, AnalysisEngineError> {
    execute_prediction_contradiction_run(
        request,
        &accepted(request),
        "snapshot-prediction-contradiction",
        cutoff(),
        assignments,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_kinds_emit_digest_bound_refusals_without_agreement_metric() {
    let request = request();
    let execution = execute(&request, &mixed_assignments()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.assignment_count, 5);
    assert_eq!(execution.artifact.covered_count, 2);
    assert_eq!(execution.artifact.partial_overlap_count, 1);
    assert_eq!(execution.artifact.adjacent_count, 1);
    assert_eq!(execution.artifact.contradictory_count, 1);
    assert_eq!(execution.artifact.refused_promotion_count, 3);
    assert_eq!(
        execution.artifact.inference_status,
        "unmatched_prediction_is_not_observed"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("contradiction_agreement_rate"));
    assert!(!payload.contains("identity_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
    assert!(!payload.contains("edge_kind_recovery_rate"));
    assert_eq!(
        execution.terminal_result.run_state,
        AnalysisRunTerminalState::Succeeded
    );
    assert_eq!(
        execution
            .terminal_result
            .summary
            .as_ref()
            .expect("summary")
            .validation_status,
        "validated"
    );
    assert_eq!(
        execution.terminal_result.result_sha256.as_deref(),
        Some(execution.artifact.sha256().expect("digest").as_str())
    );
    assert_eq!(
        execution.terminal_result.result_schema_version.as_deref(),
        Some(PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn compact_oversized_artifact_counts_fail_closed() {
    let assignment_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let artifact = PredictionContradictionArtifact {
        schema_version: PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        assignment_count,
        covered_count: 1,
        partial_overlap_count: 1,
        adjacent_count: 1,
        contradictory_count: assignment_count - 3,
        refused_promotion_count: assignment_count - 1,
        inference_status: "unmatched_prediction_is_not_observed".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");
    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidPredictionContradictionArtifact)
    );
    assert_eq!(
        PredictionContradictionArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidPredictionContradictionArtifact)
    );
}

#[test]
fn future_available_assignments_are_excluded() {
    let request = request();
    let mut with_future = mixed_assignments();
    with_future.push(assignment(
        "future-covered",
        closed(0, 8),
        closed(0, 10),
        "2026-08-02T00:00:00Z",
    ));
    let execution = execute(&request, &with_future).expect("cutoff");
    assert_eq!(execution.artifact.assignment_count, 5);
    assert_eq!(execution.artifact.covered_count, 2);
}

#[test]
fn future_duplicate_identity_cannot_change_a_historical_cutoff_result() {
    let request = request();
    let mut with_future_duplicate = mixed_assignments();
    with_future_duplicate.push(assignment(
        "covered-a",
        closed(0, 10),
        closed(20, 30),
        "2026-08-02T00:00:00Z",
    ));

    let execution = execute(&request, &with_future_duplicate)
        .expect("future-unavailable evidence must not affect the historical run");
    assert_eq!(execution.artifact.assignment_count, 5);
    assert_eq!(execution.artifact.covered_count, 2);
    assert_eq!(execution.artifact.contradictory_count, 1);
}

#[test]
fn empty_or_incomplete_kind_mix_and_duplicate_fail_closed() {
    let request = request();
    let stamp = "2026-07-01T00:00:00Z";
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let covered_only = vec![
        assignment("covered-a", closed(0, 8), closed(0, 10), stamp),
        assignment("covered-b", closed(2, 8), closed(0, 10), stamp),
        assignment("covered-c", closed(2, 10), closed(0, 10), stamp),
        assignment("covered-d", closed(0, 10), closed(0, 10), stamp),
    ];
    assert_eq!(
        execute(&request, &covered_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let missing_contradiction = vec![
        assignment("covered-a", closed(0, 8), closed(0, 10), stamp),
        assignment("partial-b", closed(0, 10), closed(5, 15), stamp),
        assignment("adjacent-c", closed(0, 10), closed(10, 20), stamp),
    ];
    assert_eq!(
        execute(&request, &missing_contradiction),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        assignment("same", closed(0, 8), closed(0, 10), stamp),
        assignment("same", closed(0, 10), closed(5, 15), stamp),
        assignment("adjacent-c", closed(0, 10), closed(10, 20), stamp),
        assignment("contradictory-d", closed(0, 10), closed(20, 30), stamp),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        PredictionContradictionAssignment::new("", closed(0, 8), closed(0, 10), available(stamp)),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_cutoff_mismatch_and_oversize() {
    let request = request();
    let assignments = mixed_assignments();
    assert_eq!(
        execute_prediction_contradiction_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &assignments,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let mut mismatched = request.clone();
    mismatched.knowledge_cutoff = "2026-07-01T00:00:00Z".into();
    assert_eq!(
        execute_prediction_contradiction_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-prediction-contradiction",
            cutoff(),
            &assignments,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    for profile in [
        "trsl_topic_lineage_v1",
        "fitted_candidate_k_v1",
        "pareto_candidate_k_v1",
        "joint_posterior_draws_v1",
        "method_effects_v1",
        "copy_identity_v1",
        "style_source_v1",
        "prompt_source_v1",
        "modality_source_v1",
        "corpus_background_v1",
        "citation_edge_v1",
        "copied_text_v1",
        "lineage_criterion_v1",
        "composed_fitted_lineage_v1",
        "case_deletion_refit_v1",
        "topic_activity_v1",
        "location_membership_v1",
        "topic_context_posterior_v1",
        "membership_posterior_icc_v1",
        "membership_target_v1",
        "outcome_order_v1",
        "relation_absence_v1",
        "subevent_containment_v1",
        "episode_membership_v1",
        "inferred_status_v1",
        "role_contradiction_v1",
        "retrospective_edge_v1",
        "summarizes_edge_v1",
        "support_edge_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_prediction_contradiction_run(
                &reused,
                &accepted(&reused),
                "snapshot-prediction-contradiction",
                cutoff(),
                &assignments,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    let oversized: Vec<PredictionContradictionAssignment> = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            assignment(
                &format!("assignment-{index}"),
                closed(0, 8),
                closed(0, 10),
                "2026-07-01T00:00:00Z",
            )
        })
        .collect();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
