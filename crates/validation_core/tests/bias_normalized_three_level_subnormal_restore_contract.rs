//! Public contract for normalized three-level subnormal restoration.

use validation_core::bias_standard_error;

#[test]
fn normalized_three_level_subnormal_restore_falls_back_without_false_zero() {
    let minimum_subnormal = f64::from_bits(1);
    let twice_minimum_subnormal = f64::from_bits(2);
    let truth = [0.0; 3];

    // For represented residuals [0, u, 2u] with u = 2^-1074, raw squares and
    // the cross-product underflow, so the exact three-level route retries after
    // power-of-two normalization. The normalized geometry [0, 1/2, 1] has exact
    // products and radicand 3/4, but restoring sqrt(3/4)/3 by scale 2u lands in
    // the subnormal range. That restoration is a second rounding boundary, so
    // exact admission must refuse it and let the general represented path decide
    // the final result instead of treating the normalized candidate as authority.
    // The correctly rounded represented-sample SE is one minimum subnormal.
    let recovered = [0.0, minimum_subnormal, twice_minimum_subnormal];
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("subnormal restoration falls back to represented-sample arithmetic");
    assert_eq!(standard_error.to_bits(), 1);

    // The same scientific geometry must remain invariant to observation order.
    let permuted = [twice_minimum_subnormal, 0.0, minimum_subnormal];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permuted subnormal geometry preserves the same represented result");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
