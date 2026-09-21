//! Release-contract RED for explicit topic-lineage posterior-uncertainty semantics.

use analysis_engine::{AnalysisEngineError, TopicLineageArtifact};

const CONFIG_JSON: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const CONFIG_SHA256: &str = "c99da5cab3050e3d5e357bcdccca5405b05263ca2493fdfc74f3080948a3763b";

fn artifact_json(posterior_approximation: &str) -> String {
    serde_json::json!({
        "schema_version": "tepp.trsl_topic_lineage.v2",
        "run_id": "run-1",
        "snapshot_id": "snapshot-1",
        "knowledge_cutoff": "2026-08-01T00:00:00Z",
        "model_contract_version": "trsl_tm_cpu_f64_v1",
        "method_configuration_json": CONFIG_JSON,
        "method_configuration_sha256": CONFIG_SHA256,
        "estimator_backend": "cpu_f64_reference",
        "posterior_approximation": posterior_approximation,
        "selected_seed": 7,
        "iterations": 4,
        "objective": -1.0,
        "topic_count": 2,
        "evidence_count": 2,
        "connected_post_count": 2,
        "lineage_count": 1,
        "sequence_edges": [{
            "predecessor_document_id": "00000000-0000-0000-0000-000000000001",
            "successor_document_id": "00000000-0000-0000-0000-000000000002",
            "topic_index": 0,
            "association_strength": 0.8
        }],
        "inference_status": "fitted_topic_association_not_causation"
    })
    .to_string()
}

#[test]
fn v2_artifact_names_the_uncertainty_approximation_used_by_the_reference_fit() {
    let artifact = TopicLineageArtifact::from_json(&artifact_json("diagonal_laplace"))
        .expect("uncertainty-bound v2 artifact");
    assert_eq!(artifact.posterior_approximation, "diagonal_laplace");
}

#[test]
fn unknown_or_stronger_uncertainty_claims_fail_closed() {
    for invalid in ["independent_diagonal", "joint_gauss_newton_laplace", "unknown"] {
        assert_eq!(
            TopicLineageArtifact::from_json(&artifact_json(invalid)),
            Err(AnalysisEngineError::InvalidTopicLineageArtifact)
        );
    }
}
