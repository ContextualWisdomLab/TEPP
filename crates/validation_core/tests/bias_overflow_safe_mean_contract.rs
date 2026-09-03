use validation_core::{bias_standard_error, mean_bias};

#[test]
fn representable_extreme_constant_bias_survives_raw_sum_overflow() {
    let truth = [0.0, 0.0];
    let recovered = [f64::MAX, f64::MAX];

    assert_eq!(mean_bias(&truth, &recovered), Ok(f64::MAX));
    assert_eq!(bias_standard_error(&truth, &recovered), Ok(0.0));
}
