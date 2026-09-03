//! End-to-end contract for cutoff-safe location-membership refusals.

use analysis_engine::{
    AnalysisEngineError, LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION,
    LOCATION_MEMBERSHIP_MODEL_CONTRACT_VERSION, LOCATION_MEMBERSHIP_OUTPUT_PROFILE,
    LocationMembershipDocument, execute_location_membership_run,
};
use location_membership::LocationKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn available(value: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(value).expect("availability")
}

fn document(id: &str, kind: LocationKind) -> LocationMembershipDocument {
    LocationMembershipDocument::new(id, kind, available("2026-07-31T23:59:59Z")).expect("document")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "location-membership-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-location-membership".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: LOCATION_MEMBERSHIP_MODEL_CONTRACT_VERSION.into(),
        output_profile: LOCATION_MEMBERSHIP_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-location-membership",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn mixed_documents() -> Vec<LocationMembershipDocument> {
    vec![
        document("loc-a", LocationKind::Location),
        document("ent-b", LocationKind::EntityIdentity),
        document("lang-c", LocationKind::LanguageChannel),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    documents: &[LocationMembershipDocument],
) -> Result<analysis_engine::LocationMembershipExecution, AnalysisEngineError> {
    execute_location_membership_run(
        request,
        &accepted(request),
        "snapshot-location-membership",
        cutoff(),
        documents,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_location_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_documents()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.document_count, 3);
    assert_eq!(execution.artifact.location_count, 1);
    assert_eq!(execution.artifact.entity_identity_count, 1);
    assert_eq!(execution.artifact.language_channel_count, 1);
    assert_eq!(execution.artifact.refused_as_entity_identity_count, 1);
    assert_eq!(execution.artifact.refused_as_language_channel_count, 1);
    assert_eq!(
        execution.artifact.inference_status,
        "location_is_not_entity_identity_not_language_channel"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("identity_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
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
        Some(LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION)
    );
    assert_eq!(
        execution
            .terminal_result
            .summary
            .as_ref()
            .expect("summary")
            .statistic_count,
        5
    );
}

#[test]
fn empty_single_kind_and_duplicate_identities_fail_closed() {
    let request = request();
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let location_only = vec![
        document("loc-a", LocationKind::Location),
        document("loc-b", LocationKind::Location),
    ];
    assert_eq!(
        execute(&request, &location_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let entity_only = vec![
        document("ent-a", LocationKind::EntityIdentity),
        document("ent-b", LocationKind::EntityIdentity),
    ];
    assert_eq!(
        execute(&request, &entity_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let language_only = vec![
        document("lang-a", LocationKind::LanguageChannel),
        document("lang-b", LocationKind::LanguageChannel),
    ];
    assert_eq!(
        execute(&request, &language_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let no_location = vec![
        document("ent-a", LocationKind::EntityIdentity),
        document("lang-b", LocationKind::LanguageChannel),
    ];
    assert_eq!(
        execute(&request, &no_location),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        document("same", LocationKind::Location),
        document("same", LocationKind::EntityIdentity),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        LocationMembershipDocument::new(
            "",
            LocationKind::Location,
            available("2026-07-31T23:59:59Z"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn availability_cutoff_and_document_limit_fail_closed() {
    let request = request();
    let mut documents = mixed_documents();
    documents[0] = LocationMembershipDocument::new(
        "loc-a",
        LocationKind::Location,
        available("2026-08-01T00:00:00Z"),
    )
    .expect("at cutoff");
    execute(&request, &documents).expect("availability at cutoff");

    documents[0] = LocationMembershipDocument::new(
        "loc-a",
        LocationKind::Location,
        available("2026-08-01T00:00:00.000000001Z"),
    )
    .expect("after cutoff");
    assert_eq!(
        execute(&request, &documents),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let oversized = (0..=analysis_engine::MAX_EVIDENCE_UNITS)
        .map(|index| document(&format!("document-{index}"), LocationKind::Location))
        .collect::<Vec<_>>();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let documents = mixed_documents();
    assert_eq!(
        execute_location_membership_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &documents,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let mut mismatched = request.clone();
    mismatched.knowledge_cutoff = "2026-07-01T00:00:00Z".into();
    assert_eq!(
        execute_location_membership_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-location-membership",
            cutoff(),
            &documents,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let mut mismatched_model = request.clone();
    mismatched_model.model_contract_version = "other-model".into();
    assert_eq!(
        execute_location_membership_run(
            &mismatched_model,
            &accepted(&mismatched_model),
            "snapshot-location-membership",
            cutoff(),
            &documents,
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
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_location_membership_run(
                &reused,
                &accepted(&reused),
                "snapshot-location-membership",
                cutoff(),
                &documents,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    assert!(
        execute_location_membership_run(
            &request,
            &accepted(&request),
            "snapshot-location-membership",
            cutoff(),
            &documents,
            "not-a-time",
        )
        .is_err()
    );
}
