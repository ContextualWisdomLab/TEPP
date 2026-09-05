//! End-to-end contract for cutoff-safe irregular event-time composition.

use analysis_engine::{
    AnalysisEngineError, IRREGULAR_EVENT_TIME_ARTIFACT_SCHEMA_VERSION,
    IRREGULAR_EVENT_TIME_MODEL_CONTRACT_VERSION, IRREGULAR_EVENT_TIME_OUTPUT_PROFILE,
    IrregularEventScore, MAX_EVIDENCE_UNITS, execute_irregular_event_time_run,
};
use psychometric_core::{LagClock, PsychometricError};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn noiseless_scores() -> Vec<IrregularEventScore> {
    vec![
        IrregularEventScore::new(0.0, 1.0, available("2026-07-01T00:00:00Z")).expect("t0"),
        IrregularEventScore::new(1.0, 0.5, available("2026-07-01T00:00:00Z")).expect("t1"),
        IrregularEventScore::new(2.0, 0.25, available("2026-07-01T00:00:00Z")).expect("t2"),
    ]
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "irregular-event-time-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-irregular-event-time".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: IRREGULAR_EVENT_TIME_MODEL_CONTRACT_VERSION.into(),
        output_profile: IRREGULAR_EVENT_TIME_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-irregular-event-time",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
    scores: &[IrregularEventScore],
    clock: LagClock,
    pool: bool,
    reference_delta: f64,
) -> Result<analysis_engine::IrregularEventTimeExecution, AnalysisEngineError> {
    execute_irregular_event_time_run(
        request,
        &accepted(request),
        "snapshot-irregular-event-time",
        cutoff(),
        scores,
        clock,
        pool,
        reference_delta,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn noiseless_irregular_series_maps_through_local_log_rate() {
    let request = request();
    let scores = noiseless_scores();
    let execution = execute(&request, &scores, LagClock::EventTime, false, 2.0).expect("execution");

    assert_eq!(
        execution.artifact.schema_version,
        IRREGULAR_EVENT_TIME_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.occasion_count, 3);
    assert_eq!(execution.artifact.interval_count, 2);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 0);
    assert!((execution.artifact.mean_log_rate - 0.5_f64.ln()).abs() < 1e-12);
    assert!((execution.artifact.mapped_reference_lag - 0.25).abs() < 1e-12);
    assert!((execution.artifact.reference_delta - 2.0).abs() < f64::EPSILON);
    assert_eq!(
        execution.artifact.inference_status,
        "composed_interval_mapped_lags_not_dsem"
    );
    assert_eq!(
        execution.terminal_result.run_state,
        AnalysisRunTerminalState::Succeeded
    );
    assert_eq!(
        execution.terminal_result.result_sha256.as_deref(),
        Some(execution.artifact.sha256().expect("digest").as_str())
    );
    assert_eq!(
        execution.terminal_result.result_schema_version.as_deref(),
        Some(IRREGULAR_EVENT_TIME_ARTIFACT_SCHEMA_VERSION)
    );
    assert!((scores[0].event_time() - 0.0).abs() < f64::EPSILON);
    assert!((scores[0].score() - 1.0).abs() < f64::EPSILON);
    assert_eq!(
        scores[0].available_time(),
        available("2026-07-01T00:00:00Z")
    );
}

#[test]
fn execution_excludes_occasions_unavailable_at_the_request_cutoff() {
    let request = request();
    let mut scores = noiseless_scores();
    scores.push(
        IrregularEventScore::new(3.0, 0.125, available("2026-08-15T00:00:00Z")).expect("late"),
    );
    let execution = execute(&request, &scores, LagClock::EventTime, false, 2.0).expect("execution");
    assert_eq!(execution.artifact.occasion_count, 3);
    assert_eq!(execution.artifact.interval_count, 2);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 1);
    assert!((execution.artifact.mean_log_rate - 0.5_f64.ln()).abs() < 1e-12);
}

#[test]
fn execution_refuses_pooled_lags_across_unequal_intervals() {
    let request = request();
    let scores = noiseless_scores();
    assert_eq!(
        execute(&request, &scores, LagClock::EventTime, true, 2.0),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::UnequalIntervalPoolingForbidden
        ))
    );
}

#[test]
fn execution_refuses_non_event_clocks() {
    let request = request();
    let scores = noiseless_scores();
    assert_eq!(
        execute(&request, &scores, LagClock::SystemTime, false, 2.0),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::EventTimeRequired
        ))
    );
}

#[test]
fn execution_refuses_snapshot_profile_cutoff_and_reference_mismatch() {
    let request = request();
    let scores = noiseless_scores();
    assert_eq!(
        execute_irregular_event_time_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &scores,
            LagClock::EventTime,
            false,
            2.0,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    for invalid_request in [
        {
            let mut value = request.clone();
            value.knowledge_cutoff = "2026-08-02T00:00:00Z".into();
            value
        },
        {
            let mut value = request.clone();
            value.model_contract_version = "other-model".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "other-profile".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request, &scores, LagClock::EventTime, false, 2.0,),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    assert_eq!(
        execute(&request, &scores, LagClock::EventTime, false, 0.0),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_non_finite_scores_empty_eligibility_and_oversize() {
    assert_eq!(
        IrregularEventScore::new(f64::NAN, 1.0, available("2026-07-01T00:00:00Z")),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let request = request();
    let late_only = vec![
        IrregularEventScore::new(0.0, 1.0, available("2026-08-15T00:00:00Z")).expect("late-a"),
        IrregularEventScore::new(1.0, 0.5, available("2026-08-15T00:00:00Z")).expect("late-b"),
    ];
    assert_eq!(
        execute(&request, &late_only, LagClock::EventTime, false, 2.0),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput
        ))
    );
    let oversized = vec![
        IrregularEventScore::new(0.0, 1.0, available("2026-07-01T00:00:00Z"))
            .expect("pad");
        MAX_EVIDENCE_UNITS + 1
    ];
    assert_eq!(
        execute(&request, &oversized, LagClock::EventTime, false, 2.0),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
