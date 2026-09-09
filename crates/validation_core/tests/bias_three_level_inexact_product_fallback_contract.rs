//! Regressions for three-level bias-SE samples whose exact shortcut must refuse safely.

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

#[test]
fn exact_three_level_products_fall_back_when_symmetric_square_sum_overflows() {
    let truth = [0.0; 3];
    let magnitude = f64::from_bits(0x5fe8_0000_0000_0000); // 1.5 × 2^511
    let recovered = [-magnitude, 0.0, magnitude];

    // Canonical translation anchors at zero, so the shortcut sees offsets
    // [-magnitude, magnitude]. Each square and the cross product is finite and
    // exact, while square + square overflows binary64. The represented-input
    // identity is SE(mean) = magnitude / sqrt(3), which remains finite and rounds
    // to the audited binary64 below. The shortcut must refuse only its unsafe
    // intermediate and let the general translated-moment path preserve the result.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("representable symmetric dispersion survives square-sum overflow");
    assert_eq!(standard_error.to_bits(), 0x5fdb_b67a_e858_4caa);

    let permuted = [magnitude, -magnitude, 0.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permutation preserves symmetric square-sum-overflow dispersion");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
