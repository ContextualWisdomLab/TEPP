use validation_core::accept_within_standard_errors;

#[test]
fn finite_rounded_tie_preserves_strict_rejection_from_represented_inputs() {
    let estimate = 1.0_f64;
    let target = 0.0_f64;
    let standard_error = f64::from_bits(0x3fef_ffff_fc00_0000);
    let multiplier = f64::from_bits(0x3ff0_0000_0200_0000);

    let represented_residual = estimate - target;
    let rounded_bound = multiplier * standard_error;
    assert_eq!(represented_residual, 1.0);
    assert_eq!(rounded_bound, represented_residual);

    // The represented factors are (1 - 2^-27) and (1 + 2^-27),
    // so their exact product is 1 - 2^-54: strictly below the residual.
    assert_eq!(
        accept_within_standard_errors(estimate, target, standard_error, multiplier),
        Ok(false),
        "a multiplication tie rounded to the residual must not erase the strict represented-input rejection"
    );
}

#[test]
fn finite_rounded_tie_repair_keeps_the_adjacent_acceptance() {
    let estimate = 1.0_f64;
    let target = 0.0_f64;
    let standard_error = f64::from_bits(0x3fef_ffff_fc00_0000);
    let multiplier = f64::from_bits(0x3ff0_0000_0200_0001);

    assert_eq!(
        accept_within_standard_errors(estimate, target, standard_error, multiplier),
        Ok(true),
        "one ULP larger multiplier is on the admissible side of the represented-input boundary"
    );
}
