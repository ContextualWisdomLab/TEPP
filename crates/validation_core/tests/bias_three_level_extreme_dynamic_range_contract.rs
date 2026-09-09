//! Public contract for three-level normalization refusal at extreme residual scale.

use validation_core::{ValidationError, bias_standard_error};

#[test]
fn three_level_extreme_dynamic_range_refuses_loss_or_returns_exact_finite_result() {
    let huge = f64::from_bits(0x7fe0_0000_0000_0000); // 2^1023
    let tiny = f64::from_bits(0x0000_0000_0000_0001); // 2^-1074
    let truth = [0.0; 3];
    let recovered = [0.0, tiny, huge];

    // Canonical translation around zero preserves all three represented values.
    // The direct three-level proof must retry after raw square overflow, but the
    // exact 2^1023 normalization would erase the nonzero minimum subnormal. That
    // proof must refuse rather than collapse a represented level to zero. If the
    // subsequent fallback can retain the geometry, the exact target for [0,b,A]
    // satisfies SE(mean)^2=(A^2+b^2-Ab)/9 and rounds to A/3 here; otherwise the
    // public contract must fail closed. Either outcome must be permutation-stable.
    let result = bias_standard_error(&truth, &recovered);
    let permuted_recovered = [huge, 0.0, tiny];
    let permuted_result = bias_standard_error(&truth, &permuted_recovered);
    assert_eq!(permuted_result, result);

    match result {
        Ok(standard_error) => assert_eq!(standard_error.to_bits(), 0x7fc5_5555_5555_5555),
        Err(error) => assert_eq!(error, ValidationError::InvalidInput),
    }
}
