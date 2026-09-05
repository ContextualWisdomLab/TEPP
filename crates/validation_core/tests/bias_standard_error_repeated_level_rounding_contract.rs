use validation_core::bias_standard_error;

#[test]
fn bias_standard_error_preserves_three_observation_repeated_level_identity() {
    let repeated = f64::from_bits(0x3fef_ffff_ffff_ffff);
    let recovered = [0.0, repeated, repeated];

    let standard_error = bias_standard_error(&[0.0; 3], &recovered)
        .expect("represented-input standard error");
    // For exactly represented residuals [0, a, a], the three-observation
    // standard error simplifies algebraically to |a| / 3. The predecessor
    // squared the normalized a values, formed the second moment, and then took
    // a square root; that extra projection lands one ULP below the single
    // correctly rounded division for a = next_down(1.0).
    assert_eq!(standard_error.to_bits(), 0x3fd5_5555_5555_5555);

    let permuted = [repeated, 0.0, repeated];
    let permuted_standard_error = bias_standard_error(&[0.0; 3], &permuted)
        .expect("permuted represented-input standard error");
    assert_eq!(permuted_standard_error.to_bits(), 0x3fd5_5555_5555_5555);

    let mirrored = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&[0.0; 3], &mirrored)
        .expect("mirrored represented-input standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3fd5_5555_5555_5555);
}
