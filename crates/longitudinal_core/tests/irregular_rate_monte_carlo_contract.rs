//! Scientific acceptance for the declared pair-weighted irregular-rate estimand.
//!
//! This test deliberately keeps Monte Carlo arithmetic in the test boundary. Production
//! finite-binary64 mean ownership remains outside TEPP.

use longitudinal_core::{
    EventTimedObservation, IrregularRateEstimand, recover_within_unit_irregular_rate_summary,
};

const REPLICATES: usize = 4_096;
const FIRST_QUARTER_REPLICATES: usize = REPLICATES / 4;
const INNOVATION_HALF_WIDTH: f64 = 0.12;
const Z_95: f64 = 1.96;

fn timed(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
    EventTimedObservation::new(unit, event_time, score)
}

fn exact_test_count(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("test count must fit in u32"))
}

#[derive(Clone, Copy)]
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

    fn signed_unit(&mut self) -> f64 {
        let mantissa = self.next_u64() >> 11;
        let high = u32::try_from(mantissa >> 32).expect("53-bit mantissa high part must fit u32");
        let low = u32::try_from(mantissa & u64::from(u32::MAX))
            .expect("53-bit mantissa low part must fit u32");
        let unit = (f64::from(high) * 4_294_967_296.0 + f64::from(low)) / 9_007_199_254_740_992.0;
        2.0 * unit - 1.0
    }
}

#[derive(Clone, Copy, Debug)]
struct GeneratedUnit {
    admitted_pairs: usize,
    pair_rate_variance_sum: f64,
}

fn append_noisy_missingness_unit(
    rows: &mut Vec<EventTimedObservation>,
    rng: &mut SplitMix64,
    unit: u32,
    true_log_rate: f64,
    retain_three_of_four: bool,
) -> GeneratedUnit {
    const LAST_NEGATIVE_TIME: usize = 12;
    let innovation_variance = INNOVATION_HALF_WIDTH * INNOVATION_HALF_WIDTH / 3.0;

    let mut log_magnitude = 0.0;
    let first_score = -1.0;
    let mut retained_times = Vec::with_capacity(LAST_NEGATIVE_TIME + 1);
    let mut negative_sum = first_score;
    retained_times.push(0_usize);
    rows.push(timed(unit, 0.0, first_score));

    for event_time in 1..=LAST_NEGATIVE_TIME {
        let innovation = INNOVATION_HALF_WIDTH * rng.signed_unit();
        log_magnitude += true_log_rate + innovation;

        let retain = if event_time == LAST_NEGATIVE_TIME {
            true
        } else {
            let draw = rng.next_u64();
            if retain_three_of_four {
                draw.trailing_zeros() < 2
            } else {
                draw.trailing_zeros() >= 2
            }
        };

        if retain {
            let score = -log_magnitude.exp();
            rows.push(timed(unit, exact_test_count(event_time), score));
            negative_sum += score;
            retained_times.push(event_time);
        }
    }

    rows.push(timed(
        unit,
        exact_test_count(LAST_NEGATIVE_TIME + 1),
        -negative_sum,
    ));

    let admitted_pairs = retained_times.len() - 1;
    let pair_rate_variance_sum = retained_times
        .windows(2)
        .map(|times| {
            let gap = exact_test_count(times[1] - times[0]);
            innovation_variance / gap
        })
        .sum();

    GeneratedUnit {
        admitted_pairs,
        pair_rate_variance_sum,
    }
}

#[derive(Clone, Copy, Debug)]
struct MonteCarloEvidence {
    attempted_replicates: usize,
    recovered_replicates: usize,
    failed_replicates: usize,
    half_rate_admitted_pairs: usize,
    third_rate_admitted_pairs: usize,
    bias: f64,
    bias_monte_carlo_se: f64,
    rmse: f64,
    first_quarter_rmse: f64,
    coverage_95: f64,
    coverage_wilson_lower_95: f64,
    coverage_wilson_upper_95: f64,
}

fn run_noisy_rate_associated_missingness(seed: u64) -> MonteCarloEvidence {
    let half_rate = 0.5_f64.ln();
    let third_rate = (1.0_f64 / 3.0).ln();
    let mut rng = SplitMix64::new(seed);

    let mut recovered_replicates = 0_usize;
    let mut failed_replicates = 0_usize;
    let mut first_quarter_recovered = 0_usize;
    let mut half_rate_admitted_pairs = 0_usize;
    let mut third_rate_admitted_pairs = 0_usize;
    let mut error_sum = 0.0;
    let mut squared_error_sum = 0.0;
    let mut first_quarter_squared_error_sum = 0.0;
    let mut covered_replicates = 0_usize;

    for replicate_index in 0..REPLICATES {
        let mut rows = Vec::with_capacity(32);
        let half = append_noisy_missingness_unit(&mut rows, &mut rng, 1, half_rate, true);
        let third = append_noisy_missingness_unit(&mut rows, &mut rng, 2, third_rate, false);
        half_rate_admitted_pairs += half.admitted_pairs;
        third_rate_admitted_pairs += third.admitted_pairs;

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

        let admitted_pairs = half.admitted_pairs + third.admitted_pairs;
        assert_eq!(summary.candidate_units(), 2);
        assert_eq!(summary.contributing_units(), 2);
        assert_eq!(summary.admitted_pairs(), admitted_pairs);
        assert_eq!(summary.refused_pairs(), 2);
        assert_eq!(
            summary.candidate_pairs(),
            summary.admitted_pairs() + summary.refused_pairs()
        );

        let half_pairs = exact_test_count(half.admitted_pairs);
        let third_pairs = exact_test_count(third.admitted_pairs);
        let admitted_pair_count = exact_test_count(admitted_pairs);
        let true_pair_target =
            (half_pairs * half_rate + third_pairs * third_rate) / admitted_pair_count;
        let error = estimate - true_pair_target;
        let conditional_standard_error =
            (half.pair_rate_variance_sum + third.pair_rate_variance_sum).sqrt()
                / admitted_pair_count;

        recovered_replicates += 1;
        error_sum += error;
        squared_error_sum += error * error;
        if replicate_index < FIRST_QUARTER_REPLICATES {
            first_quarter_recovered += 1;
            first_quarter_squared_error_sum += error * error;
        }
        if error.abs() <= Z_95 * conditional_standard_error {
            covered_replicates += 1;
        }
    }

    assert!(recovered_replicates > 1);
    assert!(first_quarter_recovered > 0);

    let recovered = exact_test_count(recovered_replicates);
    let bias = error_sum / recovered;
    let rmse = (squared_error_sum / recovered).sqrt();
    let first_quarter_rmse =
        (first_quarter_squared_error_sum / exact_test_count(first_quarter_recovered)).sqrt();
    let sample_error_variance =
        (squared_error_sum - recovered * bias * bias).max(0.0) / (recovered - 1.0);
    let bias_monte_carlo_se = (sample_error_variance / recovered).sqrt();
    let coverage_95 = exact_test_count(covered_replicates) / recovered;

    let z_squared = Z_95 * Z_95;
    let wilson_denominator = 1.0 + z_squared / recovered;
    let wilson_center = (coverage_95 + z_squared / (2.0 * recovered)) / wilson_denominator;
    let wilson_margin = Z_95
        * ((coverage_95 * (1.0 - coverage_95) / recovered)
            + z_squared / (4.0 * recovered * recovered))
            .sqrt()
        / wilson_denominator;

    MonteCarloEvidence {
        attempted_replicates: REPLICATES,
        recovered_replicates,
        failed_replicates,
        half_rate_admitted_pairs,
        third_rate_admitted_pairs,
        bias,
        bias_monte_carlo_se,
        rmse,
        first_quarter_rmse,
        coverage_95,
        coverage_wilson_lower_95: wilson_center - wilson_margin,
        coverage_wilson_upper_95: wilson_center + wilson_margin,
    }
}

#[test]
fn noisy_rate_associated_missingness_recovers_declared_pair_estimand() {
    let first = run_noisy_rate_associated_missingness(0x4950_2026_0912);
    let replay = run_noisy_rate_associated_missingness(0x4950_2026_0912);

    assert_eq!(first.attempted_replicates, REPLICATES);
    assert_eq!(first.recovered_replicates, REPLICATES);
    assert_eq!(first.failed_replicates, 0);
    assert!(
        first.half_rate_admitted_pairs > 2 * first.third_rate_admitted_pairs,
        "rate-associated retention must create informative pair multiplicity"
    );

    assert!(first.bias.abs() <= 3.0 * first.bias_monte_carlo_se);
    assert!(first.rmse > 0.01 && first.rmse < 0.03);
    assert!(
        (first.rmse - first.first_quarter_rmse).abs() <= 0.25 * first.rmse,
        "RMSE should stabilize rather than depend on the first quarter of replicates"
    );
    assert!(first.coverage_95 > 0.90 && first.coverage_95 < 0.99);
    assert!(first.coverage_wilson_lower_95 <= 0.95);
    assert!(first.coverage_wilson_upper_95 >= 0.95);

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
    assert_eq!(
        first.bias_monte_carlo_se.to_bits(),
        replay.bias_monte_carlo_se.to_bits()
    );
    assert_eq!(first.rmse.to_bits(), replay.rmse.to_bits());
    assert_eq!(
        first.first_quarter_rmse.to_bits(),
        replay.first_quarter_rmse.to_bits()
    );
    assert_eq!(first.coverage_95.to_bits(), replay.coverage_95.to_bits());
    assert_eq!(
        first.coverage_wilson_lower_95.to_bits(),
        replay.coverage_wilson_lower_95.to_bits()
    );
    assert_eq!(
        first.coverage_wilson_upper_95.to_bits(),
        replay.coverage_wilson_upper_95.to_bits()
    );
}
