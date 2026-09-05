use validation_core::wilson_coverage_interval;

#[test]
fn exact_count_all_covered_preserves_representable_small_z_uncertainty() {
    // This z is the binary64 value 0x1.0000000000001p-27. Squaring it yields
    // 0x1.0000000000002p-54. For n=1, the exact represented-input Wilson lower
    // endpoint 1 / (1 + z^2) rounds to next_down(1.0), not to exact 1.0.
    // Forming 1 + z^2 first rounds that denominator to 1.0 and erases the
    // representable finite-sample uncertainty.
    let z = f64::from_bits(0x3e40_0000_0000_0001);
    assert_eq!((z * z).to_bits(), 0x3c90_0000_0000_0002);

    let (lower, upper) = wilson_coverage_interval(&[0.0], &[-1.0], &[1.0], z)
        .expect("one covered interval with finite positive z must produce Wilson evidence");

    assert_eq!(lower.to_bits(), 0x3fef_ffff_ffff_ffff);
    assert_eq!(upper, 1.0);
}

#[test]
fn exact_count_all_covered_keeps_unrepresentable_tiny_uncertainty_at_one() {
    // At z=2^-28, z^2=2^-56. The exact lower endpoint differs from one by less
    // than half of the binary64 spacing immediately below one, so exact 1.0 is
    // the correctly rounded represented endpoint. The boundary repair must not
    // manufacture an uncertainty value that binary64 cannot represent.
    let z = f64::from_bits(0x3e30_0000_0000_0000);
    assert_eq!((z * z).to_bits(), 0x3c70_0000_0000_0000);

    let (lower, upper) = wilson_coverage_interval(&[0.0], &[-1.0], &[1.0], z)
        .expect("finite positive z below the representable miss threshold remains valid");

    assert_eq!(lower, 1.0);
    assert_eq!(upper, 1.0);
}
