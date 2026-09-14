//! Regression contracts for topic-context posterior temporal binding.

use std::collections::BTreeMap;

use analysis_engine::{
    TOPIC_CONTEXT_POSTERIOR_MODEL_CONTRACT_VERSION, TOPIC_CONTEXT_POSTERIOR_OUTPUT_PROFILE,
    TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION, TopicActivityInterval, TopicContextMembership,
    TopicContextPosteriorArtifact, TopicContextPosteriorSnapshotManifest, TopicDocumentRelation,
    TopicPostPlausibleValue, execute_topic_context_posterior_run,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

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

fn request(cutoff: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "topic-context-posterior-temporal-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-topic-context-posterior".into(),
        knowledge_cutoff: cutoff.into(),
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

fn execute(cutoff: &str) -> analysis_engine::TopicContextPosteriorExecution {
    let request = request(cutoff);
    let artifact = artifact();
    let accepted = AnalysisRunAccepted::new(
        "run-topic-context-posterior",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted");
    execute_topic_context_posterior_run(
        &request,
        &accepted,
        &manifest(&artifact),
        &artifact,
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent cutoff must execute")
}

#[test]
fn equivalent_rfc3339_cutoff_instants_are_the_same_contract() {
    let execution = execute("2026-08-01T01:00:00+01:00");
    assert_eq!(execution.artifact.knowledge_cutoff, "2026-08-01T00:00:00Z");
}

#[test]
fn terminal_validation_status_is_not_the_scientific_inference_claim() {
    let execution = execute("2026-08-01T00:00:00Z");
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
        "posterior_topic_coordinates_not_importance"
    );
}
