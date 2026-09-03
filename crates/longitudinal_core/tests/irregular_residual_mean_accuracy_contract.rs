//! Accuracy contract for means of extreme but finite event-time log rates.

use longitudinal_core::{
    EventTimeInterval, LaggedWithinResidual, recover_centered_irregular_residual_log_rate,
};

fn pair(rate: f64, interval: f64) -> LaggedWithinResidual {
    let earlier = 1.0_f64;
    let later = (rate * interval).exp();
    LaggedWithinResidual::new(
        earlier,
        later,
        EventTimeInterval::new(interval).expect("positive event interval"),
    )
}

fn ratio_pair(later: f64, interval: f64) -> LaggedWithinResidual {
    LaggedWithinResidual::new(
        1.0,
        later,
        EventTimeInterval::new(interval).expect("positive event interval"),
    )
}

#[test]
fn compensated_mean_preserves_small_signal_between_extreme_rates() {
    let pairs = [
        pair(1.0e100, 1.0e-100),
        pair(1.0, 1.0),
        pair(-1.0e100, 1.0e-100),
    ];
    let recovered = recover_centered_irregular_residual_log_rate(&pairs)
        .expect("finite extreme-rate mean remains identifiable");
    let expected = 1.0 / 3.0;
    assert!((recovered - expected).abs() <= 1.0e-12);
}

#[test]
fn full_exponent_range_cancellation_preserves_representable_tiny_mean() {
    let tiny_interval = 4.0e-309_f64;
    let next_after_one = f64::from_bits(1.0_f64.to_bits() + 1);
    let tiny_rate = next_after_one.ln();
    let pairs = [
        ratio_pair(2.0, tiny_interval),
        ratio_pair(next_after_one, 1.0),
        ratio_pair(0.5, tiny_interval),
    ];

    let recovered = recover_centered_irregular_residual_log_rate(&pairs)
        .expect("opposing extreme rates retain the finite low-order mean");
    let expected = tiny_rate / 3.0;
    assert!(expected > 0.0 && expected.is_finite());
    assert_eq!(recovered.to_bits(), expected.to_bits());
}

#[test]
fn cancellation_does_not_underflow_subnormal_rates_before_the_mean() {
    let minimum_subnormal = f64::from_bits(1);
    let target_small_rate = f64::from_bits(2);
    let next_after_one = f64::from_bits(1.0_f64.to_bits() + 1);
    let small_interval = next_after_one.ln() / target_small_rate;
    let extreme_interval = 4.0e-309_f64;
    let pairs = [
        ratio_pair(2.0, extreme_interval),
        ratio_pair(next_after_one, small_interval),
        ratio_pair(next_after_one, small_interval),
        ratio_pair(0.5, extreme_interval),
    ];

    let recovered = recover_centered_irregular_residual_log_rate(&pairs)
        .expect("large cancellation must retain a representable subnormal mean");
    assert_eq!(recovered.to_bits(), minimum_subnormal.to_bits());
}

#[test]
fn same_sign_rate_mean_is_bit_stable_under_pair_permutation() {
    let pairs_a = [
        ratio_pair(2.0, 1.0),
        ratio_pair(4.0, 1.0),
        ratio_pair(8.0, 1.0),
    ];
    let pairs_b = [
        ratio_pair(2.0, 1.0),
        ratio_pair(8.0, 1.0),
        ratio_pair(4.0, 1.0),
    ];

    let recovered_a = recover_centered_irregular_residual_log_rate(&pairs_a)
        .expect("first permutation remains identifiable");
    let recovered_b = recover_centered_irregular_residual_log_rate(&pairs_b)
        .expect("second permutation remains identifiable");

    assert_eq!(
        recovered_a.to_bits(),
        recovered_b.to_bits(),
        "scientific evidence order must not change the binary64 reference mean"
    );
}
