//! Interval-aware historical eligibility against a knowledge cutoff.

use temporal_core::{
    AvailableTime, KnowledgeCutoff, TemporalBoundary, TemporalError, TemporalInterval,
    TemporalPrecision, evaluate_historical_eligibility,
};

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn cutoff(stamp: &str) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(stamp).expect("cutoff")
}

fn exact(stamp: &str) -> TemporalInterval<AvailableTime> {
    TemporalInterval::exact(available(stamp), TemporalPrecision::Second).expect("exact")
}

/// Independently compute the latest representable availability nanosecond.
///
/// The production gate must agree with this comparison: eligible iff the
/// latest possible availability instant is `<=` the cutoff. Unknown or
/// open-ended availability has no latest instant and must fail closed.
fn latest_possible_ns(
    availability: &TemporalInterval<AvailableTime>,
) -> Result<i128, TemporalError> {
    if !availability.is_known() {
        return Err(TemporalError::UncertainAvailability);
    }
    match availability.upper() {
        TemporalBoundary::Unbounded => Err(TemporalError::UncertainAvailability),
        TemporalBoundary::Included(value) => Ok(value.instant().as_nanosecond()),
        TemporalBoundary::Excluded(value) => Ok(value.instant().as_nanosecond() - 1),
    }
}

fn expected_decision(
    availability: &TemporalInterval<AvailableTime>,
    knowledge_cutoff: &KnowledgeCutoff,
) -> Result<(), TemporalError> {
    match latest_possible_ns(availability) {
        Ok(latest) if latest <= knowledge_cutoff.instant().as_nanosecond() => Ok(()),
        Ok(_) => Err(TemporalError::IneligibleAtCutoff),
        Err(error) => Err(error),
    }
}

#[test]
fn computed_latest_instant_agrees_with_the_eligibility_gate() {
    let cut = cutoff("2026-06-01T00:00:00Z");
    let closed = |start: &str, end: &str| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(available(start)),
            TemporalBoundary::Included(available(end)),
            TemporalPrecision::Second,
        )
        .expect("closed")
    };
    let upper_open = |end: &str| {
        TemporalInterval::bounded(
            TemporalBoundary::Unbounded,
            TemporalBoundary::Excluded(available(end)),
            TemporalPrecision::Second,
        )
        .expect("upper open")
    };
    let lower_open = |start: &str| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(available(start)),
            TemporalBoundary::Unbounded,
            TemporalPrecision::Second,
        )
        .expect("lower open")
    };

    let cases = [
        exact("2026-06-01T00:00:00Z"),
        exact("2026-05-01T00:00:00Z"),
        exact("2026-06-01T00:00:01Z"),
        closed("2026-01-01T00:00:00Z", "2026-06-01T00:00:00Z"),
        closed("2026-01-01T00:00:00Z", "2026-06-01T00:00:01Z"),
        upper_open("2026-06-01T00:00:00Z"),
        upper_open("2026-06-01T00:00:00.000000001Z"),
        upper_open("2026-06-01T00:00:00.000000002Z"),
        lower_open("2026-01-01T00:00:00Z"),
        TemporalInterval::<AvailableTime>::unknown(),
    ];

    for availability in cases {
        assert_eq!(
            evaluate_historical_eligibility(&availability, &cut),
            expected_decision(&availability, &cut)
        );
    }
}

#[test]
fn unknown_and_open_ended_availability_fail_closed() {
    let cut = cutoff("2026-06-01T00:00:00Z");
    assert_eq!(
        evaluate_historical_eligibility(&TemporalInterval::unknown(), &cut),
        Err(TemporalError::UncertainAvailability)
    );
    let open_upper = TemporalInterval::bounded(
        TemporalBoundary::Included(available("2026-01-01T00:00:00Z")),
        TemporalBoundary::Unbounded,
        TemporalPrecision::Day,
    )
    .expect("open upper");
    assert_eq!(
        evaluate_historical_eligibility(&open_upper, &cut),
        Err(TemporalError::UncertainAvailability)
    );
}

#[test]
fn exact_availability_after_cutoff_is_ineligible() {
    let cut = cutoff("2026-06-01T00:00:00Z");
    assert_eq!(
        evaluate_historical_eligibility(&exact("2026-06-01T00:00:00Z"), &cut),
        Ok(())
    );
    assert_eq!(
        evaluate_historical_eligibility(&exact("2026-06-01T00:00:01Z"), &cut),
        Err(TemporalError::IneligibleAtCutoff)
    );
}
