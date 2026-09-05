use validation_core::bias_standard_error;

#[test]
fn bias_standard_error_does_not_double_round_through_a_non_power_scale() {
    let ulp_at_one = 2.0_f64.powi(-52);
    let low = 1.0 - 4.0 * ulp_at_one;
    let high = 1.0 + ulp_at_one;
    let recovered = [low, low, high];

    let standard_error = bias_standard_error(&[0.0; 3], &recovered)
        .expect("represented-input standard error");
    // The exact represented residual gap is 5 * 2^-52. For [a, a, a+d],
    // SE(mean) is exactly d / 3 before final binary64 rounding. Scaling by d
    // first instead turns that into rounded(1/3) * d and lands one ULP low.
    assert_eq!(standard_error.to_bits(), 0x3cba_aaaa_aaaa_aaab);

    let mirrored = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&[0.0; 3], &mirrored)
        .expect("mirrored represented-input standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3cba_aaaa_aaaa_aaab);
}
