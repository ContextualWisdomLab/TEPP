use validation_core::bias_standard_error;

#[test]
fn bias_standard_error_preserves_singleton_repeated_level_identity_beyond_three_rows() {
    let repeated = f64::from_bits(0x3fef_ffff_ffff_ffff);
    let recovered = [0.0, repeated, repeated, repeated];

    let standard_error = bias_standard_error(&[0.0; 4], &recovered)
        .expect("represented-input standard error");
    // For an exactly represented four-observation two-level sample [0, a, a, a],
    // the sample standard error of the mean simplifies to |a| / 4. The generic
    // translated second-moment path squares and square-roots the exact gap and
    // lands one ULP below that single exact power-of-two division for
    // a = next_down(1.0).
    assert_eq!(standard_error.to_bits(), 0x3fcf_ffff_ffff_ffff);

    let permuted = [repeated, 0.0, repeated, repeated];
    let permuted_standard_error = bias_standard_error(&[0.0; 4], &permuted)
        .expect("permuted represented-input standard error");
    assert_eq!(permuted_standard_error.to_bits(), 0x3fcf_ffff_ffff_ffff);

    let mirrored = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&[0.0; 4], &mirrored)
        .expect("mirrored represented-input standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3fcf_ffff_ffff_ffff);
}
