//! Deterministic public contract for pair-weighted irregular-rate estimand identity.
//!
//! Realistic rate-associated missingness and Monte Carlo uncertainty are exercised
//! separately in `irregular_rate_monte_carlo_contract.rs`. This file keeps only
//! deterministic estimand, weighting, refusal, and fail-closed fixtures.

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
    let summary =
        recover_within_unit_irregular_rate_summary(&rows, IrregularRateEstimand::LagPairAverageV1)
            .expect("pair-average evidence");

    assert_eq!(
        summary.estimand().wire_name(),
        "tepp.irregular_rate.lag_pair_average.v1"
    );
    assert_eq!(summary.candidate_units(), 2);
    assert_eq!(summary.contributing_units(), 2);
    assert_eq!(summary.candidate_pairs(), 5);
    assert_eq!(summary.admitted_pairs(), 3);
    assert_eq!(summary.sign_or_zero_refused_pairs(), 2);
    assert_eq!(summary.nonrepresentable_rate_refused_pairs(), 0);
    assert_eq!(summary.refused_pairs(), 2);
    assert_eq!(
        summary.admitted_pairs() + summary.refused_pairs(),
        summary.candidate_pairs()
    );

    let pair_average = summary.estimate().expect("three admitted rates");
    let legacy = recover_within_unit_irregular_residual_log_rate(&rows).expect("legacy pair mean");
    assert_eq!(pair_average.to_bits(), legacy.to_bits());

    let unit_one = f64::midpoint((2.0_f64 / 3.0).ln(), (1.0_f64 / 2.0).ln());
    let unit_two = (1.0_f64 / 3.0).ln();
    let equal_unit_comparison = f64::midpoint(unit_one, unit_two);
    assert!(
        (pair_average - equal_unit_comparison).abs() > 1.0e-3,
        "unequal admitted-pair counts must make pair and equal-unit targets distinguishable"
    );
}

#[test]
fn extra_admitted_pair_changes_pair_weight_without_changing_unit_target() {
    let base = unequal_pair_count_rows();
    let extended = [
        base[0],
        base[1],
        base[2],
        base[3],
        base[4],
        base[5],
        base[6],
        timed(2, 3.0, -4.0 / 3.0),
        timed(2, 4.0, 4.0 / 3.0),
    ];

    let base_summary =
        recover_within_unit_irregular_rate_summary(&base, IrregularRateEstimand::LagPairAverageV1)
            .expect("base pair-average evidence");
    let extended_summary = recover_within_unit_irregular_rate_summary(
        &extended,
        IrregularRateEstimand::LagPairAverageV1,
    )
    .expect("extended pair-average evidence");

    assert_eq!(base_summary.admitted_pairs(), 3);
    assert_eq!(extended_summary.admitted_pairs(), 4);
    assert_eq!(base_summary.candidate_pairs(), 5);
    assert_eq!(extended_summary.candidate_pairs(), 7);
    assert_eq!(base_summary.refused_pairs(), 2);
    assert_eq!(extended_summary.refused_pairs(), 3);

    let base_pair_average = base_summary.estimate().expect("base estimate");
    let extended_pair_average = extended_summary.estimate().expect("extended estimate");
    assert!(
        extended_pair_average < base_pair_average,
        "an extra admitted ln(1/3) rate must shift the pair-weighted target toward unit two"
    );
    assert!(
        (extended_pair_average - base_pair_average).abs() > 0.05,
        "the multiplicity perturbation must remain scientifically visible"
    );

    let unit_one = f64::midpoint((2.0_f64 / 3.0).ln(), (1.0_f64 / 2.0).ln());
    let unit_two_base = (1.0_f64 / 3.0).ln();
    let unit_two_extended = f64::midpoint(unit_two_base, unit_two_base);
    assert_eq!(unit_two_extended.to_bits(), unit_two_base.to_bits());

    let equal_unit_base = f64::midpoint(unit_one, unit_two_base);
    let equal_unit_extended = f64::midpoint(unit_one, unit_two_extended);
    assert_eq!(equal_unit_extended.to_bits(), equal_unit_base.to_bits());
}

#[test]
fn balanced_follow_up_recovers_declared_pair_weighted_truth() {
    let rows = [
        timed(1, 0.0, -8.0),
        timed(1, 1.0, -4.0),
        timed(1, 2.0, -2.0),
        timed(1, 3.0, 14.0),
        timed(2, 0.0, -27.0),
        timed(2, 1.0, -9.0),
        timed(2, 2.0, -3.0),
        timed(2, 3.0, 39.0),
    ];
    let summary =
        recover_within_unit_irregular_rate_summary(&rows, IrregularRateEstimand::LagPairAverageV1)
            .expect("balanced known-truth evidence");

    assert_eq!(summary.candidate_units(), 2);
    assert_eq!(summary.contributing_units(), 2);
    assert_eq!(summary.candidate_pairs(), 6);
    assert_eq!(summary.admitted_pairs(), 4);
    assert_eq!(summary.refused_pairs(), 2);

    let half_rate = 0.5_f64.ln();
    let third_rate = (1.0_f64 / 3.0).ln();
    let expected = f64::midpoint(half_rate, third_rate);
    let actual = summary.estimate().expect("balanced pair target");
    assert!(
        (actual - expected).abs() <= 8.0 * f64::EPSILON,
        "balanced follow-up must recover the equal pair-weighted known truth"
    );
}

#[test]
fn rate_associated_highly_unbalanced_follow_up_recovers_pair_weighted_truth() {
    let mut rows = Vec::with_capacity(21);
    let mut event_time = 0.0;
    for exponent in (1..=16).rev() {
        rows.push(timed(1, event_time, -2.0_f64.powi(exponent)));
        event_time += 1.0;
    }
    rows.push(timed(1, event_time, 131_070.0));
    rows.extend([
        timed(2, 0.0, -27.0),
        timed(2, 1.0, -9.0),
        timed(2, 2.0, -3.0),
        timed(2, 3.0, 39.0),
    ]);

    let summary =
        recover_within_unit_irregular_rate_summary(&rows, IrregularRateEstimand::LagPairAverageV1)
            .expect("rate-associated follow-up evidence");

    assert_eq!(summary.candidate_units(), 2);
    assert_eq!(summary.contributing_units(), 2);
    assert_eq!(summary.candidate_pairs(), 19);
    assert_eq!(summary.admitted_pairs(), 17);
    assert_eq!(summary.refused_pairs(), 2);

    let half_rate = 0.5_f64.ln();
    let third_rate = (1.0_f64 / 3.0).ln();
    let expected_pair_target = (15.0 * half_rate + 2.0 * third_rate) / 17.0;
    let equal_unit_target = f64::midpoint(half_rate, third_rate);
    let actual = summary.estimate().expect("unbalanced pair target");

    assert!(
        (actual - expected_pair_target).abs() <= 16.0 * f64::EPSILON,
        "declared pair weighting must recover the known 15:2 follow-up target"
    );
    assert!(
        (actual - equal_unit_target).abs() > 0.1,
        "rate-associated follow-up must not silently masquerade as equal-unit weighting"
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
    let summary =
        recover_within_unit_irregular_rate_summary(&rows, IrregularRateEstimand::LagPairAverageV1)
            .expect("denominator evidence survives zero admissible rates");

    assert_eq!(summary.estimate(), None);
    assert_eq!(summary.candidate_units(), 2);
    assert_eq!(summary.contributing_units(), 0);
    assert_eq!(summary.candidate_pairs(), 2);
    assert_eq!(summary.admitted_pairs(), 0);
    assert_eq!(summary.sign_or_zero_refused_pairs(), 2);
    assert_eq!(summary.nonrepresentable_rate_refused_pairs(), 0);
    assert_eq!(summary.refused_pairs(), 2);
}

#[test]
fn nonrepresentable_same_sign_rate_has_its_own_failure_denominator() {
    let adjacent_one = f64::from_bits(1.0_f64.to_bits() + 1);
    let rows = [
        timed(1, -f64::MAX, 1.0),
        timed(1, 0.0, adjacent_one),
        timed(1, f64::MAX, -2.0),
        timed(2, 0.0, 3.0),
        timed(2, 1.0, 1.0),
        timed(2, 2.0, -4.0),
    ];

    let summary =
        recover_within_unit_irregular_rate_summary(&rows, IrregularRateEstimand::LagPairAverageV1)
            .expect("pair-level transform refusal remains reportable evidence");

    assert_eq!(summary.candidate_units(), 2);
    assert_eq!(summary.candidate_pairs(), 4);
    assert_eq!(summary.admitted_pairs(), 1);
    assert_eq!(summary.sign_or_zero_refused_pairs(), 2);
    assert_eq!(summary.nonrepresentable_rate_refused_pairs(), 1);
    assert_eq!(summary.refused_pairs(), 3);
    assert_eq!(summary.contributing_units(), 1);
    assert!(summary.estimate().is_some());
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
        Err(LongitudinalError::IrregularRateEstimandUnavailable)
    );
}
