//! Regression for a three-level bias-SE sample whose exact identity rejects an inexact later square.

use validation_core::bias_standard_error;

#[test]
fn later_inexact_three_level_square_falls_back_without_changing_represented_dispersion() {
    let truth = [0.0; 3];
    let recovered = [-0.03125, 0.0, 0.1];

    // Canonical translation stays [-1/32, 0, 0.1]. The first square is exact,
    // so the three-level proof evaluates the later represented 0.1 square;
    // that square is inexact and must refuse the shortcut before the general
    // translated moment path recovers the represented-input standard error.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("represented three-level dispersion remains finite");
    assert_eq!(standard_error.to_bits(), 0x3fa4_4444_4444_4445);

    let permuted = [0.1, -0.03125, 0.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permutation preserves represented three-level dispersion");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
