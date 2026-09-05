use validation_core::bias_standard_error;

#[test]
fn bias_standard_error_does_not_round_the_exact_residual_mean_before_dispersion() {
    let ulp_at_one = 2.0_f64.powi(-52);
    let recovered = [1.0, 1.0 - ulp_at_one, 1.0];

    let standard_error = bias_standard_error(&[0.0; 3], &recovered)
        .expect("exact-residual represented standard error");
    // The represented residuals are exact and have mean 1 - 2^-52/3. Their
    // exact standard error is 2^-52/3. Rounding that mean first instead creates
    // deviations [2^-53, -2^-53, 2^-53] and overstates the uncertainty.
    assert_eq!(standard_error.to_bits(), 0x3c95_5555_5555_5555);

    let mirrored = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&[0.0; 3], &mirrored)
        .expect("mirrored exact-residual represented standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3c95_5555_5555_5555);
}
