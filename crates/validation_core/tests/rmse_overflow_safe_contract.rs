use validation_core::{rmse_standard_error, root_mean_square_error};

#[test]
fn representable_extreme_rmse_does_not_fail_on_squared_residual_overflow() {
    let truth = [0.0, 0.0];
    let recovered = [f64::MAX, f64::MAX];

    assert_eq!(root_mean_square_error(&truth, &recovered), Ok(f64::MAX));
    assert_eq!(rmse_standard_error(&truth, &recovered), Ok(0.0));
}
