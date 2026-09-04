use validation_core::bias_standard_error;

#[test]
fn common_rounded_residual_highs_do_not_recenter_low_terms_on_a_rounded_mean() {
    let quarter_ulp_at_one = 2.0_f64.powi(-54);
    let truth = [quarter_ulp_at_one, 0.0, 0.0];
    let recovered = [1.0; 3];

    let standard_error =
        bias_standard_error(&truth, &recovered).expect("represented standard error");
    // Every binary64 subtraction rounds to 1.0, but the represented-input
    // residuals are [1 - 2^-54, 1, 1]. Their exact mean is 1 - 2^-54 / 3,
    // so SE is exactly 2^-54 / 3 before the final binary64 rounding. Rounding
    // the low-term mean first and then centering moves the result one ULP up.
    assert_eq!(standard_error.to_bits(), 0x3c75_5555_5555_5555);

    let mirrored_truth = truth.map(|value| -value);
    let mirrored_recovered = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&mirrored_truth, &mirrored_recovered)
        .expect("mirrored represented standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3c75_5555_5555_5555);
}
