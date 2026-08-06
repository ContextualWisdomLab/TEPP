//! Strict versioned JSON wire and JSON Schema contracts for temporal values.

use serde_json::{Value, json};
use temporal_core::{
    EventTime, TEMPORAL_WIRE_SCHEMA_VERSION, TemporalBoundary, TemporalError, TemporalInterval,
    TemporalPrecision,
};

fn replace_field(serialized: &str, field: &str, replacement: Value) -> String {
    let mut value: Value = serde_json::from_str(serialized).expect("wire JSON must parse");
    value[field] = replacement;
    serde_json::to_string(&value).expect("tampered JSON must serialize")
}

#[test]
fn typed_clock_wire_round_trip_preserves_clock_and_normalized_instant() {
    let value =
        EventTime::parse_rfc3339("2026-08-06T10:30:00+09:00").expect("event time must parse");
    let serialized = value.to_wire_json().expect("clock must serialize");
    let json_value: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    let restored = EventTime::from_wire_json(&serialized).expect("clock must reconstruct");

    assert_eq!(
        json_value["schema_version"],
        json!(TEMPORAL_WIRE_SCHEMA_VERSION)
    );
    assert_eq!(json_value["clock_type"], json!("event_time"));
    assert_eq!(json_value["timestamp"], json!("2026-08-06T01:30:00Z"));
    assert_eq!(restored, value);
}

#[test]
fn typed_clock_wire_rejects_unknown_fields_versions_types_and_invalid_timestamps() {
    let value = EventTime::parse_rfc3339("2026-08-06T01:30:00Z").expect("event time must parse");
    let serialized = value.to_wire_json().expect("clock must serialize");

    let unsupported = replace_field(&serialized, "schema_version", json!(2));
    assert_eq!(
        EventTime::from_wire_json(&unsupported).unwrap_err(),
        TemporalError::UnsupportedWireVersion
    );

    let wrong_clock = replace_field(&serialized, "clock_type", json!("document_time"));
    assert_eq!(
        EventTime::from_wire_json(&wrong_clock).unwrap_err(),
        TemporalError::ClockTypeMismatch
    );

    let invalid_timestamp = replace_field(&serialized, "timestamp", json!("2026-08-06"));
    assert_eq!(
        EventTime::from_wire_json(&invalid_timestamp).unwrap_err(),
        TemporalError::InvalidTimestamp
    );

    let mut unknown: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    unknown["timezone"] = json!("Asia/Seoul");
    assert_eq!(
        EventTime::from_wire_json(&unknown.to_string()).unwrap_err(),
        TemporalError::InvalidWirePayload
    );

    assert_eq!(
        EventTime::from_wire_json("not JSON").unwrap_err(),
        TemporalError::InvalidWirePayload
    );
}

#[test]
fn bounded_interval_wire_round_trip_preserves_boundaries_precision_and_certainty() {
    let start = EventTime::parse_rfc3339("2026-04-01T00:00:00Z").expect("start must parse");
    let end = EventTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("end must parse");
    let interval = TemporalInterval::bounded(
        TemporalBoundary::Included(start),
        TemporalBoundary::Excluded(end),
        TemporalPrecision::Quarter,
    )
    .expect("interval must validate");
    let serialized = interval.to_wire_json().expect("interval must serialize");
    let json_value: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    let restored = TemporalInterval::<EventTime>::from_wire_json(&serialized)
        .expect("interval must reconstruct");

    assert_eq!(json_value["clock_type"], json!("event_time"));
    assert_eq!(json_value["certainty"], json!("bounded"));
    assert_eq!(json_value["precision"], json!("quarter"));
    assert_eq!(json_value["lower"]["kind"], json!("included"));
    assert_eq!(json_value["upper"]["kind"], json!("excluded"));
    assert_eq!(restored, interval);
}

#[test]
fn exact_and_unknown_intervals_round_trip_with_distinct_semantics() {
    let value = EventTime::parse_rfc3339("2026-08-06T01:00:00Z").expect("event time must parse");
    let exact = TemporalInterval::exact(value, TemporalPrecision::Second)
        .expect("exact interval must validate");
    let unknown = TemporalInterval::<EventTime>::unknown();

    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(
            &exact.to_wire_json().expect("exact interval must serialize")
        )
        .expect("exact interval must reconstruct"),
        exact
    );
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(
            &unknown
                .to_wire_json()
                .expect("unknown interval must serialize")
        )
        .expect("unknown interval must reconstruct"),
        unknown
    );
}

#[test]
fn interval_wire_rejects_unknown_fields_wrong_clocks_and_invalid_semantics() {
    let start = EventTime::parse_rfc3339("2026-04-01T00:00:00Z").expect("start must parse");
    let end = EventTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("end must parse");
    let interval = TemporalInterval::bounded(
        TemporalBoundary::Included(start),
        TemporalBoundary::Excluded(end),
        TemporalPrecision::Quarter,
    )
    .expect("interval must validate");
    let serialized = interval.to_wire_json().expect("interval must serialize");

    let unsupported = replace_field(&serialized, "schema_version", json!(3));
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&unsupported).unwrap_err(),
        TemporalError::UnsupportedWireVersion
    );

    let wrong_clock = replace_field(&serialized, "clock_type", json!("available_time"));
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&wrong_clock).unwrap_err(),
        TemporalError::ClockTypeMismatch
    );

    let reversed = replace_field(
        &serialized,
        "lower",
        json!({"kind":"included","timestamp":"2027-01-01T00:00:00Z"}),
    );
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&reversed).unwrap_err(),
        TemporalError::InvalidIntervalOrder
    );

    let unknown_precision = replace_field(&serialized, "precision", json!("unknown"));
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&unknown_precision).unwrap_err(),
        TemporalError::InvalidTemporalPrecision
    );

    let mismatched_certainty = replace_field(&serialized, "certainty", json!("exact"));
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&mismatched_certainty).unwrap_err(),
        TemporalError::InvalidIntervalCertainty
    );

    let mut unknown_field: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    unknown_field["lower"]["inclusive"] = json!(true);
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&unknown_field.to_string()).unwrap_err(),
        TemporalError::InvalidWirePayload
    );
}

#[test]
fn interval_wire_rejects_malformed_boundary_payloads() {
    let value = EventTime::parse_rfc3339("2026-08-06T01:00:00Z").expect("event time must parse");
    let interval = TemporalInterval::exact(value, TemporalPrecision::Second)
        .expect("exact interval must validate");
    let serialized = interval.to_wire_json().expect("interval must serialize");

    let missing_timestamp = replace_field(&serialized, "lower", json!({"kind":"included"}));
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&missing_timestamp).unwrap_err(),
        TemporalError::InvalidWirePayload
    );

    let unbounded_with_timestamp = replace_field(
        &serialized,
        "lower",
        json!({"kind":"unbounded","timestamp":"2026-08-06T01:00:00Z"}),
    );
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&unbounded_with_timestamp).unwrap_err(),
        TemporalError::InvalidWirePayload
    );

    let invalid_kind = replace_field(&serialized, "lower", json!({"kind":"closed"}));
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&invalid_kind).unwrap_err(),
        TemporalError::InvalidWirePayload
    );

    let invalid_upper_timestamp = replace_field(
        &serialized,
        "upper",
        json!({"kind":"included","timestamp":"2026-08-06"}),
    );
    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(&invalid_upper_timestamp).unwrap_err(),
        TemporalError::InvalidTimestamp
    );
}

#[test]
fn temporal_json_schemas_are_draft_2020_12_and_clock_specific() {
    let clock_schema = EventTime::wire_json_schema();
    let interval_schema = TemporalInterval::<EventTime>::wire_json_schema();

    assert_eq!(
        clock_schema["$schema"],
        json!("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(clock_schema["additionalProperties"], json!(false));
    assert_eq!(
        clock_schema["properties"]["clock_type"]["const"],
        json!("event_time")
    );
    assert_eq!(
        clock_schema["properties"]["timestamp"]["format"],
        json!("date-time")
    );
    assert_eq!(
        interval_schema["$schema"],
        json!("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(interval_schema["additionalProperties"], json!(false));
    assert_eq!(
        interval_schema["properties"]["clock_type"]["const"],
        json!("event_time")
    );
    assert_eq!(
        interval_schema["properties"]["precision"]["enum"],
        json!([
            "nanosecond",
            "microsecond",
            "millisecond",
            "second",
            "minute",
            "hour",
            "day",
            "month",
            "quarter",
            "year",
            "unknown"
        ])
    );
}

#[test]
fn temporal_wire_errors_have_stable_redacted_messages() {
    let cases = [
        (
            TemporalError::InvalidTimestamp,
            "invalid temporal timestamp",
        ),
        (
            TemporalError::InvalidWirePayload,
            "invalid temporal wire payload",
        ),
        (
            TemporalError::UnsupportedWireVersion,
            "unsupported temporal wire version",
        ),
        (
            TemporalError::ClockTypeMismatch,
            "temporal clock type mismatch",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
