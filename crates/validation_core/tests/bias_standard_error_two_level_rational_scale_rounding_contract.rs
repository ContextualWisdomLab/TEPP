use validation_core::{ValidationError, bias_standard_error};

#[test]
fn bias_standard_error_preserves_exact_rational_square_two_level_geometry() {
    let repeated = f64::from_bits(0x3fef_ffff_ffff_fffe);
    let recovered = [
        0.0, 0.0, repeated, repeated, repeated, repeated, repeated, repeated, repeated, repeated,
    ];

    let standard_error = bias_standard_error(&[0.0; 10], &recovered)
        .expect("represented-input standard error");
    // With two observations at one exact residual level and eight at the other,
    // m(n-m)/(n^2(n-1)) = 2*8/(10^2*9) = 4/225. Therefore SE(mean) is exactly
    // 2*|gap|/15. GAP-103 admits only reciprocal-integer-square count factors,
    // so its translated sum/square/sqrt fallback returns the adjacent upper
    // binary64 value for this represented gap.
    assert_eq!(standard_error.to_bits(), 0x3fc1_1111_1111_1110);

    let permuted = [
        repeated, 0.0, repeated, repeated, repeated, 0.0, repeated, repeated, repeated, repeated,
    ];
    let permuted_standard_error = bias_standard_error(&[0.0; 10], &permuted)
        .expect("permuted represented-input standard error");
    assert_eq!(permuted_standard_error.to_bits(), 0x3fc1_1111_1111_1110);

    let mirrored = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&[0.0; 10], &mirrored)
        .expect("mirrored represented-input standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3fc1_1111_1111_1110);

    let minimum_subnormal = f64::from_bits(1);
    assert_eq!(
        bias_standard_error(
            &[0.0; 10],
            &[
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
            ],
        ),
        Err(ValidationError::InvalidInput)
    );
}
