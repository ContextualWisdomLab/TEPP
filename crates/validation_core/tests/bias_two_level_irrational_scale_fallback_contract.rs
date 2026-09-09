//! Regression for a repeated two-level bias-SE sample whose count scale is not a rational square.

use validation_core::bias_standard_error;

#[test]
fn repeated_two_level_irrational_count_scale_preserves_represented_dispersion() {
    let truth = [0.0; 4];
    let recovered = [0.0, 0.0, 1.0, 1.0];

    // With two observations at each exact residual level, SE(mean)^2 is 1/12.
    // The count-only scale is therefore not an exact rational square, so the
    // two-level shortcut must refuse that scale and preserve the same represented
    // result through the general translated-moment path.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("repeated two-level dispersion remains representable");
    assert_eq!(standard_error.to_bits(), 0x3fd2_79a7_4590_331c);

    let permuted = [1.0, 0.0, 1.0, 0.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permutation preserves repeated two-level dispersion");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
