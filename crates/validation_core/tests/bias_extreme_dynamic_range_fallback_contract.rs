//! Public contract for exact-translation fallback at extreme residual dynamic range.

use validation_core::bias_standard_error;

#[test]
fn extreme_dynamic_range_preserves_representable_standard_error_after_normalization_refusal() {
    let huge = f64::from_bits(0x7fe0_0000_0000_0000); // 2^1023
    let small = f64::from_bits(0x3cb0_0000_0000_0000); // 2^-52
    let truth = [0.0; 4];
    let recovered = [0.0, huge, -huge, small];

    // The bounded n=4 exact route refuses because the common dyadic unit would
    // require a coefficient shift wider than its u128 proof domain, and the
    // pairwise reference also encounters the unrepresentable distance 2*huge.
    // The bias fallback can translate the represented residuals exactly around
    // the zero anchor, but normalizing `small` by the exact 2^1023 scale would
    // underflow it to zero. That route must therefore refuse rather than erase
    // represented mass, then recover the finite SE from the unnormalized sample.
    // For residuals [0, A, -A, b], SE(mean)^2 = A^2/6 + b^2/16. The correctly
    // rounded binary64 result for A=2^1023 and b=2^-52 is 0x7fca20bd700c2c3e.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("extreme represented dynamic range retains a finite standard error");
    assert_eq!(standard_error.to_bits(), 0x7fca_20bd_700c_2c3e);

    let permuted_recovered = [small, -huge, 0.0, huge];
    let permuted_standard_error = bias_standard_error(&truth, &permuted_recovered)
        .expect("permutation preserves the represented residual geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
