//! Contract tests for canonical zero-valued topic-lineage hyperparameters.

use analysis_engine::{
    TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION, TopicLineageArtifact,
    TopicLineageArtifactDocumentUncertainty, TopicLineageArtifactUncertaintyCoordinate,
};

const POSITIVE_ZERO_CONFIG: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.0,\"ridge\":0.0,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const POSITIVE_ZERO_SHA256: &str = "8e3f5013e106e57fb841136957b9956848f4faddf67a75511cb45004a7034c8b";
const NEGATIVE_RELATION_ZERO_CONFIG: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":-0.0,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const NEGATIVE_RELATION_ZERO_SHA256: &str = "e81e568f7766a9a777942ae43ab794b94ce4cb14cddafb71a2abcb79ef1417c6";
const NEGATIVE_RIDGE_ZERO_CONFIG: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":-0.0,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const NEGATIVE_RIDGE_ZERO_SHA256: &str = "ed6783cb3b93034873e2aa1c7e1147ff80365988d9b493141b5a2fe0f6ba0fe7";

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
fn zero_capable_hyperparameters_have_one_positive_zero_byte_identity() {
    assert!(
        artifact(POSITIVE_ZERO_CONFIG, POSITIVE_ZERO_SHA256)
            .to_json()
            .is_ok(),
        "declared zero ablations remain valid in their canonical positive-zero spelling"
    );
    assert!(
        artifact(
            NEGATIVE_RELATION_ZERO_CONFIG,
            NEGATIVE_RELATION_ZERO_SHA256
        )
        .to_json()
        .is_err(),
        "negative-zero relation strength must not mint a second configuration identity"
    );
    assert!(
        artifact(NEGATIVE_RIDGE_ZERO_CONFIG, NEGATIVE_RIDGE_ZERO_SHA256)
            .to_json()
            .is_err(),
        "negative-zero ridge must not mint a second configuration identity"
    );
}
