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

#[test]
fn finite_rounded_tie_accepts_when_exact_product_is_above_the_residual() {
    let estimate = 1.0_f64;
    let target = 0.0_f64;
    let standard_error = f64::from_bits(0x3fef_ffff_ffff_fc19);
    let multiplier = f64::from_bits(0x3ff0_0000_0000_01f4);

    assert_eq!(multiplier * standard_error, 1.0);
    assert_eq!(
        accept_within_standard_errors(estimate, target, standard_error, multiplier),
        Ok(true),
        "the tie discriminator must preserve acceptance when the exact represented product lies above one"
    );
}

#[test]
fn finite_exact_tie_remains_accepted_when_product_has_no_roundoff() {
    assert_eq!(
        accept_within_standard_errors(1.0, 0.0, 1.0, 1.0),
        Ok(true)
    );
}

#[test]
fn inexact_subtraction_tie_stays_on_the_conservative_rounded_path() {
    let target = f64::from_bits(1);
    assert_eq!(1.0 - target, 1.0);
    assert_eq!(
        accept_within_standard_errors(1.0, target, 1.0, 1.0),
        Ok(true),
        "this repair must not infer a product-only correction when subtraction itself rounded"
    );
}
