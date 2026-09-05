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
fn inexact_subtraction_tie_below_bound_remains_accepted() {
    let target = f64::from_bits(1);
    assert_eq!(1.0 - target, 1.0);
    assert_eq!(
        accept_within_standard_errors(1.0, target, 1.0, 1.0),
        Ok(true),
        "the exact represented residual is below the exact unit bound"
    );
}

#[test]
fn inexact_subtraction_tie_above_bound_preserves_strict_rejection() {
    let target = f64::from_bits(0xbc90_0000_0000_0000); // -2^-54
    let represented_residual = 1.0 - target;
    assert_eq!(represented_residual, 1.0);

    // The exact represented residual is 1 + 2^-54, but the subtraction rounds
    // to 1.0. The bound is exactly 1.0, so the scientific inequality is false.
    assert_eq!(
        accept_within_standard_errors(1.0, target, 1.0, 1.0),
        Ok(false),
        "subtraction rounding must not turn a strict represented-input rejection into equality"
    );
}

#[test]
fn negative_rounded_difference_uses_absolute_residual_correction_sign() {
    let target = f64::from_bits(0x3c90_0000_0000_0000); // +2^-54
    let represented_difference = -1.0 - target;
    assert_eq!(represented_difference, -1.0);

    // The exact difference is -1 - 2^-54, so the absolute residual is
    // 1 + 2^-54 and remains strictly above the exact unit bound.
    assert_eq!(
        accept_within_standard_errors(-1.0, target, 1.0, 1.0),
        Ok(false),
        "the subtraction low term must flip sign when the rounded difference is negative"
    );
}

#[test]
fn subnormal_bound_rounding_must_not_hide_a_strict_rejection() {
    let minimum_subnormal = f64::from_bits(1);
    let multiplier = f64::from_bits(0x1e5_8000_0000_0000); // 1.5 * 2^-538
    let standard_error = f64::from_bits(0x1e6_0000_0000_0000); // 2^-537

    assert_eq!(multiplier * standard_error, minimum_subnormal);
    // The exact represented product is 3/4 of the minimum subnormal. Its FMA
    // correction is only -1/4 ULP at zero and therefore rounds to signed zero.
    assert_eq!(multiplier.mul_add(standard_error, -minimum_subnormal), -0.0);

    assert_eq!(
        accept_within_standard_errors(minimum_subnormal, 0.0, standard_error, multiplier),
        Ok(false),
        "a product rounded up to the minimum subnormal must not cover the larger exact residual"
    );
}

#[test]
fn exact_minimum_subnormal_bound_remains_accepted() {
    let minimum_subnormal = f64::from_bits(1);
    assert_eq!(
        accept_within_standard_errors(minimum_subnormal, 0.0, minimum_subnormal, 1.0),
        Ok(true),
        "an exactly represented minimum-subnormal bound still covers an equal residual"
    );
}

#[test]
fn minimum_normal_bound_rounding_must_not_hide_a_strict_rejection() {
    let minimum_normal = f64::MIN_POSITIVE;
    let standard_error = f64::from_bits(0x1fff_ffff_fc00_0000); // (1 - 2^-27) * 2^-511
    let multiplier = f64::from_bits(0x2000_0000_0200_0000); // (1 + 2^-27) * 2^-511

    assert_eq!(multiplier * standard_error, minimum_normal);
    // The exact represented product is (1 - 2^-54) * 2^-1022,
    // one quarter of a minimum-subnormal ULP below the minimum normal. The
    // multiplication rounds up, while its FMA correction itself rounds to zero.
    assert_eq!(multiplier.mul_add(standard_error, -minimum_normal), -0.0);

    assert_eq!(
        accept_within_standard_errors(minimum_normal, 0.0, standard_error, multiplier),
        Ok(false),
        "an underflowed product correction at the normal/subnormal boundary must not erase a strict rejection"
    );
}

#[test]
fn exact_minimum_normal_bound_remains_accepted() {
    let minimum_normal = f64::MIN_POSITIVE;
    assert_eq!(
        accept_within_standard_errors(minimum_normal, 0.0, minimum_normal, 1.0),
        Ok(true),
        "an exactly represented minimum-normal bound still covers an equal residual"
    );
}
