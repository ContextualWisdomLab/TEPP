//! Nonempty interval contracts at nanosecond resolution.

use temporal_core::{EventTime, TemporalBoundary, TemporalError, TemporalInterval, TemporalPrecision};

fn time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("test timestamp must parse")
}

#[test]
fn adjacent_excluded_nanosecond_bounds_are_empty() {
    let lower = time("2026-08-12T00:00:00.000000000Z");
    let upper = time("2026-08-12T00:00:00.000000001Z");

    assert_eq!(
        TemporalInterval::bounded(
            TemporalBoundary::Excluded(lower),
            TemporalBoundary::Excluded(upper),
            TemporalPrecision::Nanosecond,
        ),
        Err(TemporalError::EmptyInterval)
    );
}

#[test]
fn wire_reconstruction_rejects_adjacent_excluded_nanosecond_bounds() {
    let payload = r#"{
        "schema_version": 1,
        "clock_type": "event_time",
        "certainty": "bounded",
        "precision": "nanosecond",
        "lower": {"kind": "excluded", "timestamp": "2026-08-12T00:00:00.000000000Z"},
        "upper": {"kind": "excluded", "timestamp": "2026-08-12T00:00:00.000000001Z"}
    }"#;

    assert_eq!(
        TemporalInterval::<EventTime>::from_wire_json(payload),
        Err(TemporalError::EmptyInterval)
    );
}
