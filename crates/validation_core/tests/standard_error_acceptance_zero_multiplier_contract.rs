use validation_core::accept_within_standard_errors;

#[test]
fn zero_multiplier_requires_exact_recovery_before_scale_reduction() {
    let minimum_subnormal = f64::from_bits(1);

    assert_eq!(
        accept_within_standard_errors(minimum_subnormal, 0.0, f64::MAX, 0.0),
        Ok(false),
        "k = 0 is an exact-recovery gate; scaling by a huge SE must not erase a nonzero residual"
    );
    assert_eq!(
        accept_within_standard_errors(-minimum_subnormal, 0.0, f64::MAX, 0.0),
        Ok(false),
        "the negative mirror must remain a nonzero residual"
    );
    assert_eq!(
        accept_within_standard_errors(0.0, 0.0, f64::MAX, 0.0),
        Ok(true),
        "exact equality remains accepted when k = 0"
    );
}

#[test]
fn exact_recovery_treats_signed_zero_as_one_numeric_value() {
    assert_eq!(
        accept_within_standard_errors(-0.0, 0.0, f64::MAX, 0.0),
        Ok(true),
        "zero multiplier must use numeric equality, not signed-zero bit identity"
    );
    assert_eq!(
        accept_within_standard_errors(0.0, -0.0, 0.0, 1.0),
        Ok(true),
        "zero-SE exact recovery must not split +0.0 and -0.0 into different scientific values"
    );
}
