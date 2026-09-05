use validation_core::{ValidationError, bias_standard_error};

#[test]
fn bias_standard_error_preserves_exact_dyadic_two_level_count_geometry() {
    let repeated = f64::from_bits(0x3fef_ffff_ffff_ffff);
    let recovered = [
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, repeated, repeated, repeated, repeated, repeated,
        repeated, repeated, repeated, repeated, repeated,
    ];

    let standard_error = bias_standard_error(&[0.0; 16], &recovered)
        .expect("represented-input standard error");
    // With six observations at one exact residual level and ten at the other,
    // m(n-m)/(n-1) = 6*10/15 = 4. Therefore SE(mean) is exactly |gap|/8.
    // Reconstructing that dyadic identity through translated sums, squares and
    // sqrt rounds next_down(1.0) up to 0.125 instead of preserving gap/8.
    assert_eq!(standard_error.to_bits(), 0x3fbf_ffff_ffff_ffff);

    let permuted = [
        repeated, 0.0, repeated, 0.0, repeated, 0.0, repeated, 0.0, repeated, 0.0,
        repeated, 0.0, repeated, repeated, repeated, repeated,
    ];
    let permuted_standard_error = bias_standard_error(&[0.0; 16], &permuted)
        .expect("permuted represented-input standard error");
    assert_eq!(permuted_standard_error.to_bits(), 0x3fbf_ffff_ffff_ffff);

    let mirrored = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&[0.0; 16], &mirrored)
        .expect("mirrored represented-input standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3fbf_ffff_ffff_ffff);

    let minimum_subnormal = f64::from_bits(1);
    assert_eq!(
        bias_standard_error(
            &[0.0; 16],
            &[
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
                minimum_subnormal,
            ],
        ),
        Err(ValidationError::InvalidInput)
    );
}
