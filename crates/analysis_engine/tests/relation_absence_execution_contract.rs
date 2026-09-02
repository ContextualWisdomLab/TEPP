//! End-to-end contract for cutoff-safe relation-absence refusals.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION,
    RELATION_ABSENCE_MODEL_CONTRACT_VERSION, RELATION_ABSENCE_OUTPUT_PROFILE,
    RelationAbsenceArtifact, RelationAbsencePair, execute_relation_absence_run,
};
use relation_absence::ObservationStatus;
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
        idempotency_key: "relation-absence-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-relation-absence".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: RELATION_ABSENCE_MODEL_CONTRACT_VERSION.into(),
        output_profile: RELATION_ABSENCE_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-relation-absence", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn pair(pair_id: &str, status: ObservationStatus, stamp: &str) -> RelationAbsencePair {
    RelationAbsencePair::new(pair_id, status, available(stamp)).expect("pair")
}

fn mixed_pairs() -> Vec<RelationAbsencePair> {
    vec![
        pair(
            "observed-a",
            ObservationStatus::Observed,
            "2026-07-01T00:00:00Z",
        ),
        pair(
            "inferred-b",
            ObservationStatus::Inferred,
            "2026-07-02T00:00:00Z",
        ),
        pair(
            "unobserved-c",
            ObservationStatus::Unobserved,
            "2026-07-03T00:00:00Z",
        ),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    pairs: &[RelationAbsencePair],
) -> Result<analysis_engine::RelationAbsenceExecution, AnalysisEngineError> {
    execute_relation_absence_run(
        request,
        &accepted(request),
        "snapshot-relation-absence",
        cutoff(),
        pairs,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_statuses_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_pairs()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.pair_count, 3);
    assert_eq!(execution.artifact.observed_count, 1);
    assert_eq!(execution.artifact.inferred_count, 1);
    assert_eq!(execution.artifact.unobserved_count, 1);
    assert_eq!(execution.artifact.refused_as_negative_count, 1);
    assert_eq!(
        execution.artifact.inference_status,
        "unobserved_is_not_negative_observed_inferred_are_presence"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("status_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
    assert!(!payload.contains("no_relationship"));
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
        Some(RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn compact_oversized_artifact_counts_fail_closed() {
    let pair_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let unobserved_count = pair_count - 2;
    let artifact = RelationAbsenceArtifact {
        schema_version: RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        pair_count,
        observed_count: 1,
        inferred_count: 1,
        unobserved_count,
        refused_as_negative_count: unobserved_count,
        inference_status: "unobserved_is_not_negative_observed_inferred_are_presence".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");
    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidRelationAbsenceArtifact)
    );
    assert_eq!(
        RelationAbsenceArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidRelationAbsenceArtifact)
    );
}

#[test]
fn future_available_pairs_are_excluded() {
    let request = request();
    let mut with_future = mixed_pairs();
    with_future.push(pair(
        "future-d",
        ObservationStatus::Observed,
        "2026-08-02T00:00:00Z",
    ));
    let execution = execute(&request, &with_future).expect("cutoff");
    assert_eq!(execution.artifact.pair_count, 3);
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
        pair("observed-a", ObservationStatus::Observed, stamp),
        pair("observed-b", ObservationStatus::Observed, stamp),
        pair("observed-c", ObservationStatus::Observed, stamp),
    ];
    assert_eq!(
        execute(&request, &observed_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let presence_only = vec![
        pair("observed-a", ObservationStatus::Observed, stamp),
        pair("inferred-b", ObservationStatus::Inferred, stamp),
    ];
    assert_eq!(
        execute(&request, &presence_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let unobserved_only = vec![
        pair("unobserved-a", ObservationStatus::Unobserved, stamp),
        pair("unobserved-b", ObservationStatus::Unobserved, stamp),
        pair("unobserved-c", ObservationStatus::Unobserved, stamp),
    ];
    assert_eq!(
        execute(&request, &unobserved_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        pair("same", ObservationStatus::Observed, stamp),
        pair("same", ObservationStatus::Unobserved, stamp),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        RelationAbsencePair::new("", ObservationStatus::Observed, available(stamp)),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_cutoff_mismatch_and_oversize() {
    let request = request();
    let pairs = mixed_pairs();
    assert_eq!(
        execute_relation_absence_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &pairs,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let mut mismatched = request.clone();
    mismatched.knowledge_cutoff = "2026-07-01T00:00:00Z".into();
    assert_eq!(
        execute_relation_absence_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-relation-absence",
            cutoff(),
            &pairs,
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
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_relation_absence_run(
                &reused,
                &accepted(&reused),
                "snapshot-relation-absence",
                cutoff(),
                &pairs,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    let oversized: Vec<RelationAbsencePair> = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            pair(
                &format!("pair-{index}"),
                ObservationStatus::Observed,
                "2026-07-01T00:00:00Z",
            )
        })
        .collect();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
