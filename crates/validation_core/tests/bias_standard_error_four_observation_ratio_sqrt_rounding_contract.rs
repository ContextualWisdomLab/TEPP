//! Exact four-observation ratio/square-root single-rounding contract.
//!
//! For represented residuals `[0, 1, 2, 7]`, the standard error is
//! `sqrt(29 / 12)`. The bounded pair-distance proof must round that mathematical
//! value once and preserve the result under permutation and sign mirroring.

use validation_core::bias_standard_error;

fn assert_four_observation_contract(recovered: [f64; 4]) {
    let truth = [0.0; 4];
    let standard_error = bias_standard_error(&truth, &recovered).expect("representable SE");
    assert_eq!(
        standard_error.to_bits(),
        0x3ff8_df7d_a2e6_6e88,
        "exact represented-input SE is sqrt(29/12); ratio-first sqrt must not round one ULP low"
    );
}

#[test]
fn exact_four_observation_dispersion_avoids_ratio_sqrt_double_rounding() {
    let samples = [
        [0.0, 1.0, 2.0, 7.0],
        [7.0, 2.0, 1.0, 0.0],
        [1.0, 7.0, 0.0, 2.0],
        [2.0, 0.0, 7.0, 1.0],
    ];
    for recovered in samples {
        assert_four_observation_contract(recovered);
        assert_four_observation_contract(recovered.map(|value| -value));
    }
}
