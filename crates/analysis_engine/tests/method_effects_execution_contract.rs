//! End-to-end contract for cutoff-safe simulation method-effect labels.

use analysis_engine::{
    AnalysisEngineError, METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION,
    METHOD_EFFECTS_MODEL_CONTRACT_VERSION, METHOD_EFFECTS_OUTPUT_PROFILE,
    execute_method_effects_run,
};
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};
use tepp_simulation::SimulationConfig;

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "method-effects-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-method-effects".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: METHOD_EFFECTS_MODEL_CONTRACT_VERSION.into(),
        output_profile: METHOD_EFFECTS_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-method-effects", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
) -> Result<analysis_engine::MethodEffectsExecution, AnalysisEngineError> {
    execute_method_effects_run(
        request,
        &accepted(request),
        "snapshot-method-effects",
        cutoff(),
        SimulationConfig::ci_default(7),
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_method_effects_emit_digest_bound_census_without_estimator_model() {
    let request = request();
    let execution = execute(&request).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.seed, 7);
    assert_eq!(execution.artifact.config_digest.len(), 64);
    assert_eq!(execution.artifact.content_digest.len(), 64);
    assert!(execution.artifact.document_count >= 2);
    assert!(execution.artifact.original_count >= 1);
    assert_eq!(
        execution.artifact.original_count
            + execution.artifact.revision_count
            + execution.artifact.translation_count
            + execution.artifact.template_copy_count,
        execution.artifact.document_count
    );
    assert_eq!(
        execution.artifact.revision_count
            + execution.artifact.translation_count
            + execution.artifact.template_copy_count,
        execution.artifact.derivative_count
    );
    assert!(execution.artifact.derivative_count >= 1);
    assert_eq!(
        execution.artifact.inference_status,
        "simulation_method_effect_labels_not_estimator_model"
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
        Some(METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn empty_available_corpus_and_single_original_fail_closed() {
    let request = request();
    let early = KnowledgeCutoff::parse_rfc3339("2025-12-31T00:00:00Z").expect("early");
    let mut early_request = request.clone();
    early_request.knowledge_cutoff = early.to_rfc3339();
    assert_eq!(
        execute_method_effects_run(
            &early_request,
            &accepted(&early_request),
            "snapshot-method-effects",
            early,
            SimulationConfig::ci_default(7),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let singleton = SimulationConfig::new(7, 1, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0).expect("config");
    assert_eq!(
        execute_method_effects_run(
            &request,
            &accepted(&request),
            "snapshot-method-effects",
            cutoff(),
            singleton,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    assert_eq!(
        execute_method_effects_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            SimulationConfig::ci_default(7),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let mut mismatched = request.clone();
    mismatched.knowledge_cutoff = "2026-07-01T00:00:00Z".into();
    assert_eq!(
        execute_method_effects_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-method-effects",
            cutoff(),
            SimulationConfig::ci_default(7),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    for profile in [
        "trsl_topic_lineage_v1",
        "fitted_candidate_k_v1",
        "pareto_candidate_k_v1",
        "joint_posterior_draws_v1",
        "composed_fitted_lineage_v1",
        "case_deletion_refit_v1",
        "topic_activity_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_method_effects_run(
                &reused,
                &accepted(&reused),
                "snapshot-method-effects",
                cutoff(),
                SimulationConfig::ci_default(7),
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
