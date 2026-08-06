//! Cross-clock parity and nominal-type contracts.

use serde_json::json;
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
    TemporalClock, TemporalInstant,
};

fn reconstruct<T: TemporalClock>(instant: TemporalInstant) -> T {
    T::from_instant(instant)
}

fn extract<T: TemporalClock>(clock: T) -> TemporalInstant {
    TemporalClock::instant(clock)
}

macro_rules! assert_clock_contract {
    ($clock:ty, $wire_name:literal) => {{
        let value = <$clock>::parse_rfc3339("2026-08-06T10:30:00+09:00")
            .expect("clock value must parse");
        let wire = value.to_wire_json().expect("clock value must serialize");
        let restored = <$clock>::from_wire_json(&wire).expect("clock value must reconstruct");
        let schema = <$clock>::wire_json_schema();
        let rebuilt = reconstruct::<$clock>(value.instant());

        assert_eq!(value.to_rfc3339(), "2026-08-06T01:30:00Z");
        assert_eq!(restored, value);
        assert_eq!(rebuilt, value);
        assert_eq!(extract(value), value.instant());
        assert_eq!(schema["properties"]["clock_type"]["const"], json!($wire_name));
        assert_eq!(<$clock as TemporalClock>::WIRE_NAME, $wire_name);
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
