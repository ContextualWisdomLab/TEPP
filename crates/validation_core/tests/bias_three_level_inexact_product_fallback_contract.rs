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
fn exact_three_level_products_fall_back_when_their_square_sum_overflows() {
    let truth = [0.0; 3];
    let first = f64::from_bits(0x5fe8_0000_0000_0000); // 1.5 × 2^511
    let second = f64::from_bits(0x5fe6_0000_0000_0000); // 1.375 × 2^511
    let recovered = [0.0, first, second];

    // Both squares and the cross product are finite and exactly represented,
    // but first² + second² exceeds binary64 even though
    // first² + second² - first·second, and therefore SE(mean), are representable.
    // The exact represented-input identity rounds to the audited binary64 below;
    // the shortcut must refuse the overflowing intermediate and preserve it via
    // the general power-of-two translated-moment path.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("representable three-level dispersion survives square-sum overflow");
    assert_eq!(standard_error.to_bits(), 0x5fce_c0e5_647d_d2ed);

    let permuted = [second, 0.0, first];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permutation preserves overflowing-square-sum dispersion");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
