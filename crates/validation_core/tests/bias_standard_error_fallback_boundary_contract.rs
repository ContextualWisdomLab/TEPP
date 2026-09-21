//! Fail-closed boundary contracts for the bounded exact bias-SE admission route.

use validation_core::{ValidationError, bias_standard_error};

#[test]
fn bounded_exact_route_defers_invalid_inputs_to_the_generic_contract() {
    let four = [0.0; 4];
    let three = [0.0; 3];
    assert_eq!(
        bias_standard_error(&four, &three),
        Err(ValidationError::InvalidInput),
        "mismatched vectors must leave the bounded exact route and fail closed",
    );

    let non_finite_truth = [f64::NAN, 0.0, 0.0, 0.0];
    assert_eq!(
        bias_standard_error(&non_finite_truth, &four),
        Err(ValidationError::InvalidInput),
        "non-finite truth values must not be admitted by the exact route",
    );

    let non_finite_recovered = [f64::INFINITY, 0.0, 0.0, 0.0];
    assert_eq!(
        bias_standard_error(&four, &non_finite_recovered),
        Err(ValidationError::InvalidInput),
        "non-finite recovered values must not be admitted by the exact route",
    );

    let overflow_truth = [-f64::MAX, 0.0, 0.0, 0.0];
    let overflow_recovered = [f64::MAX, 0.0, 0.0, 0.0];
    assert_eq!(
        bias_standard_error(&overflow_truth, &overflow_recovered),
        Err(ValidationError::InvalidInput),
        "a non-representable residual must fall through and remain fail closed",
    );
}
