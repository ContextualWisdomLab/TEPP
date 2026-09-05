use validation_core::{ValidationError, bias_standard_error};

#[test]
fn bias_standard_error_preserves_exact_integer_divisor_two_level_geometry() {
    let repeated = f64::from_bits(0x3fef_ffff_ffff_fffd);
    let recovered = [
        0.0, 0.0, 0.0, repeated, repeated, repeated, repeated, repeated, repeated,
    ];

    let standard_error = bias_standard_error(&[0.0; 9], &recovered)
        .expect("represented-input standard error");
    // With three observations at one exact residual level and six at the other,
    // m(n-m)/(n^2(n-1)) = 3*6/(9^2*8) = 1/36. Therefore SE(mean) is exactly
    // |gap|/6. The predecessor translated sum/square/sqrt path returns the
    // adjacent upper binary64 value for this represented gap.
    assert_eq!(standard_error.to_bits(), 0x3fc5_5555_5555_5553);

    let permuted = [
        repeated, 0.0, repeated, 0.0, repeated, 0.0, repeated, repeated, repeated,
    ];
    let permuted_standard_error = bias_standard_error(&[0.0; 9], &permuted)
        .expect("permuted represented-input standard error");
    assert_eq!(permuted_standard_error.to_bits(), 0x3fc5_5555_5555_5553);

    let mirrored = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&[0.0; 9], &mirrored)
        .expect("mirrored represented-input standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3fc5_5555_5555_5553);

    let minimum_subnormal = f64::from_bits(1);
    assert_eq!(
        bias_standard_error(
            &[0.0; 9],
            &[
                0.0,
                0.0,
                0.0,
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
