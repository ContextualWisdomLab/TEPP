//! Edge cases for temporal interval wire reconstruction.

use serde_json::{Value, json};
use temporal_core::{EventTime, TemporalError, TemporalInterval, TemporalPrecision};

fn mutate(serialized: &str, field: &str, replacement: Value) -> String {
    let mut value: Value = serde_json::from_str(serialized).expect("wire JSON must parse");
    value[field] = replacement;
    serde_json::to_string(&value).expect("tampered JSON must serialize")
}

#[test]
fn exact_interval_round_trip_executes_edge_binary_reconstruction() {
    let value =
        EventTime::parse_rfc3339("2026-08-06T00:00:00Z").expect("event time must parse");
    let interval = TemporalInterval::exact(value, TemporalPrecision::Second)
        .expect("exact interval must validate");
    let serialized = interval
        .to_wire_json()
        .expect("validated exact interval must serialize");

    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&serialized)
            .expect("exact interval must reconstruct"),
        interval
    );
}

#[test]
fn unknown_certainty_rejects_known_precision_or_known_boundaries() {
    let unknown = TemporalInterval::<EventTime>::unknown();
    let serialized = unknown
        .to_wire_json()
        .expect("validated unknown interval must serialize");

    let known_precision = mutate(&serialized, "precision", json!("day"));
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&known_precision).unwrap_err(),
        TemporalError::InvalidIntervalCertainty
    );

    let known_lower = mutate(
        &serialized,
        "lower",
        json!({"kind":"included","timestamp":"2026-08-06T00:00:00Z"}),
    );
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&known_lower).unwrap_err(),
        TemporalError::InvalidIntervalCertainty
    );

    let known_upper = mutate(
        &serialized,
        "upper",
        json!({"kind":"included","timestamp":"2026-08-06T00:00:00Z"}),
    );
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&known_upper).unwrap_err(),
        TemporalError::InvalidIntervalCertainty
    );
}

#[test]
fn excluded_boundary_requires_a_timestamp() {
    let unknown = TemporalInterval::<EventTime>::unknown();
    let serialized = unknown
        .to_wire_json()
        .expect("validated unknown interval must serialize");
    let malformed = mutate(&serialized, "lower", json!({"kind":"excluded"}));

    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&malformed).unwrap_err(),
        TemporalError::InvalidWirePayload
    );
}
