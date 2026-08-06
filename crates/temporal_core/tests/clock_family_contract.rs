//! Cross-clock parity and nominal-type contracts.

use serde_json::json;
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
    TemporalBoundary, TemporalClock, TemporalError, TemporalInstant, TemporalInterval,
    TemporalPrecision,
};

fn reconstruct<T: TemporalClock>(instant: TemporalInstant) -> T {
    T::from_instant(instant)
}

fn extract<T: TemporalClock>(clock: T) -> TemporalInstant {
    TemporalClock::instant(clock)
}

macro_rules! assert_clock_contract {
    ($clock:ty, $wire_name:literal) => {{
        let value =
            <$clock>::parse_rfc3339("2026-08-06T10:30:00+09:00").expect("clock value must parse");
        let wire = value.to_wire_json().expect("clock value must serialize");
        let restored = <$clock>::from_wire_json(&wire).expect("clock value must reconstruct");
        let schema = <$clock>::wire_json_schema();
        let rebuilt = reconstruct::<$clock>(value.instant());

        assert_eq!(value.to_rfc3339(), "2026-08-06T01:30:00Z");
        assert_eq!(restored, value);
        assert_eq!(rebuilt, value);
        assert_eq!(extract(value), value.instant());
        assert_eq!(
            schema["properties"]["clock_type"]["const"],
            json!($wire_name)
        );
        assert_eq!(<$clock as TemporalClock>::WIRE_NAME, $wire_name);
    }};
}

macro_rules! assert_interval_contract {
    ($clock:ty) => {{
        let before =
            <$clock>::parse_rfc3339("2025-12-31T23:59:59Z").expect("before value must parse");
        let lower =
            <$clock>::parse_rfc3339("2026-01-01T00:00:00Z").expect("lower value must parse");
        let middle =
            <$clock>::parse_rfc3339("2026-06-01T00:00:00Z").expect("middle value must parse");
        let upper =
            <$clock>::parse_rfc3339("2027-01-01T00:00:00Z").expect("upper value must parse");
        let after =
            <$clock>::parse_rfc3339("2027-01-01T00:00:01Z").expect("after value must parse");

        assert_eq!(
            TemporalInterval::<$clock>::bounded(
                TemporalBoundary::Unbounded,
                TemporalBoundary::Unbounded,
                TemporalPrecision::Year,
            ),
            Err(TemporalError::InvalidIntervalCertainty)
        );
        assert_eq!(
            TemporalInterval::<$clock>::bounded(
                TemporalBoundary::Included(upper),
                TemporalBoundary::Included(lower),
                TemporalPrecision::Year,
            ),
            Err(TemporalError::InvalidIntervalOrder)
        );
        assert_eq!(
            TemporalInterval::<$clock>::bounded(
                TemporalBoundary::Included(lower),
                TemporalBoundary::Excluded(lower),
                TemporalPrecision::Year,
            ),
            Err(TemporalError::EmptyInterval)
        );

        let bounded = TemporalInterval::<$clock>::bounded(
            TemporalBoundary::Included(lower),
            TemporalBoundary::Excluded(upper),
            TemporalPrecision::Year,
        )
        .expect("bounded interval must validate");
        assert!(!bounded.contains(before));
        assert!(bounded.contains(lower));
        assert!(bounded.contains(middle));
        assert!(!bounded.contains(upper));
        assert!(!bounded.contains(after));

        let exact = TemporalInterval::<$clock>::exact(middle, TemporalPrecision::Second)
            .expect("exact interval must validate");
        assert!(exact.contains(middle));
        assert!(!exact.contains(lower));

        let unknown = TemporalInterval::<$clock>::unknown();
        assert!(!unknown.contains(middle));
    }};
}

#[test]
fn every_nominal_clock_supports_the_same_validated_wire_and_schema_contract() {
    assert_clock_contract!(EventTime, "event_time");
    assert_clock_contract!(AssertionTime, "assertion_time");
    assert_clock_contract!(DocumentTime, "document_time");
    assert_clock_contract!(SystemTime, "system_time");
    assert_clock_contract!(AvailableTime, "available_time");
    assert_clock_contract!(KnowledgeCutoff, "knowledge_cutoff");
}

#[test]
fn every_nominal_clock_supports_the_same_interval_validation_contract() {
    assert_interval_contract!(EventTime);
    assert_interval_contract!(AssertionTime);
    assert_interval_contract!(DocumentTime);
    assert_interval_contract!(SystemTime);
    assert_interval_contract!(AvailableTime);
    assert_interval_contract!(KnowledgeCutoff);
}
