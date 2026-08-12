//! Nonempty interval contracts at nanosecond resolution.

use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
    TemporalBoundary, TemporalClock, TemporalError, TemporalInstant, TemporalInterval,
    TemporalPrecision,
};

fn assert_adjacent_excluded_bounds_are_empty<T: TemporalClock>() {
    let lower = TemporalInstant::parse_rfc3339("2026-08-12T00:00:00.000000000Z")
        .expect("lower test timestamp must parse");
    let upper = TemporalInstant::parse_rfc3339("2026-08-12T00:00:00.000000001Z")
        .expect("upper test timestamp must parse");
    let separated_upper = TemporalInstant::parse_rfc3339("2026-08-12T00:00:00.000000002Z")
        .expect("separated upper test timestamp must parse");

    assert_eq!(
        TemporalInterval::<T>::bounded(
            TemporalBoundary::Excluded(T::from_instant(lower)),
            TemporalBoundary::Excluded(T::from_instant(upper)),
            TemporalPrecision::Nanosecond,
        ),
        Err(TemporalError::EmptyInterval)
    );
    assert!(
        TemporalInterval::<T>::bounded(
            TemporalBoundary::Excluded(T::from_instant(lower)),
            TemporalBoundary::Excluded(T::from_instant(separated_upper)),
            TemporalPrecision::Nanosecond,
        )
        .is_ok()
    );
}

#[test]
fn adjacent_excluded_nanosecond_bounds_are_empty() {
    assert_adjacent_excluded_bounds_are_empty::<EventTime>();
    assert_adjacent_excluded_bounds_are_empty::<AssertionTime>();
    assert_adjacent_excluded_bounds_are_empty::<DocumentTime>();
    assert_adjacent_excluded_bounds_are_empty::<SystemTime>();
    assert_adjacent_excluded_bounds_are_empty::<AvailableTime>();
    assert_adjacent_excluded_bounds_are_empty::<KnowledgeCutoff>();
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
