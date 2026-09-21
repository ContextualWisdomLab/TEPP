//! Public contract for exact residual dispersion when anchor translation overflows.

use validation_core::bias_standard_error;

#[test]
fn combined_high_low_translation_overflow_preserves_exact_residual_dispersion() {
    let dominant = f64::from_bits(0x7fd0_0000_0000_0000);
    let opposite = f64::from_bits(0xffe7_ffff_ffff_ffff);
    let low_positive = f64::from_bits(0x7c70_0000_0000_0000);
    let low_negative_magnitude = f64::from_bits(0x7c88_0000_0000_0000);

    // Let u = 2^968. These represented observation pairs have exact residual
    // coefficients [2^54 + 1, -3*2^54 + 5, 0] in units of u. Their rounded
    // high residuals remain finite, but translating the first two high parts
    // produces f64::MAX while the corresponding low-term delta is 2^970, so
    // recombining that exact anchor-relative residual would overflow binary64.
    // That failed translation must not make the rounded high parts authoritative:
    // the exact pair-distance numerator is
    // 8_437_482_395_119_093_815_498_145_966_063_658, and
    // SE(mean)^2 = P*u^2/18 rounds to the binary64 value below.
    let truth = [-low_positive, low_negative_magnitude, 0.0];
    let recovered = [dominant, opposite, 0.0];
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("representable exact residual dispersion survives translation overflow");
    assert_eq!(standard_error.to_bits(), 0x7fd3_3ac7_82eb_914d);

    let permuted_truth = [0.0, -low_positive, low_negative_magnitude];
    let permuted_recovered = [0.0, dominant, opposite];
    let permuted_standard_error = bias_standard_error(&permuted_truth, &permuted_recovered)
        .expect("permutation preserves exact residual dispersion");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
