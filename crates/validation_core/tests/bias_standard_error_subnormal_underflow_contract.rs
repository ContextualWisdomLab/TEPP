//! Regression contract for a represented spread whose standard error is below binary64.
//!
//! Four observations containing one minimum subnormal residual have exact pair-distance
//! numerator 3 over denominator 48, so `SE(mean) = 2^-1076`. That quantity is nonzero
//! mathematically but has no nonzero binary64 representation. The Validation Evidence
//! boundary must therefore fail closed rather than promote zero as recovered precision.

use validation_core::{ValidationError, bias_standard_error};

#[test]
fn nonzero_subnormal_spread_fails_closed_below_binary64() {
    let minimum_subnormal = f64::from_bits(1);
    let truth = [0.0; 4];
    let recovered = [minimum_subnormal, 0.0, 0.0, 0.0];

    assert_eq!(
        bias_standard_error(&truth, &recovered),
        Err(ValidationError::InvalidInput)
    );

    let permuted = [0.0, minimum_subnormal, 0.0, 0.0];
    assert_eq!(
        bias_standard_error(&truth, &permuted),
        Err(ValidationError::InvalidInput),
        "fail-closed admission must not depend on observation order"
    );
}
