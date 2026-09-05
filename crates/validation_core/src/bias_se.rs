//! Exact represented-input admission for mean-bias standard error.
//!
//! The general bias implementation remains the fallback authority. This module
//! admits a bounded small-sample pair-distance identity whose residual and pairwise
//! differences are proven exact in binary64 and whose reduced dyadic pair-distance
//! ratio fits `u128`; the exact rational square root is then rounded against
//! binary64 midpoints without first rounding the ratio under the square root.

use crate::ValidationError;
use core::cmp::Ordering;

fn subtraction_roundoff(recovered: f64, truth: f64, residual: f64) -> f64 {
    let negated_truth = -truth;
    let truth_virtual = residual - recovered;
    let recovered_virtual = residual - truth_virtual;
    let recovered_roundoff = recovered - recovered_virtual;
    let truth_roundoff = negated_truth - truth_virtual;
    recovered_roundoff + truth_roundoff
}

fn positive_dyadic(value: f64) -> Option<(u128, i32)> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let bits = value.to_bits();
    let exponent_bits = i32::try_from((bits >> 52) & 0x7ff).ok()?;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    let (mut significand, mut exponent) = if exponent_bits == 0 {
        (u128::from(fraction), -1074)
    } else {
        (
            u128::from((1_u64 << 52) | fraction),
            exponent_bits - 1023 - 52,
        )
    };
    if significand == 0 {
        return None;
    }
    let trailing = significand.trailing_zeros();
    significand >>= trailing;
    exponent += i32::try_from(trailing).ok()?;
    Some((significand, exponent))
}

fn multiply_by_power_of_two(value: u128, shift: u32) -> Option<u128> {
    let factor = 1_u128.checked_shl(shift)?;
    value.checked_mul(factor)
}

fn compare_scaled_ratio_to_dyadic_square(
    numerator: u128,
    numerator_exponent: i32,
    denominator: u128,
    significand: u128,
    exponent: i32,
) -> Option<Ordering> {
    let square = significand.checked_mul(significand)?;
    let right = denominator.checked_mul(square)?;
    let square_exponent = exponent.checked_mul(2)?;
    let exponent_delta = numerator_exponent.checked_sub(square_exponent)?;
    if exponent_delta >= 0 {
        let left = multiply_by_power_of_two(numerator, exponent_delta.unsigned_abs())?;
        Some(left.cmp(&right))
    } else {
        let shifted_right = multiply_by_power_of_two(right, exponent_delta.unsigned_abs())?;
        Some(numerator.cmp(&shifted_right))
    }
}

fn midpoint_dyadic(left: f64, right: f64) -> Option<(u128, i32)> {
    let (left_significand, left_exponent) = positive_dyadic(left)?;
    let (right_significand, right_exponent) = positive_dyadic(right)?;
    let common_exponent = left_exponent.min(right_exponent);
    let left_shift = left_exponent.checked_sub(common_exponent)?.unsigned_abs();
    let right_shift = right_exponent.checked_sub(common_exponent)?.unsigned_abs();
    let left_units = multiply_by_power_of_two(left_significand, left_shift)?;
    let right_units = multiply_by_power_of_two(right_significand, right_shift)?;
    let mut midpoint_significand = left_units.checked_add(right_units)?;
    let mut midpoint_exponent = common_exponent.checked_sub(1)?;
    let trailing = midpoint_significand.trailing_zeros();
    midpoint_significand >>= trailing;
    midpoint_exponent += i32::try_from(trailing).ok()?;
    Some((midpoint_significand, midpoint_exponent))
}

fn exact_power_of_two(exponent: i32) -> Option<f64> {
    if (-1022..=1023).contains(&exponent) {
        let biased_exponent = u64::try_from(exponent + 1023).ok()?;
        return Some(f64::from_bits(biased_exponent << 52));
    }
    if (-1074..=-1023).contains(&exponent) {
        let shift = u32::try_from(exponent + 1074).ok()?;
        return Some(f64::from_bits(1_u64 << shift));
    }
    None
}

fn correctly_rounded_scaled_sqrt_ratio(
    numerator: u128,
    denominator: u128,
    unit_exponent: i32,
) -> Option<f64> {
    const MAX_EXACT_BINARY64_DENOMINATOR: u128 = 1_u128 << 53;
    if numerator == 0 || denominator == 0 || denominator > MAX_EXACT_BINARY64_DENOMINATOR {
        return None;
    }
    let unit = exact_power_of_two(unit_exponent)?;
    let denominator_f64 = denominator as f64;
    // The binary64 numerator conversion is only a seed. The returned value is
    // admitted solely after the exact u128 dyadic-square and midpoint comparisons
    // below. This lets the bounded proof retain exact reduced numerators above
    // 2^53 without pretending that their seed conversion is exact.
    let mut candidate = ((numerator as f64) / denominator_f64).sqrt() * unit;
    if !candidate.is_finite() || candidate <= 0.0 {
        return None;
    }
    let target_exponent = unit_exponent.checked_mul(2)?;

    for _ in 0..4 {
        let (candidate_significand, candidate_exponent) = positive_dyadic(candidate)?;
        let candidate_comparison = compare_scaled_ratio_to_dyadic_square(
            numerator,
            target_exponent,
            denominator,
            candidate_significand,
            candidate_exponent,
        )?;
        if candidate_comparison == Ordering::Equal {
            return Some(candidate);
        }

        let upward = candidate_comparison == Ordering::Greater;
        let bits = candidate.to_bits();
        let neighbor = if upward {
            f64::from_bits(bits.checked_add(1)?)
        } else {
            if bits == 1 {
                return None;
            }
            f64::from_bits(bits - 1)
        };
        if !neighbor.is_finite() || neighbor <= 0.0 {
            return None;
        }
        let (midpoint_significand, midpoint_exponent) = midpoint_dyadic(candidate, neighbor)?;
        let midpoint_comparison = compare_scaled_ratio_to_dyadic_square(
            numerator,
            target_exponent,
            denominator,
            midpoint_significand,
            midpoint_exponent,
        )?;

        let neighbor_is_closer = if upward {
            midpoint_comparison == Ordering::Greater
        } else {
            midpoint_comparison == Ordering::Less
        };
        if neighbor_is_closer {
            candidate = neighbor;
            continue;
        }
        if midpoint_comparison == Ordering::Equal && candidate.to_bits() & 1 == 1 {
            return Some(neighbor);
        }
        return Some(candidate);
    }
    None
}

fn exact_pair_distance_standard_error(
    truth: &[f64],
    recovered: &[f64],
) -> Option<Result<f64, ValidationError>> {
    // Keep this O(n²) reference proof deliberately bounded. n=2 and n=3 have
    // cheaper exact identities in `bias.rs`; four through eight observations are
    // the smallest remaining sample sizes with demonstrated one-ULP errors in
    // the translated floating moment/sqrt path.
    if truth.len() != recovered.len() || !(4..=8).contains(&truth.len()) {
        return None;
    }
    let sample_count = truth.len();

    let mut residuals = Vec::with_capacity(sample_count);
    for index in 0..sample_count {
        let truth_value = truth[index];
        let recovered_value = recovered[index];
        if !truth_value.is_finite() || !recovered_value.is_finite() {
            return None;
        }
        let residual = recovered_value - truth_value;
        if !residual.is_finite()
            || subtraction_roundoff(recovered_value, truth_value, residual) != 0.0
        {
            return None;
        }
        residuals.push(residual);
    }

    let pair_count = sample_count.checked_mul(sample_count.checked_sub(1)?)? / 2;
    let mut pair_dyadics = Vec::with_capacity(pair_count);
    let mut unit_exponent = i32::MAX;
    for left in 0..sample_count {
        for right in left + 1..sample_count {
            let difference = residuals[left] - residuals[right];
            if !difference.is_finite()
                || subtraction_roundoff(residuals[left], residuals[right], difference) != 0.0
            {
                return None;
            }
            if difference == 0.0 {
                pair_dyadics.push(None);
                continue;
            }
            let dyadic = positive_dyadic(difference.abs())?;
            unit_exponent = unit_exponent.min(dyadic.1);
            pair_dyadics.push(Some(dyadic));
        }
    }
    if unit_exponent == i32::MAX {
        return Some(Ok(0.0));
    }

    let mut pair_square_sum = 0_u128;
    for dyadic in pair_dyadics.into_iter().flatten() {
        let shift = dyadic.1.checked_sub(unit_exponent)?.unsigned_abs();
        let coefficient = multiply_by_power_of_two(dyadic.0, shift)?;
        let square = coefficient.checked_mul(coefficient)?;
        pair_square_sum = pair_square_sum.checked_add(square)?;
    }
    if pair_square_sum == 0 {
        return Some(Ok(0.0));
    }

    // For n observations, sum((ri-rj)^2, i<j) / [n²(n-1)] is exactly
    // SE(mean)^2. Reduce that rational before the bounded midpoint proof. The
    // reduced numerator remains an exact u128 authority; its binary64 conversion
    // is only the candidate seed.
    let sample_count_u128 = sample_count as u128;
    let denominator = sample_count_u128
        .checked_mul(sample_count_u128)?
        .checked_mul(sample_count_u128.checked_sub(1)?)?;
    let mut divisor_left = pair_square_sum;
    let mut divisor_right = denominator;
    while divisor_right != 0 {
        let remainder = divisor_left % divisor_right;
        divisor_left = divisor_right;
        divisor_right = remainder;
    }
    let reduced_numerator = pair_square_sum / divisor_left;
    let reduced_denominator = denominator / divisor_left;
    let standard_error = correctly_rounded_scaled_sqrt_ratio(
        reduced_numerator,
        reduced_denominator,
        unit_exponent,
    )?;
    Some(Ok(standard_error))
}

/// Standard error of mean signed bias.
///
/// Four- through eight-observation samples whose represented residuals and
/// pairwise differences are exact use the exact pair-distance identity when its
/// reduced dyadic ratio fits the bounded integer proof. All other samples retain
/// the established bias implementation and its existing fail-closed behavior.
pub fn bias_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    if let Some(result) = exact_pair_distance_standard_error(truth, recovered) {
        return result;
    }
    crate::bias::bias_standard_error(truth, recovered)
}

#[cfg(test)]
mod tests {
    use super::{
        correctly_rounded_scaled_sqrt_ratio, exact_pair_distance_standard_error,
        exact_power_of_two, midpoint_dyadic, multiply_by_power_of_two, positive_dyadic,
    };

    #[test]
    fn exact_ratio_sqrt_corrects_both_adjacent_rounding_directions() {
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(116, 48, 0)
                .expect("bounded exact ratio")
                .to_bits(),
            0x3ff8_df7d_a2e6_6e88
        );
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(136, 48, 0)
                .expect("bounded exact ratio")
                .to_bits(),
            0x3ffa_ee98_6a40_25f8
        );
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(48, 48, 0),
            Some(1.0)
        );
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(1_739_374_438_758_325_417, 16, 0)
                .expect("large exact numerator remains bounded by u128 midpoint proof")
                .to_bits(),
            0x41b3_a706_d408_9e32
        );
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(155_324_619_328_335_851, 25, 0)
                .expect("five-observation reduced ratio remains bounded")
                .to_bits(),
            0x4192_caf1_6406_5ad0
        );
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(621_603_287_214_182_303, 45, 0)
                .expect("six-observation reduced ratio remains bounded")
                .to_bits(),
            0x419c_057d_42fc_5857
        );
    }

    #[test]
    fn exact_ratio_sqrt_refuses_outside_bounded_proof() {
        let too_large_denominator = (1_u128 << 53) + 1;
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(0, 48, 0), None);
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(1, 0, 0), None);
        assert_eq!(
            correctly_rounded_scaled_sqrt_ratio(1, too_large_denominator, 0),
            None
        );
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(1, 48, 1024), None);
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(1, 48, -1075), None);
    }

    #[test]
    fn dyadic_helpers_cover_normal_subnormal_and_refusal_boundaries() {
        assert_eq!(positive_dyadic(1.0), Some((1, 0)));
        assert_eq!(positive_dyadic(f64::from_bits(1)), Some((1, -1074)));
        assert_eq!(positive_dyadic(0.0), None);
        assert_eq!(positive_dyadic(-1.0), None);
        assert_eq!(positive_dyadic(f64::INFINITY), None);
        assert_eq!(exact_power_of_two(0), Some(1.0));
        assert_eq!(exact_power_of_two(-1074), Some(f64::from_bits(1)));
        assert_eq!(exact_power_of_two(1024), None);
        assert_eq!(exact_power_of_two(-1075), None);
        assert_eq!(multiply_by_power_of_two(3, 2), Some(12));
        assert_eq!(multiply_by_power_of_two(1, 128), None);
        assert_eq!(multiply_by_power_of_two(u128::MAX, 1), None);
        assert!(midpoint_dyadic(1.0, f64::from_bits(1.0_f64.to_bits() + 1)).is_some());
        assert_eq!(midpoint_dyadic(0.0, f64::from_bits(1)), None);
    }

    #[test]
    fn four_observation_identity_is_power_of_two_scale_invariant() {
        let truth = [0.0; 4];
        let recovered = [0.0, 1.0, 2.0, 7.0];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("admitted")
                .expect("representable")
                .to_bits(),
            0x3ff8_df7d_a2e6_6e88
        );

        let unit = 2.0_f64.powi(400);
        let scaled = recovered.map(|value| value * unit);
        let expected = f64::from_bits(0x58f8_df7d_a2e6_6e88);
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &scaled)
                .expect("scaled admitted")
                .expect("scaled representable"),
            expected
        );
    }

    #[test]
    fn four_observation_identity_reduces_the_exact_ratio_before_bounded_admission() {
        let truth = [0.0; 4];
        let recovered = [0.0, 14_099_687.0, 16_729_100.0, 94_045_527.0];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("reduced ratio admitted")
                .expect("representable")
                .to_bits(),
            0x4174_46e5_76f8_7445
        );
    }

    #[test]
    fn four_observation_identity_keeps_large_reduced_numerator_in_bounded_proof() {
        let truth = [0.0; 4];
        let recovered = [19_274_968.0, 693_729_138.0, 711_353_557.0, 1_625_519_116.0];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("large reduced ratio admitted")
                .expect("representable")
                .to_bits(),
            0x41b3_a706_d408_9e32
        );
    }

    #[test]
    fn five_observation_identity_keeps_exact_pair_distance_ratio_authoritative() {
        let truth = [0.0; 5];
        let recovered = [
            1_342_748_146.0,
            1_434_848_064.0,
            1_525_257_611.0,
            1_685_877_224.0,
            1_771_341_094.0,
        ];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("five-observation ratio admitted")
                .expect("representable")
                .to_bits(),
            0x4192_caf1_6406_5ad0
        );
    }

    #[test]
    fn six_observation_identity_keeps_exact_pair_distance_ratio_authoritative() {
        let truth = [0.0; 6];
        let recovered = [
            1_120_315_269.0,
            1_513_609_015.0,
            1_569_037_659.0,
            1_789_057_504.0,
            1_807_936_669.0,
            1_914_796_738.0,
        ];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &recovered)
                .expect("six-observation ratio admitted")
                .expect("representable")
                .to_bits(),
            0x419c_057d_42fc_5857
        );
    }

    #[test]
    fn bounded_pair_distance_identity_covers_exact_zero_and_fallbacks() {
        let truth = [0.0; 4];
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[3.0; 4]),
            Some(Ok(0.0))
        );
        assert_eq!(
            exact_pair_distance_standard_error(&truth[..3], &[0.0; 3]),
            None
        );
        assert_eq!(
            exact_pair_distance_standard_error(&[0.0; 8], &[0.0; 8]),
            Some(Ok(0.0))
        );
        assert_eq!(
            exact_pair_distance_standard_error(&[0.0; 9], &[0.0; 9]),
            None
        );
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[0.0, 1.0, f64::INFINITY, 2.0]),
            None
        );
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[f64::MAX, -f64::MAX, 0.0, 0.0]),
            None
        );

        let tiny = 2.0_f64.powi(-54);
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[0.0, 1.0, tiny, 2.0]),
            None
        );
        assert_eq!(
            exact_pair_distance_standard_error(&[1.0, 0.0, 0.0, 0.0], &[tiny, 0.0, 0.0, 0.0]),
            None
        );
        assert_eq!(
            exact_pair_distance_standard_error(&truth, &[0.0, 1.0, 2.0, 67_108_864.0]),
            None
        );
    }
}
