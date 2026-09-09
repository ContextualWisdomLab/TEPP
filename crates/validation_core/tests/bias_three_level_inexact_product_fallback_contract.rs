//! Regression for a three-level bias-SE sample whose exact identity rejects inexact squares.

use validation_core::bias_standard_error;

#[test]
fn inexact_three_level_products_fall_back_without_changing_represented_dispersion() {
    let truth = [0.0; 3];
    let recovered = [0.0, 0.1, 0.2];

    // Canonical translation is [-0.1, 0.0, 0.1]. Squaring the represented
    // 0.1 value is inexact, so the exact three-level identity must refuse and
    // the general translated moment path must still recover the represented-input SE.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("represented three-level dispersion remains finite");
    assert_eq!(standard_error.to_bits(), 0x3fad_8f72_08e6_b82e);

    let permuted = [0.2, 0.0, 0.1];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permutation preserves represented three-level dispersion");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
