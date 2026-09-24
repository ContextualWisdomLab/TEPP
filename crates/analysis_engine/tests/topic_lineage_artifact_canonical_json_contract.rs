//! Contract tests for exact canonical topic-lineage artifact JSON bytes.

use analysis_engine::{
    TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION, TopicLineageArtifact,
    TopicLineageArtifactDocumentUncertainty, TopicLineageArtifactEdge,
    TopicLineageArtifactUncertaintyCoordinate,
};

const CONFIG_JSON: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const CONFIG_SHA256: &str = "c99da5cab3050e3d5e357bcdccca5405b05263ca2493fdfc74f3080948a3763b";

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

fn artifact() -> TopicLineageArtifact {
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
            uncertainty("00000000-0000-0000-0000-000000000001", 0.125),
            uncertainty("00000000-0000-0000-0000-000000000002", 0.25),
        ],
        selected_seed: 7,
        iterations: 4,
        objective: -1.0,
        topic_count: 2,
        evidence_count: 2,
        connected_post_count: 2,
        lineage_count: 1,
        sequence_edges: vec![TopicLineageArtifactEdge {
            predecessor_document_id: "00000000-0000-0000-0000-000000000001".into(),
            successor_document_id: "00000000-0000-0000-0000-000000000002".into(),
            topic_index: 0,
            association_strength: 0.8,
        }],
        inference_status: "fitted_topic_association_not_causation".into(),
    }
}

#[test]
fn artifact_parser_accepts_only_owner_canonical_json_bytes() {
    let canonical = artifact().to_json().expect("canonical artifact");
    assert!(TopicLineageArtifact::from_json(&canonical).is_ok());

    let trailing_newline = format!("{canonical}\n");
    assert!(TopicLineageArtifact::from_json(&trailing_newline).is_err());

    let value: serde_json::Value = serde_json::from_str(&canonical).expect("json value");
    let pretty = serde_json::to_string_pretty(&value).expect("pretty json");
    assert_ne!(pretty, canonical);
    assert!(TopicLineageArtifact::from_json(&pretty).is_err());
}
