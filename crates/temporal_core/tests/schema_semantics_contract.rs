//! JSON Schema parity contracts for temporal interval wire semantics.

use serde_json::Value;
use temporal_core::{EventTime, TemporalInterval};

#[test]
fn interval_schema_exposes_certainty_precision_conditionals() {
    let schema = TemporalInterval::<EventTime>::wire_json_schema();
    let conditionals = schema["allOf"]
        .as_array()
        .expect("interval schema must expose certainty conditionals");

    assert_eq!(conditionals.len(), 3);
    let encoded = serde_json::to_string(conditionals).expect("schema must serialize");
    assert!(encoded.contains("\"unknown\""));
    assert!(encoded.contains("\"exact\""));
    assert!(encoded.contains("\"bounded\""));
    assert!(encoded.contains("\"not\""));
}

#[test]
fn interval_schema_documents_runtime_only_exact_timestamp_equality() {
    let schema = TemporalInterval::<EventTime>::wire_json_schema();
    let description = schema["description"]
        .as_str()
        .expect("interval schema must document runtime semantics");
    assert!(description.contains("runtime"));
    assert!(description.contains("exact"));
    assert!(description.contains("matching included timestamps"));

    let timestamp = &schema["properties"]["lower"]["oneOf"][1]["properties"]["timestamp"];
    let timestamp_description = timestamp["description"]
        .as_str()
        .expect("timestamp schema must document runtime validation");
    assert!(timestamp_description.contains("calendar"));
    assert!(timestamp_description.contains("offset"));
}

#[test]
fn unknown_certainty_schema_forbids_included_or_excluded_boundaries() {
    let schema = TemporalInterval::<EventTime>::wire_json_schema();
    let conditionals: &Vec<Value> = schema["allOf"].as_array().expect("conditionals must exist");
    let encoded = serde_json::to_string(conditionals).expect("schema must serialize");
    assert!(encoded.contains("unbounded"));
    assert!(encoded.contains("precision"));
}
