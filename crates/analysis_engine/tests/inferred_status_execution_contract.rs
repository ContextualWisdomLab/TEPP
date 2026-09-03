//! End-to-end contract for cutoff-safe inferred-status refusals.

use analysis_engine::{
    AnalysisEngineError, INFERRED_STATUS_ARTIFACT_SCHEMA_VERSION,
    INFERRED_STATUS_MODEL_CONTRACT_VERSION, INFERRED_STATUS_OUTPUT_PROFILE, InferredStatusEvidence,
    MAX_EVIDENCE_UNITS, execute_inferred_status_run,
};
use inferred_status::EvidenceStatus;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "inferred-status-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-inferred-status".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: INFERRED_STATUS_MODEL_CONTRACT_VERSION.into(),
        output_profile: INFERRED_STATUS_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-inferred-status", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn evidence(evidence_id: &str, status: EvidenceStatus, stamp: &str) -> InferredStatusEvidence {
    InferredStatusEvidence::new(evidence_id, status, available(stamp)).expect("evidence")
}

fn mixed_evidence() -> Vec<InferredStatusEvidence> {
    vec![
        evidence(
            "observed-a",
            EvidenceStatus::Observed,
            "2026-07-01T00:00:00Z",
        ),
        evidence(
            "inferred-b",
            EvidenceStatus::Inferred,
            "2026-07-02T00:00:00Z",
        ),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    evidence: &[InferredStatusEvidence],
) -> Result<analysis_engine::InferredStatusExecution, AnalysisEngineError> {
    execute_inferred_status_run(
        request,
        &accepted(request),
        "snapshot-inferred-status",
        cutoff(),
        evidence,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_statuses_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_evidence()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        INFERRED_STATUS_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.evidence_count, 2);
    assert_eq!(execution.artifact.observed_count, 1);
    assert_eq!(execution.artifact.inferred_count, 1);
    assert_eq!(execution.artifact.refused_as_observed_count, 1);
    assert_eq!(execution.artifact.refused_as_transition_count, 1);
    assert_eq!(
        execution.artifact.inference_status,
        "inferred_is_not_observed_and_not_transition"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("identity_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
    assert!(!payload.contains("unobserved"));
    assert!(!payload.contains("no_relationship"));
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
        Some(INFERRED_STATUS_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn equivalent_cutoff_offsets_are_compared_by_instant() {
    let mut equivalent = request();
    equivalent.knowledge_cutoff = "2026-08-01T09:00:00+09:00".into();
    let execution = execute(&equivalent, &mixed_evidence()).expect("equivalent cutoff instant");
    assert_eq!(execution.artifact.knowledge_cutoff, "2026-08-01T00:00:00Z");
}

#[test]
fn future_available_evidence_is_excluded() {
    let request = request();
    let mut with_future = mixed_evidence();
    with_future.push(evidence(
        "future-c",
        EvidenceStatus::Observed,
        "2026-08-02T00:00:00Z",
    ));
    let execution = execute(&request, &with_future).expect("cutoff");
    assert_eq!(execution.artifact.evidence_count, 2);
    assert_eq!(execution.artifact.observed_count, 1);
}

#[test]
fn empty_or_single_class_and_duplicate_fail_closed() {
    let request = request();
    let stamp = "2026-07-01T00:00:00Z";
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let observed_only = vec![
        evidence("observed-a", EvidenceStatus::Observed, stamp),
        evidence("observed-b", EvidenceStatus::Observed, stamp),
    ];
    assert_eq!(
        execute(&request, &observed_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let inferred_only = vec![
        evidence("inferred-a", EvidenceStatus::Inferred, stamp),
        evidence("inferred-b", EvidenceStatus::Inferred, stamp),
    ];
    assert_eq!(
        execute(&request, &inferred_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        evidence("same", EvidenceStatus::Observed, stamp),
        evidence("same", EvidenceStatus::Inferred, stamp),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        InferredStatusEvidence::new("", EvidenceStatus::Observed, available(stamp)),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_cutoff_mismatch_and_oversize() {
    let request = request();
    let rows = mixed_evidence();
    assert_eq!(
        execute_inferred_status_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &rows,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let mut mismatched = request.clone();
    mismatched.knowledge_cutoff = "2026-07-01T00:00:00Z".into();
    assert_eq!(
        execute_inferred_status_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-inferred-status",
            cutoff(),
            &rows,
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
        "episode_membership_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_inferred_status_run(
                &reused,
                &accepted(&reused),
                "snapshot-inferred-status",
                cutoff(),
                &rows,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    let oversized: Vec<InferredStatusEvidence> = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            evidence(
                &format!("evidence-{index}"),
                EvidenceStatus::Observed,
                "2026-07-01T00:00:00Z",
            )
        })
        .collect();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
