use validation_core::{ValidationError, bias_standard_error};

#[test]
fn bias_standard_error_preserves_exact_two_level_count_geometry() {
    let repeated = f64::from_bits(0x3fef_ffff_ffff_ffff);
    let recovered = [0.0, 0.0, repeated, repeated, repeated];

    let standard_error = bias_standard_error(&[0.0; 5], &recovered)
        .expect("represented-input standard error");
    // For an exactly translated two-level sample with counts 2 and 3,
    // SE(mean) = |gap| * sqrt(2 * 3 / (5^2 * 4)). Counting the two levels
    // preserves that exact sample geometry. Reconstructing the same quantity
    // through rounded translated sums and squares lands one ULP high for
    // gap = next_down(1.0).
    assert_eq!(standard_error.to_bits(), 0x3fcf_5a7c_ecdb_6849);

    let permuted = [repeated, 0.0, repeated, 0.0, repeated];
    let permuted_standard_error = bias_standard_error(&[0.0; 5], &permuted)
        .expect("permuted represented-input standard error");
    assert_eq!(permuted_standard_error.to_bits(), 0x3fcf_5a7c_ecdb_6849);

    let mirrored = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&[0.0; 5], &mirrored)
        .expect("mirrored represented-input standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3fcf_5a7c_ecdb_6849);

    let minimum_subnormal = f64::from_bits(1);
    assert_eq!(
        bias_standard_error(
            &[0.0; 5],
            &[0.0, 0.0, minimum_subnormal, minimum_subnormal, minimum_subnormal],
        ),
        Err(ValidationError::InvalidInput)
    );
}
