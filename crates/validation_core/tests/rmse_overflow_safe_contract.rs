use validation_core::{rmse_standard_error, root_mean_square_error};

#[test]
fn representable_extreme_rmse_does_not_fail_on_squared_residual_overflow() {
    let truth = [0.0, 0.0];
    let recovered = [f64::MAX, f64::MAX];

    assert_eq!(root_mean_square_error(&truth, &recovered), Ok(f64::MAX));
    assert_eq!(rmse_standard_error(&truth, &recovered), Ok(0.0));
}

#[test]
fn subnormal_rmse_preserves_representable_error_and_refuses_false_zero() {
    let ulp = f64::from_bits(1);

    assert_eq!(root_mean_square_error(&[0.0, 0.0], &[ulp, 0.0]), Ok(ulp));
    assert_eq!(
        root_mean_square_error(&[0.0, 0.0, 0.0, 0.0], &[ulp, 0.0, 0.0, 0.0]),
        Err(validation_core::ValidationError::InvalidInput)
    );
}
