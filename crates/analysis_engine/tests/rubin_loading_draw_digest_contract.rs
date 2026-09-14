//! Canonical executor-owned complete-data draw-payload digest contract.

use analysis_engine::{
    AnalysisEngineError, RUBIN_LOADING_MODEL_CONTRACT_VERSION, RUBIN_LOADING_OUTPUT_PROFILE,
    RubinLoadingObservation, RubinLoadingUncertaintyArtifact, execute_rubin_loading_uncertainty_run,
};
use psychometric_core::IndicatorKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

const SNAPSHOT_ID: &str = "snapshot-rubin-draw-digest";

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("availability")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "rubin-draw-digest-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: SNAPSHOT_ID.into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: RUBIN_LOADING_MODEL_CONTRACT_VERSION.into(),
        output_profile: RUBIN_LOADING_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-rubin-draw-digest", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn observation(
    factor_score: f64,
    draws: Vec<f64>,
    available_at: &str,
) -> RubinLoadingObservation {
    RubinLoadingObservation::new(SNAPSHOT_ID, factor_score, draws, available(available_at))
        .expect("observation")
}

fn baseline_rows() -> Vec<RubinLoadingObservation> {
    vec![
        observation(-1.0, vec![-0.7, -0.9], "2026-07-01T00:00:00Z"),
        observation(0.0, vec![0.0, 0.0], "2026-07-01T00:00:00Z"),
        observation(1.0, vec![0.7, 0.9], "2026-07-01T00:00:00Z"),
    ]
}

fn execute(rows: &[RubinLoadingObservation]) -> analysis_engine::RubinLoadingUncertaintyExecution {
    let request = request();
    let accepted = accepted(&request);
    execute_rubin_loading_uncertainty_run(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        rows,
        "2026-08-02T00:00:00Z",
    )
    .expect("execution")
}

#[test]
fn canonical_draw_digest_is_cutoff_safe_and_value_sensitive() {
    let baseline = execute(&baseline_rows());
    let baseline_digest = baseline.artifact.complete_data_draws_sha256();
    assert_eq!(baseline_digest.len(), 64);
    assert!(baseline_digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));

    let mut with_late = baseline_rows();
    with_late.push(observation(
        2.0,
        vec![100.0, 200.0, 300.0],
        "2026-08-15T00:00:00Z",
    ));
    let replay = execute(&with_late);
    assert_eq!(
        replay.artifact.complete_data_draws_sha256(),
        baseline.artifact.complete_data_draws_sha256(),
        "future-unavailable rows must not change the historical admitted matrix digest"
    );

    let mut changed = baseline_rows();
    changed[2] = observation(1.0, vec![0.700_000_000_000_000_1, 0.9], "2026-07-01T00:00:00Z");
    let changed = execute(&changed);
    assert_ne!(
        changed.artifact.complete_data_draws_sha256(),
        baseline.artifact.complete_data_draws_sha256(),
        "one admitted draw-bit change must change payload identity"
    );
}

#[test]
fn artifact_import_rejects_malformed_draw_payload_digest() {
    let execution = execute(&baseline_rows());
    let canonical = execution.artifact.to_json().expect("artifact json");
    let mut value: serde_json::Value = serde_json::from_str(&canonical).expect("json");
    value["complete_data_draws_sha256"] = serde_json::json!("not-a-digest");
    assert_eq!(
        RubinLoadingUncertaintyArtifact::from_json(&value.to_string()),
        Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)
    );
}
