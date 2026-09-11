//! Regression contracts for exact bias-SE recovery beyond pairwise-f64 admission.
//!
//! Issue #491 previously characterized a minimum-anchor linear proof. The first
//! fixture shows why production admission must search represented anchors rather
//! than force the minimum. The second shows that represented anchors alone are
//! still incomplete: no observed residual is an exact universal anchor, while
//! the neutral dyadic anchor `0` preserves every residual exactly. Both fixtures
//! have exact pair numerators that fit the bounded proof and both differ by one
//! ULP from the predecessor translated floating-moment fallback.

use validation_core::bias_standard_error;

#[test]
fn exact_nonminimum_anchor_recovers_correctly_rounded_four_observation_bias_se() {
    let diameter = 9_007_199_254_740_992.0_f64; // 2^53
    let truth = [0.0; 4];
    let recovered = [0.0, 1.0, 2.0, -diameter];

    let forward = bias_standard_error(&truth, &recovered)
        .expect("the exact represented residual geometry is scientifically computable");
    assert_eq!(forward.to_bits(), 0x4320_0000_0000_0001);

    let reversed = [recovered[3], recovered[2], recovered[1], recovered[0]];
    let reverse = bias_standard_error(&truth, &reversed)
        .expect("permutation must preserve the exact represented geometry");
    assert_eq!(reverse.to_bits(), forward.to_bits());
}

#[test]
fn exact_zero_anchor_recovers_when_no_observed_residual_is_a_universal_anchor() {
    let tiny = 2.0_f64.powi(-54);
    let truth = [0.0; 4];
    let recovered = [1.0, tiny, 2.0, 3.0];

    let forward = bias_standard_error(&truth, &recovered)
        .expect("neutral-anchor exact geometry is scientifically computable");
    assert_eq!(forward.to_bits(), 0x3fe4_a7e9_cb8a_3491);

    let permuted = [recovered[2], recovered[0], recovered[3], recovered[1]];
    let permuted_result = bias_standard_error(&truth, &permuted)
        .expect("neutral-anchor proof must remain permutation invariant");
    assert_eq!(permuted_result.to_bits(), forward.to_bits());
}
