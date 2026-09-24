//! Release-contract tests for coordinate-bound diagonal-Laplace quantities.

use analysis_engine::{AnalysisEngineError, TopicLineageArtifact};
use serde_json::{Value, json};

const CONFIG_JSON: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";
const CONFIG_SHA256: &str = "c99da5cab3050e3d5e357bcdccca5405b05263ca2493fdfc74f3080948a3763b";

fn canonical_uncertainty() -> Value {
    json!([
        {
            "document_id": "00000000-0000-0000-0000-000000000001",
            "coordinates": [{
                "numerator_topic_index": 0,
                "reference_topic_index": 1,
                "variance": 0.125
            }]
        },
        {
            "document_id": "00000000-0000-0000-0000-000000000002",
            "coordinates": [{
                "numerator_topic_index": 0,
                "reference_topic_index": 1,
                "variance": 0.25
            }]
        }
    ])
}

fn artifact_json(uncertainty: Value) -> String {
    json!({
        "schema_version": "tepp.trsl_topic_lineage.v2",
        "run_id": "run-1",
        "snapshot_id": "snapshot-1",
        "knowledge_cutoff": "2026-08-01T00:00:00Z",
        "model_contract_version": "trsl_tm_cpu_f64_v1",
        "method_configuration_json": CONFIG_JSON,
        "method_configuration_sha256": CONFIG_SHA256,
        "estimator_backend": "cpu_f64_reference",
        "posterior_approximation": "diagonal_laplace",
        "diagonal_laplace_uncertainty": uncertainty,
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
fn v2_accepts_coordinate_bound_diagonal_laplace_quantities() {
    let payload = artifact_json(canonical_uncertainty());
    let artifact = TopicLineageArtifact::from_json(&payload)
        .expect("coordinate-bound diagonal uncertainty");
    assert_eq!(artifact.diagonal_laplace_uncertainty.len(), 2);
    assert_eq!(
        artifact.diagonal_laplace_uncertainty[0].document_id,
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(
        artifact.diagonal_laplace_uncertainty[0].coordinates[0].numerator_topic_index,
        0
    );
    assert_eq!(
        artifact.diagonal_laplace_uncertainty[0].coordinates[0].reference_topic_index,
        1
    );
    assert_eq!(artifact.to_json().expect("canonical artifact"), payload);
}

#[test]
fn missing_uncertainty_projection_fails_closed() {
    let mut value: Value =
        serde_json::from_str(&artifact_json(canonical_uncertainty())).expect("artifact fixture");
    value
        .as_object_mut()
        .expect("artifact object")
        .remove("diagonal_laplace_uncertainty");
    assert_eq!(
        TopicLineageArtifact::from_json(&value.to_string()),
        Err(AnalysisEngineError::InvalidTopicLineageArtifact)
    );
}

#[test]
fn malformed_or_detached_uncertainty_coordinates_fail_closed() {
    let canonical = canonical_uncertainty();
    let invalid = [
        json!([]),
        json!([canonical[0].clone()]),
        json!([canonical[0].clone(), canonical[0].clone()]),
        json!([canonical[1].clone(), canonical[0].clone()]),
        {
            let mut value = canonical.clone();
            value[1]["document_id"] = json!("00000000-0000-0000-0000-000000000003");
            value
        },
        {
            let mut value = canonical.clone();
            value[0]["coordinates"] = json!([]);
            value
        },
        {
            let mut value = canonical.clone();
            value[0]["coordinates"][0]["numerator_topic_index"] = json!(1);
            value
        },
        {
            let mut value = canonical.clone();
            value[0]["coordinates"][0]["reference_topic_index"] = json!(0);
            value
        },
        {
            let mut value = canonical.clone();
            value[0]["coordinates"][0]["variance"] = json!(0.0);
            value
        },
        {
            let mut value = canonical.clone();
            value[0]["coordinates"][0]["variance"] = json!(-0.5);
            value
        },
    ];

    for uncertainty in invalid {
        assert_eq!(
            TopicLineageArtifact::from_json(&artifact_json(uncertainty)),
            Err(AnalysisEngineError::InvalidTopicLineageArtifact)
        );
    }
}

#[test]
fn changing_uncertainty_changes_artifact_identity() {
    let first = TopicLineageArtifact::from_json(&artifact_json(canonical_uncertainty()))
        .expect("first artifact");
    let mut changed = canonical_uncertainty();
    changed[1]["coordinates"][0]["variance"] = json!(0.5);
    let second = TopicLineageArtifact::from_json(&artifact_json(changed)).expect("second artifact");

    assert_ne!(first.sha256().expect("first digest"), second.sha256().expect("second digest"));
}
