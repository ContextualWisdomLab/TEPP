//! Contract tests for canonical topic-lineage sequence-edge ordering.

use analysis_engine::{
    TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION, TopicLineageArtifact,
    TopicLineageArtifactDocumentUncertainty, TopicLineageArtifactEdge,
    TopicLineageArtifactUncertaintyCoordinate,
};

const CONFIG_JSON: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const CONFIG_SHA256: &str = "c99da5cab3050e3d5e357bcdccca5405b05263ca2493fdfc74f3080948a3763b";

fn edge(predecessor: &str, successor: &str, topic_index: u64) -> TopicLineageArtifactEdge {
    TopicLineageArtifactEdge {
        predecessor_document_id: predecessor.into(),
        successor_document_id: successor.into(),
        topic_index,
        association_strength: 0.8,
    }
}

fn uncertainty(document_id: &str, variance: f64) -> TopicLineageArtifactDocumentUncertainty {
    TopicLineageArtifactDocumentUncertainty {
        document_id: document_id.into(),
        coordinates: vec![TopicLineageArtifactUncertaintyCoordinate {
            numerator_topic_index: 0,
            reference_topic_index: 1,
            variance,
        }],
    }
}

fn artifact(sequence_edges: Vec<TopicLineageArtifactEdge>) -> TopicLineageArtifact {
    TopicLineageArtifact {
        schema_version: TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-1".into(),
        snapshot_id: "snapshot-1".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "trsl_tm_cpu_f64_v1".into(),
        method_configuration_json: CONFIG_JSON.into(),
        method_configuration_sha256: CONFIG_SHA256.into(),
        estimator_backend: "cpu_f64_reference".into(),
        posterior_approximation: "diagonal_laplace".into(),
        diagonal_laplace_uncertainty: vec![
            uncertainty("11111111-1111-4111-8111-111111111111", 0.125),
            uncertainty("22222222-2222-4222-8222-222222222222", 0.25),
            uncertainty("33333333-3333-4333-8333-333333333333", 0.375),
            uncertainty("44444444-4444-4444-8444-444444444444", 0.5),
        ],
        selected_seed: 7,
        iterations: 4,
        objective: -1.0,
        topic_count: 2,
        evidence_count: 4,
        connected_post_count: 4,
        lineage_count: 2,
        sequence_edges,
        inference_status: "fitted_topic_association_not_causation".into(),
    }
}

#[test]
fn artifact_sequence_edges_require_one_canonical_coordinate_order() {
    let first = edge(
        "11111111-1111-4111-8111-111111111111",
        "22222222-2222-4222-8222-222222222222",
        0,
    );
    let second = edge(
        "33333333-3333-4333-8333-333333333333",
        "44444444-4444-4444-8444-444444444444",
        1,
    );

    let canonical = artifact(vec![first.clone(), second.clone()]);
    assert!(canonical.to_json().is_ok());

    let reordered = artifact(vec![second, first]);
    assert!(
        reordered.to_json().is_err(),
        "permuting the same semantic edge set must not mint another valid artifact digest"
    );
}
