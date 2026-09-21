//! Regression contract for a subnormal standard error that rounds to one binary64 unit.
//!
//! For four represented residual coefficients `[2, -1, 0, 0]` on the minimum-
//! subnormal unit, the exact pair-distance numerator is `P = 19` and the
//! denominator is `48`. Thus `SE(mean) = sqrt(19 / 48) * 2^-1074`, which lies
//! above the zero/minimum-subnormal midpoint and must round to one minimum
//! subnormal rather than fail closed or collapse to zero.

use validation_core::bias_standard_error;

#[test]
fn subnormal_standard_error_rounds_once_to_minimum_binary64() {
    let minimum_subnormal = f64::from_bits(1);
    let truth = [0.0; 4];
    let recovered = [f64::from_bits(2), -minimum_subnormal, 0.0, 0.0];

    assert_eq!(
        bias_standard_error(&truth, &recovered)
            .expect("represented standard error")
            .to_bits(),
        minimum_subnormal.to_bits()
    );

    let permuted = [0.0, f64::from_bits(2), 0.0, -minimum_subnormal];
    assert_eq!(
        bias_standard_error(&truth, &permuted)
            .expect("permutation-invariant represented standard error")
            .to_bits(),
        minimum_subnormal.to_bits()
    );
}
