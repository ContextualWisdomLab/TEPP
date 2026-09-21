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
fn distinct_exact_three_level_values_exit_two_level_state() {
    let truth = [0.0; 3];
    let recovered = [0.0, 1.0, 2.0];

    // Canonical translation anchors at the middle level and produces [-1, 0, 1].
    // The first nonzero value establishes the candidate repeated gap; the later
    // opposite-signed nonzero value is a distinct represented level, so the
    // two-level state must be abandoned before the three-level shortcut is tried.
    // Its radicand is exactly 3 but sqrt(3) is not an exact binary64 square root,
    // leaving the general translated moment path to return sqrt(1/3).
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("distinct represented levels retain finite dispersion");
    assert_eq!(standard_error.to_bits(), 0x3fe2_79a7_4590_331c);

    let permuted = [2.0, 0.0, 1.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("canonical translation preserves permutation invariance");
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

#[test]
fn exact_three_level_products_fall_back_when_only_radicand_overflows() {
    let truth = [0.0; 3];
    let magnitude = f64::from_bits(0x5fe4_0000_0000_0000); // 1.25 × 2^511
    let recovered = [-magnitude, 0.0, magnitude];

    // Each represented square and their sum remain finite and exact at this
    // magnitude, while subtracting the negative exact cross-product overflows.
    // The exact three-level shortcut must refuse that unsafe radicand without
    // rejecting the finite represented-input target; the translated moment path
    // returns magnitude / sqrt(3), correctly rounded below.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("representable dispersion survives radicand overflow");
    assert_eq!(standard_error.to_bits(), 0x5fd7_1811_16f4_3fe3);

    let permuted = [magnitude, -magnitude, 0.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permutation preserves radicand-overflow dispersion");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
