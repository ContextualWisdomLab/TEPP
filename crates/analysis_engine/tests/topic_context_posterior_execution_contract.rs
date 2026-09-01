//! End-to-end contract for cutoff-safe posterior topic-context analysis-run.

use std::collections::BTreeMap;

use analysis_engine::{
    AnalysisEngineError, TOPIC_CONTEXT_POSTERIOR_MODEL_CONTRACT_VERSION,
    TOPIC_CONTEXT_POSTERIOR_OUTPUT_PROFILE, TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION,
    TopicActivityInterval, TopicContextMembership, TopicContextPosteriorArtifact,
    TopicContextPosteriorSnapshotManifest, TopicDocumentRelation, TopicPostPlausibleValue,
    execute_topic_context_posterior_run,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn artifact() -> TopicContextPosteriorArtifact {
    let documents = [
        "018f3f7a-7b7c-7d00-8000-000000000001",
        "018f3f7a-7b7c-7d00-8000-000000000002",
    ];
    TopicContextPosteriorArtifact {
        schema_version: TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION.into(),
        run_id: "run-topic-context-posterior".into(),
        snapshot_id: "snapshot-topic-context-posterior".into(),
        source_snapshot_sha256: "0".repeat(64),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        event_clock_code: "event_time_rfc3339".into(),
        model_contract_version: "trsl-tm-v1".into(),
        posterior_draw_set_id: "draw-set-1".into(),
        posterior_draw_count: 2,
        topic_count: 2,
        topic_ids: vec![
            "018f3f7a-7b7c-7d00-8000-000000000101".into(),
            "018f3f7a-7b7c-7d00-8000-000000000102".into(),
        ],
        activity_intervals: [
            "018f3f7a-7b7c-7d00-8000-000000000101",
            "018f3f7a-7b7c-7d00-8000-000000000102",
        ]
        .map(|topic_id| TopicActivityInterval {
            topic_id: topic_id.into(),
            state_code: "active".into(),
            valid_from: "2026-07-01T00:00:00Z".into(),
            valid_to: "2026-07-15T00:00:00Z".into(),
        })
        .into(),
        lineage_events: vec![],
        document_relations: vec![TopicDocumentRelation {
            source_document_id: documents[0].into(),
            target_document_id: documents[1].into(),
            relation_kind_code: "event_lineage_precedes".into(),
            event_time: "2026-07-15T00:00:00Z".into(),
            evidence_sha256: "c".repeat(64),
            evidence_resource_id: "evidence-relation-1".into(),
            provenance_assertion_id: "provenance-relation-1".into(),
        }],
        plausible_values: documents
            .iter()
            .flat_map(|document| {
                (0..2).map(|draw| TopicPostPlausibleValue {
                    document_id: (*document).into(),
                    draw_index: draw,
                    event_time: "2026-07-15T00:00:00Z".into(),
                    logistic_normal_coordinates: vec![if draw == 0 { 0.0 } else { 0.1 }],
                })
            })
            .collect(),
        memberships: documents
            .iter()
            .flat_map(|document| {
                ["business_unit", "process_unit", "team", "person"].map(|dimension| {
                    TopicContextMembership {
                        document_id: (*document).into(),
                        dimension_code: dimension.into(),
                        context_id: format!("{dimension}-{document}"),
                        weight: 1.0,
                        valid_from: "2026-07-01T00:00:00Z".into(),
                        valid_to: "2026-08-01T00:00:00Z".into(),
                        evidence_sha256: "b".repeat(64),
                        evidence_resource_id: format!("evidence-{dimension}-{document}"),
                        provenance_assertion_id: format!("provenance-{dimension}-{document}"),
                    }
                })
            })
            .collect(),
        inference_status: "posterior_topic_coordinates_not_importance".into(),
    }
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "topic-context-posterior-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-topic-context-posterior".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: TOPIC_CONTEXT_POSTERIOR_MODEL_CONTRACT_VERSION.into(),
        output_profile: TOPIC_CONTEXT_POSTERIOR_OUTPUT_PROFILE.into(),
    }
}

fn manifest(artifact: &TopicContextPosteriorArtifact) -> TopicContextPosteriorSnapshotManifest {
    TopicContextPosteriorSnapshotManifest {
        snapshot_id: artifact.snapshot_id.clone(),
        source_snapshot_sha256: artifact.source_snapshot_sha256.clone(),
        knowledge_cutoff: artifact.knowledge_cutoff.clone(),
        artifact_sha256: artifact.sha256().expect("artifact digest"),
        document_available_at: BTreeMap::from([
            (
                "018f3f7a-7b7c-7d00-8000-000000000001".into(),
                "2026-07-20T00:00:00Z".into(),
            ),
            (
                "018f3f7a-7b7c-7d00-8000-000000000002".into(),
                "2026-08-01T00:00:00Z".into(),
            ),
        ]),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-topic-context-posterior",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
) -> Result<analysis_engine::TopicContextPosteriorExecution, AnalysisEngineError> {
    let artifact = artifact();
    execute_topic_context_posterior_run(
        request,
        &accepted(request),
        &manifest(&artifact),
        &artifact,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn validated_posterior_emits_digest_without_claiming_importance() {
    let request = request();
    let execution = execute(&request).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.topic_count, 2);
    assert_eq!(execution.artifact.posterior_draw_count, 2);
    assert_eq!(
        execution.artifact.inference_status,
        "posterior_topic_coordinates_not_importance"
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
        Some(TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION)
    );
    assert_eq!(
        execution
            .terminal_result
            .summary
            .as_ref()
            .expect("summary")
            .statistic_count,
        4
    );
}

#[test]
fn producer_contract_refusal_and_run_identity_mismatch_fail_closed() {
    let request = request();
    let mut invalid = artifact();
    invalid.plausible_values.pop();
    let invalid_manifest = manifest(&artifact());
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &invalid_manifest,
            &invalid,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let original_artifact = artifact();
    let mut artifact_snapshot_mismatch = original_artifact.clone();
    artifact_snapshot_mismatch.snapshot_id = "other-snapshot".into();
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &manifest(&original_artifact),
            &artifact_snapshot_mismatch,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let mut mismatched_run = artifact();
    mismatched_run.run_id = "other-run".into();
    let mismatched_manifest = manifest(&mismatched_run);
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &mismatched_manifest,
            &mismatched_run,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let artifact = artifact();
    let mut mismatched_manifest = manifest(&artifact);
    mismatched_manifest.snapshot_id = "other-snapshot".into();
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &mismatched_manifest,
            &artifact,
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
            value.output_profile = "lineage_criterion_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "case_deletion_refit_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "composed_fitted_lineage_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "fitted_candidate_k_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "trsl_topic_lineage_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "method_effects_v1".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}

#[test]
fn execution_binds_snapshot_digest_availability_and_producer_contract() {
    let request = request();
    let artifact = artifact();

    let mut wrong_snapshot_digest = manifest(&artifact);
    wrong_snapshot_digest.source_snapshot_sha256 = "1".repeat(64);
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &wrong_snapshot_digest,
            &artifact,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut future_evidence = manifest(&artifact);
    future_evidence.document_available_at.insert(
        "018f3f7a-7b7c-7d00-8000-000000000001".into(),
        "2026-08-01T00:00:01Z".into(),
    );
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &future_evidence,
            &artifact,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut foreign_producer = artifact.clone();
    foreign_producer.model_contract_version = "unapproved-producer-v1".into();
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &manifest(&foreign_producer),
            &foreign_producer,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    for invalid_artifact in [
        {
            let mut value = artifact.clone();
            value.knowledge_cutoff = "2026-07-31T23:59:59Z".into();
            value
        },
        {
            let mut value = artifact.clone();
            value.inference_status = "topic_importance".into();
            value
        },
    ] {
        assert_eq!(
            execute_topic_context_posterior_run(
                &request,
                &accepted(&request),
                &manifest(&artifact),
                &invalid_artifact,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}

#[test]
fn execution_rejects_unbound_artifact_and_availability_manifests() {
    let request = request();
    let artifact = artifact();

    let mut wrong_artifact_digest = manifest(&artifact);
    wrong_artifact_digest.artifact_sha256 = "1".repeat(64);
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &wrong_artifact_digest,
            &artifact,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut incomplete_availability = manifest(&artifact);
    incomplete_availability.document_available_at.pop_first();
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &incomplete_availability,
            &artifact,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut malformed_availability = manifest(&artifact);
    *malformed_availability
        .document_available_at
        .first_entry()
        .expect("document")
        .get_mut() = "not-a-time".into();
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &malformed_availability,
            &artifact,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut malformed_manifest_digest = manifest(&artifact);
    malformed_manifest_digest.artifact_sha256 = "not-a-digest".into();
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &malformed_manifest_digest,
            &artifact,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut malformed_source_digest = manifest(&artifact);
    malformed_source_digest.source_snapshot_sha256 = "not-a-digest".into();
    let mut matching_malformed_source = artifact.clone();
    matching_malformed_source.source_snapshot_sha256 = "not-a-digest".into();
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &malformed_source_digest,
            &matching_malformed_source,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut substituted_availability = manifest(&artifact);
    substituted_availability.document_available_at.pop_first();
    substituted_availability.document_available_at.insert(
        "018f3f7a-7b7c-7d00-8000-000000000099".into(),
        "2026-07-20T00:00:00Z".into(),
    );
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &substituted_availability,
            &artifact,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_rejects_malformed_manifest_and_completion_times() {
    let request = request();
    let artifact = artifact();

    let mut malformed_cutoff = manifest(&artifact);
    malformed_cutoff.knowledge_cutoff = "not-a-time".into();
    assert_eq!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &malformed_cutoff,
            &artifact,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    assert!(
        execute_topic_context_posterior_run(
            &request,
            &accepted(&request),
            &manifest(&artifact),
            &artifact,
            "not-a-time",
        )
        .is_err()
    );
}
