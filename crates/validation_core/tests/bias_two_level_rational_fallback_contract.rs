//! Public contracts for exact two-level rational-scale fallback paths.

use validation_core::bias_standard_error;

#[test]
fn balanced_ten_observation_two_level_sample_preserves_one_sixth_standard_error() {
    let truth = [0.0; 10];
    let recovered = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0];

    // Five observations at each represented level give
    // SE(mean)^2 = 25 / (10^2 * 9) = 1/36, hence SE(mean) = 1/6.
    // The count-only rational scale is exact, while a normal-magnitude gap is
    // outside the subnormal-unit shortcut and must use the ordinary exact
    // sum-over-count restoration without changing the represented result.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("balanced represented two-level dispersion remains finite");
    assert_eq!(standard_error.to_bits(), 0x3fc5_5555_5555_5555);

    let permuted = [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0];
    let permuted_standard_error = bias_standard_error(&truth, &permuted)
        .expect("permutation preserves represented two-level dispersion");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
