//! Recover representable mean bias even when individual signed residuals overflow binary64.
//!
//! Opposite residuals produced by swapping `±f64::MAX` must cancel to exact zero rather than fail
//! merely because each subtraction is individually infinite. A three-observation variant must retain
//! a one-minimum-subnormal mean after the overflowing pair cancels. The genuinely one-sided
//! unrepresentable case remains fail closed with `InvalidInput`; cancellation is not permission to
//! manufacture a finite result where the represented-input mean itself is outside binary64.

use validation_core::{ValidationError, mean_bias};

#[test]
fn opposite_overflowing_residuals_cancel_to_representable_zero() {
    let truth = [-f64::MAX, f64::MAX];
    let recovered = [f64::MAX, -f64::MAX];

    assert_eq!(
        mean_bias(&truth, &recovered),
        Ok(0.0),
        "representable mean bias must not fail only because individual signed residuals overflow"
    );
}

#[test]
fn overflowing_residual_cancellation_preserves_minimum_subnormal_mean() {
    let minimum_subnormal = f64::from_bits(1);
    let three_minimum_subnormals = f64::from_bits(3);
    let truth = [-f64::MAX, f64::MAX, 0.0];
    let recovered = [f64::MAX, -f64::MAX, three_minimum_subnormals];

    let bias = mean_bias(&truth, &recovered)
        .expect("the exact represented-input mean bias is one minimum subnormal");
    assert_eq!(bias.to_bits(), minimum_subnormal.to_bits());
}

#[test]
fn one_sided_unrepresentable_mean_bias_still_fails_closed() {
    assert_eq!(
        mean_bias(&[-f64::MAX], &[f64::MAX]),
        Err(ValidationError::InvalidInput)
    );
}
