//! Public contract for two-level count geometry without an exact rational-square scale.

use validation_core::bias_standard_error;

#[test]
fn balanced_two_level_non_square_count_factor_uses_general_moment_path() {
    // Two residual levels with counts 2/2 give the count-only factor
    // m(n-m)/(n^2(n-1)) = 1/12. Because 12 is not a perfect square, the bounded
    // exact rational-scale admission must decline this geometry instead of
    // manufacturing an integer numerator/denominator. The translated general
    // moment remains exact through its dyadic normalization and returns
    // sqrt(1/12), preserving the ordinary n=4 psychometric SE contract.
    let truth = [0.0; 4];
    let recovered = [0.0, 0.0, 1.0, 1.0];

    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("balanced two-level SE remains representable through moment fallback");
    assert_eq!(standard_error.to_bits(), 0x3fd2_79a7_4590_331c);

    let permuted = [1.0, 0.0, 1.0, 0.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("two-level count geometry remains permutation invariant");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
