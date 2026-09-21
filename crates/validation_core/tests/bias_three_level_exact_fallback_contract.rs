//! Public regression for the exact three-level fallback after subtraction roundoff.
//!
//! The bounded pair-distance route must refuse when represented residual subtraction loses
//! low-order input mass. The fallback then recovers the exact translated three-level geometry
//! and must preserve its correctly rounded standard error without using a copied estimator.

use validation_core::bias_standard_error;

#[test]
fn low_term_three_level_fallback_preserves_exact_eisenstein_norm() {
    let unit = f64::from_bits(0x3c30_0000_0000_0000); // 2^-60
    let truth = [0.0, -5.0 * unit, -8.0 * unit];
    let recovered = [1.0; 3];

    // Every represented residual high rounds to 1.0, while the exact subtraction low terms are
    // [0, 5u, 8u]. Canonical translation anchors at 5u and therefore yields [-5u, 0, 3u].
    // For [0, x, y], SE(mean)^2 = (x^2 + y^2 - xy) / 9, so this geometry has
    // radicand (25 + 9 + 15)u^2 = 49u^2 and exact SE = 7u / 3. The expected bits are the
    // independently derived correctly rounded binary64 value of that represented-input result.
    let expected_bits = 0x3c42_aaaa_aaaa_aaab;
    assert_eq!(
        bias_standard_error(&truth, &recovered)
            .expect("represented three-level fallback remains finite")
            .to_bits(),
        expected_bits
    );

    let permuted_truth = [truth[2], truth[0], truth[1]];
    assert_eq!(
        bias_standard_error(&permuted_truth, &recovered)
            .expect("fallback is permutation invariant")
            .to_bits(),
        expected_bits
    );
}
