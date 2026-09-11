//! Public contract for three-level normalization whose restored SE is subnormal.

use validation_core::bias_standard_error;

#[test]
fn normalized_three_level_subnormal_restoration_falls_back_without_false_zero() {
    // Let u = 2^-1074. Canonical translation of [0, 2u, 4u] uses 2u as
    // the anchor and yields [-2u, 0, 2u]. Raw squares underflow, so the exact
    // three-level identity retries after reversible power-of-two normalization.
    // The normalized geometry is [-1, 0, 1], whose SE is sqrt(3)/3. Restoring
    // by 2u rounds to one minimum-subnormal unit. The normalized admission must
    // refuse that second rounding boundary and let the represented fallback
    // return the same nonzero scientific result rather than a false zero.
    let truth = [0.0; 3];
    let minimum_subnormal = f64::from_bits(1);
    let recovered = [0.0, f64::from_bits(2), f64::from_bits(4)];

    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("subnormal restoration remains representable through fallback");
    assert_eq!(standard_error.to_bits(), minimum_subnormal.to_bits());

    let permuted = [f64::from_bits(4), 0.0, f64::from_bits(2)];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permutation preserves the subnormal restored result");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
