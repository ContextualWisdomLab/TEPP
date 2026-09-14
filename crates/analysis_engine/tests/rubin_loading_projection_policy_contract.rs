//! Projection-policy contract for Rubin loading uncertainty.
//!
//! Correct Rubin combination arithmetic is not, by itself, evidence that an
//! arbitrary caller-supplied draw matrix belongs to a validated inferential
//! regime. Until draw-generation provenance is bound to approved validation
//! evidence, the product artifact must say that its projection is descriptive.

use analysis_engine::{
    AnalysisEngineError, RUBIN_LOADING_MODEL_CONTRACT_VERSION, RUBIN_LOADING_OUTPUT_PROFILE,
    RubinLoadingObservation, RubinLoadingUncertaintyArtifact,
    execute_rubin_loading_uncertainty_run,
};
use psychometric_core::IndicatorKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

const SNAPSHOT_ID: &str = "snapshot-rubin-projection-policy";
const CUTOFF: &str = "2026-08-01T00:00:00Z";
const DESCRIPTIVE_ONLY: &str = "descriptive_only_unbound_draw_generation_provenance";

fn observation(factor_score: f64, draws: Vec<f64>) -> RubinLoadingObservation {
    RubinLoadingObservation::new(
        SNAPSHOT_ID,
        factor_score,
        draws,
        AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("availability"),
    )
    .expect("observation")
}

fn execute() -> analysis_engine::RubinLoadingUncertaintyExecution {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "rubin-projection-policy-idem".into(),
        tenant_workspace_id: "tenant-rubin-projection-policy".into(),
        snapshot_id: SNAPSHOT_ID.into(),
        knowledge_cutoff: CUTOFF.into(),
        model_contract_version: RUBIN_LOADING_MODEL_CONTRACT_VERSION.into(),
        output_profile: RUBIN_LOADING_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new(
        "run-rubin-projection-policy",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted");
    let rows = vec![
        observation(-1.0, vec![-0.7, -0.9]),
        observation(0.0, vec![0.0, 0.0]),
        observation(1.0, vec![0.7, 0.9]),
    ];

    execute_rubin_loading_uncertainty_run(
        &request,
        &accepted,
        SNAPSHOT_ID,
        KnowledgeCutoff::parse_rfc3339(CUTOFF).expect("cutoff"),
        IndicatorKind::AdditiveLogRatio,
        &rows,
        "2026-08-02T00:00:00Z",
    )
    .expect("descriptive combination remains executable")
}

#[test]
fn provenance_free_draws_are_explicitly_descriptive_only() {
    let execution = execute();

    assert_eq!(execution.artifact.projection_status(), DESCRIPTIVE_ONLY);
    let artifact_json: serde_json::Value =
        serde_json::from_str(&execution.artifact.to_json().expect("artifact json"))
            .expect("valid json");
    assert_eq!(
        artifact_json
            .get("projection_status")
            .and_then(serde_json::Value::as_str),
        Some(DESCRIPTIVE_ONLY)
    );
}

#[test]
fn projection_policy_wire_refuses_missing_or_forged_status() {
    let execution = execute();
    let canonical = execution.artifact.to_json().expect("artifact json");

    let mut missing: serde_json::Value = serde_json::from_str(&canonical).expect("valid json");
    missing
        .as_object_mut()
        .expect("artifact object")
        .remove("projection_status");
    assert_eq!(
        RubinLoadingUncertaintyArtifact::from_json(&missing.to_string()),
        Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)
    );

    let mut forged: serde_json::Value = serde_json::from_str(&canonical).expect("valid json");
    forged["projection_status"] = serde_json::json!("validated_rubin_inference");
    assert_eq!(
        RubinLoadingUncertaintyArtifact::from_json(&forged.to_string()),
        Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)
    );
}
