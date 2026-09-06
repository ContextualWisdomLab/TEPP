//! Characterizes an anchor-linear exact proof that is stricter about coordinates than pair subtraction.
//!
//! This is test-only evidence for issue #491. Production admission remains
//! `n=4..=16` and the current pairwise proof remains the fail-closed authority.
//! The fixture isolates a represented-input geometry where every subtraction from
//! the minimum anchor is exact, while one non-anchor pair subtraction is rounded.
//! Exact integer coordinates still recover the pair-distance numerator through
//! `n * sum(c_i^2) - (sum(c_i))^2`, so pairwise-f64 subtraction exactness is not a
//! scientific prerequisite for this bounded represented geometry.

use validation_core::bias_standard_error;

fn subtraction_roundoff(recovered: f64, truth: f64, residual: f64) -> f64 {
    let negated_truth = -truth;
    let truth_virtual = residual - recovered;
    let recovered_virtual = residual - truth_virtual;
    let recovered_roundoff = recovered - recovered_virtual;
    let truth_roundoff = negated_truth - truth_virtual;
    recovered_roundoff + truth_roundoff
}

fn exact_pair_numerator(coefficients: &[u128]) -> u128 {
    let mut total = 0_u128;
    for left in 0..coefficients.len() {
        for right in left + 1..coefficients.len() {
            let difference = coefficients[left].abs_diff(coefficients[right]);
            total = total
                .checked_add(
                    difference
                        .checked_mul(difference)
                        .expect("bounded coefficient square fits u128"),
                )
                .expect("bounded pair numerator fits u128");
        }
    }
    total
}

#[test]
fn anchor_linear_identity_survives_a_rounded_non_anchor_pair_difference() {
    let tiny = 2.0_f64.powi(-54);
    let residuals = [0.0, 1.0, tiny, 2.0];

    for residual in residuals {
        let anchored = residual - 0.0;
        assert_eq!(
            subtraction_roundoff(residual, 0.0, anchored),
            0.0,
            "minimum-anchor coordinates must be represented exactly"
        );
    }

    let rounded_non_anchor_pair = residuals[1] - residuals[2];
    assert_ne!(
        subtraction_roundoff(residuals[1], residuals[2], rounded_non_anchor_pair),
        0.0,
        "the fixture must remain outside the current pairwise-f64 subtraction proof"
    );

    // Common exact unit is 2^-54, giving integer coordinates
    // [0, 2^54, 1, 2^55].
    let coefficients = [0_u128, 1_u128 << 54, 1, 1_u128 << 55];
    let coefficient_sum = coefficients
        .iter()
        .copied()
        .try_fold(0_u128, |sum, value| sum.checked_add(value))
        .expect("bounded coefficient sum fits u128");
    let square_sum = coefficients
        .iter()
        .copied()
        .try_fold(0_u128, |sum, value| {
            sum.checked_add(value.checked_mul(value)?)
        })
        .expect("bounded squared-coordinate sum fits u128");
    let sample_count = u128::try_from(coefficients.len()).expect("sample count fits u128");
    let linear_numerator = sample_count
        .checked_mul(square_sum)
        .and_then(|scaled_square_sum| {
            coefficient_sum
                .checked_mul(coefficient_sum)
                .and_then(|squared_sum| scaled_square_sum.checked_sub(squared_sum))
        })
        .expect("bounded anchor-linear numerator fits u128");
    let pair_numerator = exact_pair_numerator(&coefficients);

    assert_eq!(linear_numerator, pair_numerator);
    assert_eq!(
        linear_numerator,
        3_569_704_090_242_693_886_528_325_169_446_915_u128
    );

    // The public result already remains numerically correct through the generic
    // fallback. This characterization concerns exact-proof admission and the
    // avoidable O(n^2) pairwise-f64 prerequisite, not a changed public value.
    assert_eq!(
        bias_standard_error(&[0.0; 4], &residuals)
            .expect("represented fixture remains scientifically computable")
            .to_bits(),
        0x3fde_a33e_2c83_c140
    );
}
