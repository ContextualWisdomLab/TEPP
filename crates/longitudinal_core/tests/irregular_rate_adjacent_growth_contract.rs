#![forbid(unsafe_code)]

//! Regression contract for adjacent-float irregular residual growth.
//!
//! Driver et al. (2017, Eq. 3) requires the logarithm of the exact represented
//! magnitude ratio. Forming that ratio first can round an adjacent-float growth
//! step at a power-of-two boundary from roughly half an epsilon to one epsilon,
//! doubling the recovered log-rate before the logarithm is evaluated.

use longitudinal_core::{
    EventTimeInterval, LaggedWithinResidual, recover_centered_irregular_residual_log_rate,
};

#[test]
fn adjacent_power_of_two_growth_keeps_the_represented_relative_change() {
    let earlier = f64::from_bits(2.0_f64.to_bits() - 1);
    let later = 2.0_f64;
    let interval = EventTimeInterval::new(1.0).expect("unit event-time interval");
    let pair = LaggedWithinResidual::new(earlier, later, interval);

    let exact_relative_change = f64::EPSILON / 2.0;
    let expected = -(-exact_relative_change).ln_1p();
    let rounded_ratio_log = (later / earlier).ln();
    assert!(
        rounded_ratio_log > expected * 1.9,
        "the fixture must reproduce ratio-first double rounding"
    );

    let recovered = recover_centered_irregular_residual_log_rate(&[pair])
        .expect("adjacent represented growth must remain recoverable");

    assert_eq!(recovered.to_bits(), expected.to_bits());
}
