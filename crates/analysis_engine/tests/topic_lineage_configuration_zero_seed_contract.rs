//! Contract tests for nonzero deterministic topic-lineage initialization seeds.

use analysis_engine::{
    TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION, TopicLineageArtifact,
    TopicLineageArtifactDocumentUncertainty, TopicLineageArtifactUncertaintyCoordinate,
};

const NONZERO_SEED_CONFIG: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[1,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const NONZERO_SEED_SHA256: &str = "ebfd5b4f226222038c61b7767d4b5208bd8460cb4ff555aba744682e7cccd1dd";
const ZERO_SEED_CONFIG: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[0,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const ZERO_SEED_SHA256: &str = "ab0ba158a7da7e7c73011bbcf4ebe4da99cc58872e2ec49d57bef6cb12043769";

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

fn artifact(
    configuration_json: &str,
    configuration_sha256: &str,
    selected_seed: u64,
) -> TopicLineageArtifact {
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
        selected_seed,
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
fn released_seed_manifest_refuses_zero_rng_alias() {
    assert!(
        artifact(NONZERO_SEED_CONFIG, NONZERO_SEED_SHA256, 1)
            .to_json()
            .is_ok(),
        "nonzero deterministic seeds remain valid"
    );
    assert!(
        artifact(ZERO_SEED_CONFIG, ZERO_SEED_SHA256, 0)
            .to_json()
            .is_err(),
        "seed zero must not alias the deterministic RNG stream used by seed one"
    );
}
