//! Overflow-safe mean-bias and bias standard-error contract.
//!
//! Finite represented results remain valid even when a naive raw residual sum or
//! square sum would overflow binary64. Constant extreme bias must retain `f64::MAX`
//! with zero standard error, and a symmetric `±1e154` spread must retain its finite
//! standard error rather than fail because an intermediate square sum is infinite.

use validation_core::{bias_standard_error, mean_bias};

#[test]
fn representable_extreme_constant_bias_survives_raw_sum_overflow() {
    let truth = [0.0, 0.0];
    let recovered = [f64::MAX, f64::MAX];

    assert_eq!(mean_bias(&truth, &recovered), Ok(f64::MAX));
    assert_eq!(bias_standard_error(&truth, &recovered), Ok(0.0));
}

#[test]
fn representable_bias_standard_error_survives_raw_square_sum_overflow() {
    let truth = [0.0, 0.0, 0.0];
    let recovered = [1.0e154, -1.0e154, 0.0];

    let expected = 1.0e154 / 3.0_f64.sqrt();
    let actual = bias_standard_error(&truth, &recovered).expect("representable standard error");
    assert!(((actual - expected) / expected).abs() <= f64::EPSILON);
}
