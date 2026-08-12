//! Typed clock, normalization, precision, certainty, and interval contracts.

use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
    TemporalBoundary, TemporalCertainty, TemporalError, TemporalInterval, TemporalPrecision,
};

#[test]
fn six_clocks_normalize_the_same_rfc3339_instant_without_losing_type() {
    let source = "2026-08-06T10:30:00+09:00";
    let event = EventTime::parse_rfc3339(source).expect("event time must parse");
    let assertion = AssertionTime::parse_rfc3339(source).expect("assertion time must parse");
    let document = DocumentTime::parse_rfc3339(source).expect("document time must parse");
    let system = SystemTime::parse_rfc3339(source).expect("system time must parse");
    let available = AvailableTime::parse_rfc3339(source).expect("available time must parse");
    let cutoff = KnowledgeCutoff::parse_rfc3339(source).expect("cutoff must parse");

    assert_eq!(event.to_rfc3339(), "2026-08-06T01:30:00Z");
    assert_eq!(assertion.to_rfc3339(), "2026-08-06T01:30:00Z");
    assert_eq!(document.to_rfc3339(), "2026-08-06T01:30:00Z");
    assert_eq!(system.to_rfc3339(), "2026-08-06T01:30:00Z");
    assert_eq!(available.to_rfc3339(), "2026-08-06T01:30:00Z");
    assert_eq!(cutoff.to_rfc3339(), "2026-08-06T01:30:00Z");
    assert_eq!(event.instant(), assertion.instant());
    assert_eq!(event.instant(), document.instant());
    assert_eq!(event.instant(), system.instant());
    assert_eq!(event.instant(), available.instant());
    assert_eq!(event.instant(), cutoff.instant());
}

#[test]
fn strict_rfc3339_rejects_ambiguous_or_lossy_inputs() {
    let invalid = [
        "2026-08-06",
        "2026-08-06T01:30:00",
        "2026-08-06 01:30:00Z",
        "2026-08-06T01:30Z",
        "2026-08-06T01:30:00-05",
        "2026-08-06T01:30:00+05:30:15",
        "2026-08-06T01:30:00Z[UTC]",
        "2026-08-06T01:30:60Z",
        "2026-02-30T01:30:00Z",
        "2026-08-06T25:30:00Z",
        "not-a-time",
    ];

    for value in invalid {
        assert_eq!(
            EventTime::parse_rfc3339(value).unwrap_err(),
            TemporalError::InvalidTimestamp
        );
    }
}

#[test]
fn daylight_saving_offsets_normalize_to_the_correct_elapsed_duration() {
    let before = EventTime::parse_rfc3339("2024-03-10T01:30:00-05:00")
        .expect("pre-transition time must parse");
    let after = EventTime::parse_rfc3339("2024-03-10T03:30:00-04:00")
        .expect("post-transition time must parse");

    assert_eq!(
        after.instant().as_nanosecond() - before.instant().as_nanosecond(),
        3_600_000_000_000
    );
    assert_eq!(before.to_rfc3339(), "2024-03-10T06:30:00Z");
    assert_eq!(after.to_rfc3339(), "2024-03-10T07:30:00Z");
}

#[test]
fn typed_clock_values_are_totally_ordered_by_normalized_instant() {
    let earlier =
        EventTime::parse_rfc3339("2026-08-06T00:00:00Z").expect("earlier event must parse");
    let later =
        EventTime::parse_rfc3339("2026-08-06T00:00:00.000000001Z").expect("later event must parse");

    assert!(earlier < later);
    assert_eq!(
        earlier.instant().as_nanosecond() + 1,
        later.instant().as_nanosecond()
    );
}

#[test]
fn exact_interval_contains_only_its_single_instant() {
    let value = EventTime::parse_rfc3339("2026-08-06T01:00:00Z").expect("event time must parse");
    let before = EventTime::parse_rfc3339("2026-08-06T00:59:59Z").expect("event time must parse");
    let after = EventTime::parse_rfc3339("2026-08-06T01:00:01Z").expect("event time must parse");
    let interval = TemporalInterval::exact(value, TemporalPrecision::Second)
        .expect("exact interval must validate");

    assert_eq!(interval.certainty(), TemporalCertainty::Exact);
    assert_eq!(interval.precision(), TemporalPrecision::Second);
    assert_eq!(interval.lower(), TemporalBoundary::Included(value));
    assert_eq!(interval.upper(), TemporalBoundary::Included(value));
    assert!(interval.contains(value));
    assert!(!interval.contains(before));
    assert!(!interval.contains(after));
    assert!(interval.is_known());
}

#[test]
fn half_open_bounded_interval_honors_inclusion_and_precision() {
    let start = EventTime::parse_rfc3339("2026-04-01T00:00:00Z").expect("quarter start must parse");
    let inside = EventTime::parse_rfc3339("2026-06-30T23:59:59Z").expect("inside value must parse");
    let end = EventTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("quarter end must parse");
    let interval = TemporalInterval::bounded(
        TemporalBoundary::Included(start),
        TemporalBoundary::Excluded(end),
        TemporalPrecision::Quarter,
    )
    .expect("bounded quarter must validate");

    assert_eq!(interval.certainty(), TemporalCertainty::Bounded);
    assert_eq!(interval.precision(), TemporalPrecision::Quarter);
    assert!(interval.contains(start));
    assert!(interval.contains(inside));
    assert!(!interval.contains(end));
}

#[test]
fn open_ended_intervals_apply_only_the_known_boundary() {
    let start = AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("start must parse");
    let later =
        AvailableTime::parse_rfc3339("2030-01-01T00:00:00Z").expect("later value must parse");
    let before =
        AvailableTime::parse_rfc3339("2025-12-31T23:59:59Z").expect("earlier value must parse");
    let interval = TemporalInterval::bounded(
        TemporalBoundary::Included(start),
        TemporalBoundary::Unbounded,
        TemporalPrecision::Day,
    )
    .expect("open-ended interval must validate");

    assert!(interval.contains(start));
    assert!(interval.contains(later));
    assert!(!interval.contains(before));
}

#[test]
fn invalid_bounded_intervals_fail_closed() {
    let first = EventTime::parse_rfc3339("2026-08-06T01:00:00Z").expect("first value must parse");
    let second = EventTime::parse_rfc3339("2026-08-06T02:00:00Z").expect("second value must parse");

    assert_eq!(
        TemporalInterval::bounded(
            TemporalBoundary::Included(second),
            TemporalBoundary::Included(first),
            TemporalPrecision::Hour,
        )
        .unwrap_err(),
        TemporalError::InvalidIntervalOrder
    );
    assert_eq!(
        TemporalInterval::bounded(
            TemporalBoundary::Included(first),
            TemporalBoundary::Excluded(first),
            TemporalPrecision::Hour,
        )
        .unwrap_err(),
        TemporalError::EmptyInterval
    );
    assert_eq!(
        TemporalInterval::bounded(
            TemporalBoundary::<EventTime>::Unbounded,
            TemporalBoundary::Unbounded,
            TemporalPrecision::Year,
        )
        .unwrap_err(),
        TemporalError::InvalidIntervalCertainty
    );
    assert_eq!(
        TemporalInterval::bounded(
            TemporalBoundary::Included(first),
            TemporalBoundary::Included(second),
            TemporalPrecision::Unknown,
        )
        .unwrap_err(),
        TemporalError::InvalidTemporalPrecision
    );
    assert_eq!(
        TemporalInterval::exact(first, TemporalPrecision::Unknown).unwrap_err(),
        TemporalError::InvalidTemporalPrecision
    );
}

#[test]
fn unknown_interval_is_explicit_and_does_not_claim_containment() {
    let interval = TemporalInterval::<DocumentTime>::unknown();
    let value =
        DocumentTime::parse_rfc3339("2026-08-06T01:00:00Z").expect("document time must parse");

    assert_eq!(interval.certainty(), TemporalCertainty::Unknown);
    assert_eq!(interval.precision(), TemporalPrecision::Unknown);
    assert_eq!(interval.lower(), TemporalBoundary::Unbounded);
    assert_eq!(interval.upper(), TemporalBoundary::Unbounded);
    assert!(!interval.contains(value));
    assert!(!interval.is_known());
}
