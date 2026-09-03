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
    assert_eq!(bias_standard_error(&truth, &recovered), Ok(expected));
}
