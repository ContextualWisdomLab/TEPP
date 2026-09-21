//! Public contract for a representable translated geometry whose SE rounds to false zero.

use validation_core::{ValidationError, bias_standard_error};

#[test]
fn translated_subnormal_geometry_rejects_false_zero_standard_error() {
    // Let u = 2^-1074. The represented residuals [0, u, u, 2u] admit an exact
    // canonical translation to [-u, 0, 0, u]. Their exact SE(mean) is
    // sqrt(1/6) * u, strictly positive but below the binary64 half-subnormal
    // rounding boundary. The bounded exact pair-distance seed therefore cannot
    // represent the result, and the general translated path must fail closed
    // rather than report a perfect zero standard error.
    let truth = [0.0; 4];
    let u = f64::from_bits(1);
    let recovered = [0.0, u, u, f64::from_bits(2)];

    assert_eq!(
        bias_standard_error(&truth, &recovered),
        Err(ValidationError::InvalidInput)
    );

    let permuted = [f64::from_bits(2), u, 0.0, u];
    assert_eq!(
        bias_standard_error(&truth, &permuted),
        Err(ValidationError::InvalidInput)
    );
}
