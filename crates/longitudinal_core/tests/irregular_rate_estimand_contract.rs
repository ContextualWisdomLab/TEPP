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

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }
}

fn append_missingness_unit(
    rows: &mut Vec<EventTimedObservation>,
    rng: &mut SplitMix64,
    unit: u32,
    log_rate: f64,
    retain_three_of_four: bool,
) -> usize {
    let mut event_time = 0.0;
    let first_score = -1.0;
    let mut negative_sum = first_score;
    let mut negative_count = 1_usize;
    rows.push(timed(unit, event_time, first_score));

    for _ in 1..12 {
        event_time += 1.0;
        let draw = rng.next_u64();
        let retain = if retain_three_of_four {
            draw & 3 != 0
        } else {
            draw & 3 == 0
        };
        if retain {
            let score = -(log_rate * event_time).exp();
            rows.push(timed(unit, event_time, score));
            negative_sum += score;
            negative_count += 1;
        }
    }

    event_time += 1.0;
    let endpoint_score = -(log_rate * event_time).exp();
    rows.push(timed(unit, event_time, endpoint_score));
    negative_sum += endpoint_score;
    negative_count += 1;

    rows.push(timed(unit, event_time + 1.0, -negative_sum));
    negative_count - 1
}

#[derive(Clone, Copy, Debug)]
struct MissingnessRecoveryEvidence {
    attempted_replicates: usize,
    recovered_replicates: usize,
    failed_replicates: usize,
    half_rate_admitted_pairs: usize,
    third_rate_admitted_pairs: usize,
    bias: f64,
    rmse: f64,
    max_absolute_error: f64,
    first_quarter_rmse: f64,
    recovery_coverage: f64,
    coverage_wilson_lower_95: f64,
    bias_standard_error: f64,
}

fn run_rate_associated_missingness_recovery(seed: u64) -> MissingnessRecoveryEvidence {
    const REPLICATES: usize = 256;
    const REPLICATES_F64: f64 = 256.0;
    const FIRST_QUARTER_REPLICATES: usize = 64;
    const FIRST_QUARTER_REPLICATES_F64: f64 = 64.0;
    const RECOVERY_TOLERANCE: f64 = 1.0e-12;

    let half_rate = 0.5_f64.ln();
    let third_rate = (1.0_f64 / 3.0).ln();
    let mut rng = SplitMix64::new(seed);
    let mut recovered_replicates = 0_usize;
    let mut failed_replicates = 0_usize;
    let mut half_rate_admitted_pairs = 0_usize;
    let mut third_rate_admitted_pairs = 0_usize;
    let mut error_sum = 0.0;
    let mut squared_error_sum = 0.0;
    let mut first_quarter_squared_error_sum = 0.0;
    let mut max_absolute_error = 0.0_f64;
    let mut covered_replicates = 0_usize;
    let mut covered_mass = 0.0;

    for replicate_index in 0..REPLICATES {
        let mut rows = Vec::with_capacity(32);
        let half_pairs = append_missingness_unit(&mut rows, &mut rng, 1, half_rate, true);
        let third_pairs = append_missingness_unit(&mut rows, &mut rng, 2, third_rate, false);
        half_rate_admitted_pairs += half_pairs;
        third_rate_admitted_pairs += third_pairs;

        let Ok(summary) = recover_within_unit_irregular_rate_summary(
            &rows,
            IrregularRateEstimand::LagPairAverageV1,
        ) else {
            failed_replicates += 1;
            continue;
        };
        let Some(estimate) = summary.estimate() else {
            failed_replicates += 1;
            continue;
        };

        assert_eq!(summary.candidate_units(), 2);
        assert_eq!(summary.contributing_units(), 2);
        assert_eq!(summary.admitted_pairs(), half_pairs + third_pairs);
        assert_eq!(summary.refused_pairs(), 2);
        assert_eq!(
            summary.candidate_pairs(),
            summary.admitted_pairs() + summary.refused_pairs()
        );

        let mut true_rate_sum = 0.0;
        let mut true_rate_denominator = 0.0;
        for _ in 0..half_pairs {
            true_rate_sum += half_rate;
            true_rate_denominator += 1.0;
        }
        for _ in 0..third_pairs {
            true_rate_sum += third_rate;
            true_rate_denominator += 1.0;
        }
        let true_pair_target = true_rate_sum / true_rate_denominator;
        let error = estimate - true_pair_target;
        let absolute_error = error.abs();

        recovered_replicates += 1;
        error_sum += error;
        squared_error_sum += error * error;
        if replicate_index < FIRST_QUARTER_REPLICATES {
            first_quarter_squared_error_sum += error * error;
        }
        max_absolute_error = max_absolute_error.max(absolute_error);
        if absolute_error <= RECOVERY_TOLERANCE {
            covered_replicates += 1;
            covered_mass += 1.0;
        }
    }

    let bias = error_sum / REPLICATES_F64;
    let rmse = (squared_error_sum / REPLICATES_F64).sqrt();
    let first_quarter_rmse =
        (first_quarter_squared_error_sum / FIRST_QUARTER_REPLICATES_F64).sqrt();
    let recovery_coverage = covered_mass / REPLICATES_F64;
    let variance_numerator = (squared_error_sum - REPLICATES_F64 * bias * bias).max(0.0);
    let bias_standard_error =
        (variance_numerator / (REPLICATES_F64 - 1.0) / REPLICATES_F64).sqrt();

    let z = 1.96_f64;
    let z_squared = z * z;
    let wilson_denominator = 1.0 + z_squared / REPLICATES_F64;
    let wilson_center =
        (recovery_coverage + z_squared / (2.0 * REPLICATES_F64)) / wilson_denominator;
    let wilson_margin = z
        * ((recovery_coverage * (1.0 - recovery_coverage) / REPLICATES_F64)
            + z_squared / (4.0 * REPLICATES_F64 * REPLICATES_F64))
            .sqrt()
        / wilson_denominator;

    assert_eq!(covered_replicates, recovered_replicates);

    MissingnessRecoveryEvidence {
        attempted_replicates: REPLICATES,
        recovered_replicates,
        failed_replicates,
        half_rate_admitted_pairs,
        third_rate_admitted_pairs,
        bias,
        rmse,
        max_absolute_error,
        first_quarter_rmse,
        recovery_coverage,
        coverage_wilson_lower_95: wilson_center - wilson_margin,
        bias_standard_error,
    }
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
fn informative_missingness_recovery_is_reproducible_and_pair_weighted() {
    let first = run_rate_associated_missingness_recovery(0x4950_2026_0912);
    let replay = run_rate_associated_missingness_recovery(0x4950_2026_0912);

    assert_eq!(first.attempted_replicates, 256);
    assert_eq!(first.recovered_replicates, 256);
    assert_eq!(first.failed_replicates, 0);
    assert!(
        first.half_rate_admitted_pairs > 2 * first.third_rate_admitted_pairs,
        "retention depends on the true rate and must create informative pair multiplicity"
    );
    assert!(first.bias.abs() <= 1.0e-13);
    assert!(first.rmse <= 3.0e-13);
    assert!(first.max_absolute_error <= 1.0e-12);
    assert!(first.recovery_coverage >= 0.999);
    assert!(first.coverage_wilson_lower_95 > 0.98);
    assert!(first.bias.abs() <= 3.0 * first.bias_standard_error + 1.0e-15);
    assert!(first.rmse <= 2.0 * first.first_quarter_rmse + 1.0e-15);

    assert_eq!(first.attempted_replicates, replay.attempted_replicates);
    assert_eq!(first.recovered_replicates, replay.recovered_replicates);
    assert_eq!(first.failed_replicates, replay.failed_replicates);
    assert_eq!(
        first.half_rate_admitted_pairs,
        replay.half_rate_admitted_pairs
    );
    assert_eq!(
        first.third_rate_admitted_pairs,
        replay.third_rate_admitted_pairs
    );
    assert_eq!(first.bias.to_bits(), replay.bias.to_bits());
    assert_eq!(first.rmse.to_bits(), replay.rmse.to_bits());
    assert_eq!(
        first.max_absolute_error.to_bits(),
        replay.max_absolute_error.to_bits()
    );
    assert_eq!(
        first.first_quarter_rmse.to_bits(),
        replay.first_quarter_rmse.to_bits()
    );
    assert_eq!(
        first.recovery_coverage.to_bits(),
        replay.recovery_coverage.to_bits()
    );
    assert_eq!(
        first.coverage_wilson_lower_95.to_bits(),
        replay.coverage_wilson_lower_95.to_bits()
    );
    assert_eq!(
        first.bias_standard_error.to_bits(),
        replay.bias_standard_error.to_bits()
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
