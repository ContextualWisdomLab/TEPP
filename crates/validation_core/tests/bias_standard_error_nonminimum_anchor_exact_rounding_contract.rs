//! Regression contract for exact bias-SE recovery when the minimum residual is not an exact anchor.
//!
//! Issue #491 previously characterized a minimum-anchor linear proof. This fixture
//! shows why production admission must instead search deterministic exact anchors:
//! subtracting the minimum residual `-2^53` from `1` rounds, while anchor `0`
//! preserves every translated coordinate exactly. The exact pair numerator is
//! `243388915243820099130562543878155`, so `SE(mean)^2 = P / 48` and the
//! correctly rounded binary64 result is one ULP above the translated floating
//! moment fallback. Observation order must not change that scientific result.

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
