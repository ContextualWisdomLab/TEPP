use validation_core::bias_standard_error;

#[test]
fn bias_standard_error_preserves_roundoff_when_residual_high_parts_differ() {
    let quarter_ulp_at_one = 2.0_f64.powi(-54);
    let truth = [quarter_ulp_at_one, 3.0 * quarter_ulp_at_one, 0.0];
    let recovered = [1.0; 3];

    let standard_error =
        bias_standard_error(&truth, &recovered).expect("represented standard error");
    // Exact represented residuals are 1-2^-54, 1-3*2^-54, and 1. Their
    // standard error rounds to this value. Rounding the pairwise residuals first
    // instead yields [1, 1-2^-52, 1] and a materially larger result.
    assert_eq!(standard_error.to_bits(), 0x3c8c_38aa_37c3_f68d);

    let mirrored_truth = truth.map(|value| -value);
    let mirrored_recovered = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&mirrored_truth, &mirrored_recovered)
        .expect("mirrored represented standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3c8c_38aa_37c3_f68d);

    // Without subtraction roundoff, the existing rounded-residual path remains
    // authoritative for the same high-part pattern.
    let exact_residuals = [1.0, 1.0 - 2.0_f64.powi(-52), 1.0];
    let control = bias_standard_error(&[0.0; 3], &exact_residuals)
        .expect("exact-residual control standard error");
    assert_eq!(control.to_bits(), 0x3c96_a09e_667f_3bcd);
}
