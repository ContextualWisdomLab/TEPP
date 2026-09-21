//! Release-contract RED for reconstructable topic-lineage estimator configuration.

use analysis_engine::{
    AnalysisEngineError, TOPIC_LINEAGE_OUTPUT_PROFILE, TopicLineageArtifact,
};

const CONFIG_JSON: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const CONFIG_SHA256: &str = "c99da5cab3050e3d5e357bcdccca5405b05263ca2493fdfc74f3080948a3763b";

fn artifact_json(config_json: &str, config_sha256: &str) -> String {
    serde_json::json!({
        "schema_version": "tepp.trsl_topic_lineage.v2",
        "run_id": "run-1",
        "snapshot_id": "snapshot-1",
        "knowledge_cutoff": "2026-08-01T00:00:00Z",
        "model_contract_version": "trsl_tm_cpu_f64_v1",
        "method_configuration_json": config_json,
        "method_configuration_sha256": config_sha256,
        "estimator_backend": "cpu_f64_reference",
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
fn v2_artifact_carries_reconstructable_exact_method_configuration() {
    assert_eq!(TOPIC_LINEAGE_OUTPUT_PROFILE, "trsl_topic_lineage_v2");
    let artifact = TopicLineageArtifact::from_json(&artifact_json(CONFIG_JSON, CONFIG_SHA256))
        .expect("configuration-bound v2 artifact");
    assert_eq!(artifact.schema_version, "tepp.trsl_topic_lineage.v2");
    assert_eq!(artifact.model_contract_version, "trsl_tm_cpu_f64_v1");
    assert_eq!(artifact.method_configuration_json, CONFIG_JSON);
    assert_eq!(artifact.method_configuration_sha256, CONFIG_SHA256);
    assert_eq!(artifact.estimator_backend, "cpu_f64_reference");
}

#[test]
fn configuration_identity_tampering_fails_closed() {
    let wrong_digest = "0".repeat(64);
    assert_eq!(
        TopicLineageArtifact::from_json(&artifact_json(CONFIG_JSON, &wrong_digest)),
        Err(AnalysisEngineError::InvalidTopicLineageArtifact)
    );

    let mismatched_topic_count = CONFIG_JSON.replace("\"topic_count\":2", "\"topic_count\":3");
    assert_eq!(
        TopicLineageArtifact::from_json(&artifact_json(&mismatched_topic_count, CONFIG_SHA256)),
        Err(AnalysisEngineError::InvalidTopicLineageArtifact)
    );
}
