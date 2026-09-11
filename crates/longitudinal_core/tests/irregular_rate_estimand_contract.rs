//! Public contract for pair-weighted irregular-rate estimand identity.

use longitudinal_core::{
    EventTimedObservation, IrregularRateEstimand, LongitudinalError,
    recover_within_unit_irregular_rate_summary, recover_within_unit_irregular_residual_log_rate,
};

fn timed(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, score)
}

fn unequal_pair_count_rows() -> [EventTimedObservation; 7] {
    [
        timed(1, 0.0, -3.0),
        timed(1, 1.0, -2.0),
        timed(1, 2.0, -1.0),
        timed(1, 3.0, 6.0),
        timed(2, 0.0, 3.0),
        timed(2, 1.0, 1.0),
        timed(2, 2.0, -4.0),
    ]
}

#[test]
fn lag_pair_average_reports_estimand_and_failure_denominators() {
    let rows = unequal_pair_count_rows();
    let summary = recover_within_unit_irregular_rate_summary(
        &rows,
        IrregularRateEstimand::LagPairAverageV1,
    )
    .expect("pair-average evidence");

    assert_eq!(
        summary.estimand().wire_name(),
        "tepp.irregular_rate.lag_pair_average.v1"
    );
    assert_eq!(summary.candidate_units(), 2);
    assert_eq!(summary.contributing_units(), 2);
    assert_eq!(summary.candidate_pairs(), 5);
    assert_eq!(summary.admitted_pairs(), 3);
    assert_eq!(summary.refused_pairs(), 2);
    assert_eq!(
        summary.admitted_pairs() + summary.refused_pairs(),
        summary.candidate_pairs()
    );

    let pair_average = summary.estimate().expect("three admitted rates");
    let legacy = recover_within_unit_irregular_residual_log_rate(&rows).expect("legacy pair mean");
    assert_eq!(pair_average.to_bits(), legacy.to_bits());

    let unit_one = ((2.0_f64 / 3.0).ln() + (1.0_f64 / 2.0).ln()) / 2.0;
    let unit_two = (1.0_f64 / 3.0).ln();
    let equal_unit_comparison = (unit_one + unit_two) / 2.0;
    assert!(
        (pair_average - equal_unit_comparison).abs() > 1.0e-3,
        "unequal admitted-pair counts must make pair and equal-unit targets distinguishable"
    );
}

#[test]
fn lag_pair_average_is_invariant_to_input_row_permutation() {
    let canonical = unequal_pair_count_rows();
    let shuffled = [
        canonical[5],
        canonical[2],
        canonical[6],
        canonical[0],
        canonical[3],
        canonical[4],
        canonical[1],
    ];

    let expected = recover_within_unit_irregular_rate_summary(
        &canonical,
        IrregularRateEstimand::LagPairAverageV1,
    )
    .expect("canonical");
    let reordered = recover_within_unit_irregular_rate_summary(
        &shuffled,
        IrregularRateEstimand::LagPairAverageV1,
    )
    .expect("shuffled");

    assert_eq!(reordered, expected);
}

#[test]
fn zero_admissible_rates_still_return_explicit_pair_denominator() {
    let rows = [
        timed(1, 0.0, 1.0),
        timed(1, 1.0, -1.0),
        timed(2, 0.0, 2.0),
        timed(2, 1.0, -2.0),
    ];
    let summary = recover_within_unit_irregular_rate_summary(
        &rows,
        IrregularRateEstimand::LagPairAverageV1,
    )
    .expect("denominator evidence survives zero admissible rates");

    assert_eq!(summary.estimate(), None);
    assert_eq!(summary.candidate_units(), 2);
    assert_eq!(summary.contributing_units(), 0);
    assert_eq!(summary.candidate_pairs(), 2);
    assert_eq!(summary.admitted_pairs(), 0);
    assert_eq!(summary.refused_pairs(), 2);
}

#[test]
fn unit_average_is_versioned_but_fails_closed_until_owner_mean_release() {
    let rows = unequal_pair_count_rows();
    assert_eq!(
        IrregularRateEstimand::UnitAverageV1.wire_name(),
        "tepp.irregular_rate.unit_average.v1"
    );
    assert_eq!(
        recover_within_unit_irregular_rate_summary(&rows, IrregularRateEstimand::UnitAverageV1),
        Err(LongitudinalError::InvalidTemporalTransformInput)
    );
}
