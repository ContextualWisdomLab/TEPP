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
