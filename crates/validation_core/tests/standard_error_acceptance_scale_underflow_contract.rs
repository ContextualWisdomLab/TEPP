use validation_core::accept_within_standard_errors;

#[test]
fn positive_standard_error_bound_survives_scale_reduction() {
    let estimate = 1.0e308_f64;
    let target = f64::from_bits(estimate.to_bits() - 1);
    let standard_error = 2.2e-16_f64;
    let multiplier = 1.0e308_f64;

    let represented_residual = estimate - target;
    let represented_bound = multiplier * standard_error;
    assert!(represented_residual.is_finite());
    assert!(represented_bound.is_finite());
    assert!(represented_residual <= represented_bound);
    assert_eq!(standard_error / estimate, 0.0);

    assert_eq!(
        accept_within_standard_errors(estimate, target, standard_error, multiplier),
        Ok(true),
        "a finite positive k*SE bound must not disappear because SE/scale underflows first"
    );
}

#[test]
fn scale_underflow_repair_does_not_accept_a_smaller_finite_bound() {
    let estimate = 1.0e308_f64;
    let target = f64::from_bits(estimate.to_bits() - 1);
    let standard_error = 1.8e-16_f64;
    let multiplier = 1.0e308_f64;

    let represented_residual = estimate - target;
    let represented_bound = multiplier * standard_error;
    assert!(represented_residual.is_finite());
    assert!(represented_bound.is_finite());
    assert!(represented_residual > represented_bound);
    assert_eq!(standard_error / estimate, 0.0);

    assert_eq!(
        accept_within_standard_errors(estimate, target, standard_error, multiplier),
        Ok(false),
        "restoring the finite bound must preserve a nearby rejection"
    );
}

#[test]
fn both_overflow_fallback_does_not_round_an_exact_rejection_into_acceptance() {
    let estimate = f64::from_bits(0x7fee_446c_f80d_ddbc);
    let target = f64::from_bits(0xffe2_d7e3_9796_6af3);
    let standard_error = f64::from_bits(0x7362_0ad2_2ddb_6f38);
    let multiplier = f64::from_bits(0x4c85_c69a_c1c7_a9ed);

    assert!((estimate - target).is_infinite());
    assert!((multiplier * standard_error).is_infinite());

    let scale = estimate.abs().max(target.abs()).max(standard_error).max(1.0);
    let predecessor_scaled_error = (estimate / scale) - (target / scale);
    let predecessor_scaled_bound = multiplier * (standard_error / scale);
    assert_eq!(predecessor_scaled_error, predecessor_scaled_bound);

    assert_eq!(
        accept_within_standard_errors(estimate, target, standard_error, multiplier),
        Ok(false),
        "the both-overflow fallback must preserve the exact represented-input inequality instead of accepting a normalization tie"
    );
}

#[test]
fn both_overflow_fallback_accepts_the_adjacent_multiplier_that_crosses_the_boundary() {
    let estimate = f64::from_bits(0x7fee_446c_f80d_ddbc);
    let target = f64::from_bits(0xffe2_d7e3_9796_6af3);
    let standard_error = f64::from_bits(0x7362_0ad2_2ddb_6f38);
    let multiplier = f64::from_bits(0x4c85_c69a_c1c7_a9ee);

    assert!((estimate - target).is_infinite());
    assert!((multiplier * standard_error).is_infinite());
    assert_eq!(
        accept_within_standard_errors(estimate, target, standard_error, multiplier),
        Ok(true),
        "one ULP larger multiplier is on the admissible side of the represented-input boundary"
    );
}
