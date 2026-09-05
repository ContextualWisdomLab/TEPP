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
