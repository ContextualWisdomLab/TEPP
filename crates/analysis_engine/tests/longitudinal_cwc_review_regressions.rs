//! Regression contracts for longitudinal CWC analysis-run integrity.

use analysis_engine::{
    AnalysisEngineError, LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION,
    LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION, LONGITUDINAL_CWC_OUTPUT_PROFILE,
    LongitudinalClusterScore, LongitudinalCwcArtifact, MAX_EVIDENCE_UNITS,
    execute_longitudinal_cwc_run,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

const SNAPSHOT_ID: &str = "snapshot-longitudinal-cwc";

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request(cutoff: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "longitudinal-cwc-review-regression".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: SNAPSHOT_ID.into(),
        knowledge_cutoff: cutoff.into(),
        model_contract_version: LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION.into(),
        output_profile: LONGITUDINAL_CWC_OUTPUT_PROFILE.into(),
    }
}

fn row(
    snapshot_id: &str,
    cluster_key: u64,
    predictor: f64,
    outcome: f64,
    available_time: &str,
) -> LongitudinalClusterScore {
    let evidence_id = format!(
        "evidence-{snapshot_id}-{cluster_key}-{:016x}-{:016x}-{available_time}",
        predictor.to_bits(),
        outcome.to_bits()
    );
    LongitudinalClusterScore::new(
        evidence_id,
        snapshot_id,
        cluster_key,
        predictor,
        outcome,
        available(available_time),
    )
    .expect("row")
}

fn rows() -> Vec<LongitudinalClusterScore> {
    vec![
        row(SNAPSHOT_ID, 1, 0.0, 2.0, "2026-07-01T00:00:00Z"),
        row(SNAPSHOT_ID, 1, 2.0, 3.0, "2026-07-01T00:00:00Z"),
        row(SNAPSHOT_ID, 2, 4.0, 10.0, "2026-07-01T00:00:00Z"),
        row(SNAPSHOT_ID, 2, 6.0, 11.0, "2026-07-01T00:00:00Z"),
    ]
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-longitudinal-cwc",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
    scores: &[LongitudinalClusterScore],
) -> analysis_engine::LongitudinalCwcExecution {
    execute_longitudinal_cwc_run(
        request,
        &accepted(request),
        SNAPSHOT_ID,
        cutoff(),
        scores,
        "2026-08-02T00:00:00Z",
    )
    .expect("execution")
}

fn artifact() -> LongitudinalCwcArtifact {
    LongitudinalCwcArtifact {
        schema_version: LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-longitudinal-cwc".into(),
        snapshot_id: SNAPSHOT_ID.into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        row_count: 4,
        cluster_count: 2,
        within_slope: 0.5,
        between_slope: 2.0,
        contextual_effect: 1.5,
        inference_status: "composed_cwc_slopes_not_causal".into(),
    }
}

#[test]
fn equivalent_cutoff_instants_bind_and_provider_status_stays_separate() {
    let request = request("2026-08-01T01:00:00+01:00");
    let accepted = accepted(&request);

    let execution = execute_longitudinal_cwc_run(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        &rows(),
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent cutoff instant must be accepted");

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
        execution.artifact.inference_status,
        "composed_cwc_slopes_not_causal"
    );
}

#[test]
fn visible_cross_snapshot_rows_fail_closed_before_scientific_composition() {
    let request = request("2026-08-01T00:00:00Z");
    let accepted = accepted(&request);
    let mut mixed = rows();
    mixed.push(row(
        "snapshot-other",
        3,
        8.0,
        20.0,
        "2026-07-15T00:00:00Z",
    ));

    assert_eq!(
        execute_longitudinal_cwc_run(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            &mixed,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
}

#[test]
fn future_unavailable_cross_snapshot_rows_do_not_change_historical_replay() {
    let request = request("2026-08-01T00:00:00Z");
    let baseline_rows = rows();
    let baseline = execute(&request, &baseline_rows);
    let mut replay_rows = baseline_rows;
    replay_rows.push(row(
        "snapshot-other",
        3,
        8.0,
        20.0,
        "2026-08-15T00:00:00Z",
    ));
    let replay = execute(&request, &replay_rows);

    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result, baseline.terminal_result);
}

#[test]
fn artifact_digest_commits_to_cutoff_visible_evidence_identity() {
    let request = request("2026-08-01T00:00:00Z");
    let baseline_rows = rows();
    let baseline = execute(&request, &baseline_rows);
    let mut substituted_rows = rows();
    substituted_rows[0] = LongitudinalClusterScore::new(
        "evidence-substituted-visible-row",
        SNAPSHOT_ID,
        1,
        0.0,
        2.0,
        available("2026-07-01T00:00:00Z"),
    )
    .expect("substituted row");
    let substituted = execute(&request, &substituted_rows);

    assert_eq!(substituted.artifact.row_count, baseline.artifact.row_count);
    assert_eq!(
        substituted.artifact.cluster_count,
        baseline.artifact.cluster_count
    );
    assert_eq!(substituted.artifact.within_slope, baseline.artifact.within_slope);
    assert_eq!(
        substituted.artifact.between_slope,
        baseline.artifact.between_slope
    );
    assert_eq!(
        substituted.artifact.contextual_effect,
        baseline.artifact.contextual_effect
    );
    assert_ne!(
        substituted.artifact.sha256().expect("substituted digest"),
        baseline.artifact.sha256().expect("baseline digest"),
        "digest-bound CWC artifact did not commit to the admitted evidence identity"
    );
    assert_ne!(
        substituted.terminal_result, baseline.terminal_result,
        "terminal result did not distinguish a different admitted evidence population"
    );
}

#[test]
fn artifact_refuses_inconsistent_contextual_effect() {
    let mut tampered = artifact();
    tampered.contextual_effect = 0.0;
    assert_eq!(
        tampered.to_json(),
        Err(AnalysisEngineError::InvalidLongitudinalCwcArtifact)
    );
}

#[test]
fn artifact_refuses_visible_count_impossible_for_the_executor() {
    let mut oversized = artifact();
    oversized.row_count = u64::try_from(MAX_EVIDENCE_UNITS).expect("limit") + 1;
    oversized.cluster_count = 2;
    assert_eq!(
        oversized.to_json(),
        Err(AnalysisEngineError::InvalidLongitudinalCwcArtifact)
    );
}

#[test]
fn artifact_refuses_all_singleton_cluster_success_shape() {
    let mut impossible = artifact();
    impossible.row_count = 2;
    impossible.cluster_count = 2;

    assert_eq!(
        impossible.to_json(),
        Err(AnalysisEngineError::InvalidLongitudinalCwcArtifact)
    );
}
