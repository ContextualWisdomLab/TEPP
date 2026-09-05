//! Exact represented-input admission for mean-bias standard error.
//!
//! The general bias implementation remains the fallback authority. This module
//! admits only a bounded four-observation identity whose residual and pairwise
//! differences are proven exact in binary64 and whose dyadic pair-distance
//! numerator fits `u128`; the exact rational square root is then rounded against
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
    const MAX_EXACT_BINARY64_INTEGER: u128 = 1_u128 << 53;
    if numerator == 0
        || denominator == 0
        || numerator > MAX_EXACT_BINARY64_INTEGER
        || denominator > MAX_EXACT_BINARY64_INTEGER
    {
        return None;
    }
    let unit = exact_power_of_two(unit_exponent)?;
    let denominator_f64 = denominator as f64;
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

fn exact_four_observation_standard_error(
    truth: &[f64],
    recovered: &[f64],
) -> Option<Result<f64, ValidationError>> {
    if truth.len() != 4 || recovered.len() != 4 {
        return None;
    }

    let mut residuals = [0.0; 4];
    for index in 0..4 {
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
        residuals[index] = residual;
    }

    let mut pair_dyadics = Vec::with_capacity(6);
    let mut unit_exponent = i32::MAX;
    for left in 0..4 {
        for right in left + 1..4 {
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

    // For n=4, sum((ri-rj)^2, i<j) / 48 is exactly SE(mean)^2.
    let standard_error = correctly_rounded_scaled_sqrt_ratio(pair_square_sum, 48, unit_exponent)?;
    Some(Ok(standard_error))
}

/// Standard error of mean signed bias.
///
/// Four-observation samples whose represented residuals and pairwise differences
/// are exact use the exact pair-distance identity when its dyadic numerator fits
/// the bounded integer proof. All other samples retain the established bias
/// implementation and its existing fail-closed behavior.
pub fn bias_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    if let Some(result) = exact_four_observation_standard_error(truth, recovered) {
        return result;
    }
    crate::bias::bias_standard_error(truth, recovered)
}

#[cfg(test)]
mod tests {
    use super::{
        correctly_rounded_scaled_sqrt_ratio, exact_four_observation_standard_error,
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
    }

    #[test]
    fn exact_ratio_sqrt_refuses_outside_bounded_proof() {
        let too_large = (1_u128 << 53) + 1;
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(0, 48, 0), None);
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(1, 0, 0), None);
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(too_large, 48, 0), None);
        assert_eq!(correctly_rounded_scaled_sqrt_ratio(1, too_large, 0), None);
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
            exact_four_observation_standard_error(&truth, &recovered)
                .expect("admitted")
                .expect("representable")
                .to_bits(),
            0x3ff8_df7d_a2e6_6e88
        );

        let unit = 2.0_f64.powi(400);
        let scaled = recovered.map(|value| value * unit);
        let expected = f64::from_bits(0x58f8_df7d_a2e6_6e88);
        assert_eq!(
            exact_four_observation_standard_error(&truth, &scaled)
                .expect("scaled admitted")
                .expect("scaled representable"),
            expected
        );
    }

    #[test]
    fn four_observation_identity_covers_exact_zero_and_fallbacks() {
        let truth = [0.0; 4];
        assert_eq!(
            exact_four_observation_standard_error(&truth, &[3.0; 4]),
            Some(Ok(0.0))
        );
        assert_eq!(
            exact_four_observation_standard_error(&truth[..3], &[0.0; 3]),
            None
        );
        assert_eq!(
            exact_four_observation_standard_error(&truth, &[0.0, 1.0, f64::INFINITY, 2.0]),
            None
        );

        let tiny = 2.0_f64.powi(-54);
        assert_eq!(
            exact_four_observation_standard_error(&truth, &[0.0, 1.0, tiny, 2.0]),
            None
        );
        assert_eq!(
            exact_four_observation_standard_error(&[1.0, 0.0, 0.0, 0.0], &[tiny, 0.0, 0.0, 0.0]),
            None
        );
    }
}
