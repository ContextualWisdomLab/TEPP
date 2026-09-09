//! Regression for the generic bias-SE fallback when exact low-term translation refuses.

use validation_core::bias_standard_error;

#[test]
fn equal_rounded_residuals_preserve_nontranslatable_low_term_dispersion() {
    let minimum_subnormal = f64::from_bits(1);
    let truth = [
        f64::from_bits(0x3c90_0000_0000_0000),
        f64::from_bits(0x3c80_0000_0000_0000),
        minimum_subnormal,
    ];
    let recovered = [1.0; 3];

    // All three represented subtractions round to 1.0, while their exact low
    // terms are -2^-54, -2^-55, and -2^-1074. No single low-term anchor can
    // translate all three differences exactly, so the general scaled fallback
    // must retain the nonzero dispersion instead of collapsing it to zero.
    let standard_error =
        bias_standard_error(&truth, &recovered).expect("low-term spread remains representable");
    assert_eq!(standard_error.to_bits(), 0x3c72_79a7_4590_331c);

    let reversed_truth = [minimum_subnormal, truth[1], truth[0]];
    let reversed = bias_standard_error(&reversed_truth, &recovered)
        .expect("permutation preserves low-term spread");
    assert_eq!(reversed.to_bits(), standard_error.to_bits());
}

#[test]
fn unequal_rounded_residuals_refuse_inexact_low_term_anchor_deltas() {
    let minimum_subnormal = f64::from_bits(1);
    let truth = [f64::from_bits(0x3c90_0000_0000_0000), minimum_subnormal, 0.0];
    let recovered = [1.0, 1.0, 2.0];

    // The first two represented residual highs are 1.0 with exact subtraction
    // low terms -2^-54 and -2^-1074; the third residual is exactly 2.0. The
    // bounded exact wrapper refuses on subtraction roundoff. In the fallback,
    // subtracting either low term from the other loses the minimum-subnormal
    // contribution, while the zero-low anchor cannot recombine 1.0 with -2^-54
    // error-free. Exact translation must therefore refuse every anchor and keep
    // the established rounded-residual scaled path. For [1, 1, 2], that path's
    // represented SE is 1/3, rounded to 0x3fd5555555555555.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("inexact low-term anchor deltas retain the rounded-residual fallback");
    assert_eq!(standard_error.to_bits(), 0x3fd5_5555_5555_5555);

    let permuted_truth = [0.0, truth[0], minimum_subnormal];
    let permuted_recovered = [2.0, 1.0, 1.0];
    let permuted = bias_standard_error(&permuted_truth, &permuted_recovered)
        .expect("low-term anchor refusal is permutation invariant");
    assert_eq!(permuted.to_bits(), standard_error.to_bits());
}
