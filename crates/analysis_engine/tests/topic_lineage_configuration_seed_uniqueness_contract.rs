//! Contract tests for unique deterministic topic-lineage initialization seeds.

use analysis_engine::{
    TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION, TopicLineageArtifact,
    TopicLineageArtifactDocumentUncertainty, TopicLineageArtifactUncertaintyCoordinate,
};

const UNIQUE_SEED_CONFIG: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const UNIQUE_SEED_SHA256: &str = "c99da5cab3050e3d5e357bcdccca5405b05263ca2493fdfc74f3080948a3763b";
const DUPLICATE_SEED_CONFIG: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11,7],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const DUPLICATE_SEED_SHA256: &str = "d2f5e763545be1c7c19ad2c3865c75cb01c52da5a6034b72e72e64a81e3cb18c";

fn uncertainty(document_id: u128, variance: f64) -> TopicLineageArtifactDocumentUncertainty {
    TopicLineageArtifactDocumentUncertainty {
        document_id: uuid::Uuid::from_u128(document_id).to_string(),
        coordinates: vec![TopicLineageArtifactUncertaintyCoordinate {
            numerator_topic_index: 0,
            reference_topic_index: 1,
            variance,
        }],
    }
}

fn artifact(configuration_json: &str, configuration_sha256: &str) -> TopicLineageArtifact {
    TopicLineageArtifact {
        schema_version: TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-1".into(),
        snapshot_id: "snapshot-1".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "trsl_tm_cpu_f64_v1".into(),
        method_configuration_json: configuration_json.into(),
        method_configuration_sha256: configuration_sha256.into(),
        estimator_backend: "cpu_f64_reference".into(),
        posterior_approximation: "diagonal_laplace".into(),
        diagonal_laplace_uncertainty: vec![uncertainty(1, 0.125), uncertainty(2, 0.25)],
        selected_seed: 7,
        iterations: 4,
        objective: -1.0,
        topic_count: 2,
        evidence_count: 2,
        connected_post_count: 0,
        lineage_count: 0,
        sequence_edges: Vec::new(),
        inference_status: "fitted_topic_association_not_causation".into(),
    }
}

#[test]
fn deterministic_seed_manifest_refuses_duplicate_seed_aliases() {
    assert!(
        artifact(UNIQUE_SEED_CONFIG, UNIQUE_SEED_SHA256)
            .to_json()
            .is_ok(),
        "ordered unique deterministic seeds remain valid"
    );
    assert!(
        artifact(DUPLICATE_SEED_CONFIG, DUPLICATE_SEED_SHA256)
            .to_json()
            .is_err(),
        "repeating an already-declared deterministic seed must not mint a second released configuration identity"
    );
}
