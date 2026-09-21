//! Public contract for normalized three-level square-sum-inexact fallback.

use validation_core::bias_standard_error;

#[test]
fn normalized_three_level_refuses_inexact_square_sum_without_losing_low_term_geometry() {
    let common_high = 2.0_f64.powi(1023);
    let dominant_low = 2.0_f64.powi(969);
    let minor_low = 2.0_f64.powi(942);
    let recovered = [common_high; 3];

    // Each public subtraction rounds to the same represented high residual, while
    // error-free subtraction retains low terms [0, +2^969, -2^942]. The bounded
    // exact pair-distance route therefore refuses the nonzero subtraction
    // roundoff and the bias fallback translates those low terms around the exact
    // zero anchor. Raw three-level products overflow; power-of-two normalization
    // yields [0, 1, -2^-27]. Both squares and the cross-product are individually
    // exact, but 1 + 2^-54 rounds to 1. The exact-admission path must refuse at
    // its square-sum roundoff proof rather than treating that rounded sum as
    // authoritative. The represented-sample identity
    // SE(mean)^2 = (A^2 + b^2 + A*b) / 9 then rounds to the bit pattern below.
    let truth = [0.0, -dominant_low, minor_low];
    let first = bias_standard_error(&truth, &recovered)
        .expect("inexact normalized square sum falls back to represented low-term geometry");
    assert_eq!(first.to_bits(), 0x7c65_5555_56aa_aaab);

    let permuted_truth = [minor_low, 0.0, -dominant_low];
    let second = bias_standard_error(&permuted_truth, &recovered)
        .expect("permutation preserves the same represented low-term geometry");
    assert_eq!(second.to_bits(), first.to_bits());
}
