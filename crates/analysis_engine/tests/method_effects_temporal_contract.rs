//! Temporal-binding and bounded-wire contracts for the method-effects profile.

use analysis_engine::{
    MAX_ANALYSIS_IDENTIFIER_BYTES, METHOD_EFFECTS_ARTIFACT_BYTE_LIMIT,
    METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION, METHOD_EFFECTS_MODEL_CONTRACT_VERSION,
    METHOD_EFFECTS_OUTPUT_PROFILE, MethodEffectsArtifact, execute_method_effects_run,
};
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};
use tepp_simulation::SimulationConfig;

fn request(knowledge_cutoff: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "method-effects-temporal-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-method-effects".into(),
        knowledge_cutoff: knowledge_cutoff.into(),
        model_contract_version: METHOD_EFFECTS_MODEL_CONTRACT_VERSION.into(),
        output_profile: METHOD_EFFECTS_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-method-effects-temporal", "accepted", &request.idempotency_key)
        .expect("accepted")
}

#[test]
fn equivalent_rfc3339_cutoff_spellings_bind_to_the_same_instant() {
    let request = request("2026-08-01T01:00:00+01:00");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");

    let execution = execute_method_effects_run(
        &request,
        &accepted(&request),
        "snapshot-method-effects",
        cutoff,
        SimulationConfig::ci_default(7),
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent cutoff instant must bind");

    assert_eq!(execution.artifact.knowledge_cutoff, "2026-08-01T00:00:00Z");
}

#[test]
fn terminal_validation_status_remains_provider_state_not_domain_inference() {
    let request = request("2026-08-01T00:00:00Z");
    let cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff).expect("cutoff");

    let execution = execute_method_effects_run(
        &request,
        &accepted(&request),
        "snapshot-method-effects",
        cutoff,
        SimulationConfig::ci_default(7),
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
        "simulation_method_effect_labels_not_estimator_model"
    );
}

#[test]
fn every_valid_artifact_shape_remains_below_the_inbound_wire_limit() {
    let maximum_identifier = "x".repeat(MAX_ANALYSIS_IDENTIFIER_BYTES);
    let artifact = MethodEffectsArtifact {
        schema_version: METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: maximum_identifier.clone(),
        snapshot_id: maximum_identifier,
        knowledge_cutoff: "9999-12-31T23:59:59Z".into(),
        seed: u64::MAX,
        config_digest: "f".repeat(64),
        content_digest: "e".repeat(64),
        document_count: u64::MAX,
        original_count: 1,
        revision_count: u64::MAX - 1,
        translation_count: 0,
        template_copy_count: 0,
        derivative_count: u64::MAX - 1,
        inference_status: "simulation_method_effect_labels_not_estimator_model".into(),
    };

    let payload = artifact.to_json().expect("maximal valid artifact");
    assert!(payload.len() < METHOD_EFFECTS_ARTIFACT_BYTE_LIMIT);
    assert_eq!(
        MethodEffectsArtifact::from_json(&payload).expect("round trip"),
        artifact
    );
}
