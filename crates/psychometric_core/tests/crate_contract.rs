//! Integration contract for the `psychometric_core` package identity.

use psychometric_core::LagClock;

#[test]
fn package_identity_is_stable() {
    let observed = std::hint::black_box(env!("CARGO_PKG_NAME"));
    assert_eq!(observed, "psychometric_core");
}

#[test]
fn lag_clock_wire_names_are_stable() {
    for (clock, name) in [
        (LagClock::EventTime, "event_time"),
        (LagClock::SystemTime, "system_time"),
        (LagClock::AssertionTime, "assertion_time"),
        (LagClock::DocumentTime, "document_time"),
        (LagClock::AvailabilityTime, "availability_time"),
        (LagClock::KnowledgeCutoff, "knowledge_cutoff"),
    ] {
        assert_eq!(std::hint::black_box(clock).as_str(), name);
    }
}
