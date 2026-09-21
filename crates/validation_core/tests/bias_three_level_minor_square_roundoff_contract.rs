//! Public contract for three-level fallback when the minor square is rounded.

use validation_core::bias_standard_error;

#[test]
fn minor_level_inexact_square_preserves_represented_three_level_dispersion() {
    let minor = f64::from_bits(0x3370_0000_0000_0001); // next binary64 above 2^-200
    let truth = [0.0; 3];
    let recovered = [0.0, 1.0, minor];

    // The neutral-zero bounded proof cannot widen the 252-bit dyadic-unit gap
    // between `1` and `minor` into u128, while the pairwise proof refuses the
    // rounded subtraction `1 - minor`. Exact translation can still retain the
    // represented sample as [0, 1, minor]. Here `1^2` is exact, `minor^2` stays
    // finite and nonzero but loses low-order product mass, so the direct
    // three-level identity must refuse on its error-free-product predicate and
    // let the general translated path remain authoritative.
    //
    // The minor perturbation is far below a binary64 midpoint around 1/3, so the
    // correctly rounded represented-sample SE remains 0x3fd5555555555555.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("inexact minor square must fall back without rejecting the sample");
    assert_eq!(standard_error.to_bits(), 0x3fd5_5555_5555_5555);

    let permuted_recovered = [minor, 0.0, 1.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted_recovered)
        .expect("permutation preserves the represented three-level geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
