//! Signed mean bias recovery metric.

use crate::ValidationError;
use crate::input::require_paired_finite;
use crate::numeric::{
    deterministic_compensated_sum, deterministic_representable_mean,
    deterministic_representable_sum_over_count, exact_power_of_two_scale,
};

fn signed_residuals(truth: &[f64], recovered: &[f64]) -> Result<Vec<f64>, ValidationError> {
    require_paired_finite(truth, recovered)?;
    truth
        .iter()
        .zip(recovered)
        .map(|(truth_value, recovered_value)| {
            let residual = recovered_value - truth_value;
            if residual.is_finite() {
                Ok(residual)
            } else {
                Err(ValidationError::InvalidInput)
            }
        })
        .collect()
}

fn subtraction_roundoff(recovered: f64, truth: f64, residual: f64) -> f64 {
    let negated_truth = -truth;
    let truth_virtual = residual - recovered;
    let recovered_virtual = residual - truth_virtual;
    let recovered_roundoff = recovered - recovered_virtual;
    let truth_roundoff = negated_truth - truth_virtual;
    recovered_roundoff + truth_roundoff
}

fn subtraction_has_roundoff(recovered: f64, truth: f64, residual: f64) -> bool {
    subtraction_roundoff(recovered, truth, residual) != 0.0
}

fn standard_error_from_deviations(deviations: &[f64]) -> Result<f64, ValidationError> {
    let scale = deviations
        .iter()
        .map(|deviation| deviation.abs())
        .fold(0.0, f64::max);
    if scale == 0.0 {
        return Ok(0.0);
    }

    let normalized_squares: Vec<_> = deviations
        .iter()
        .map(|deviation| {
            let normalized = *deviation / scale;
            normalized * normalized
        })
        .collect();
    let square_sum = deterministic_compensated_sum(normalized_squares);
    let sample_count = deviations.len() as f64;
    // Form SE directly as sqrt(sum(d²) / (n * (n - 1))). Separately
    // rounding sqrt(sample_variance) and sqrt(n) can move the final binary64
    // standard error by one ULP even when the represented squared-deviation
    // ratio is exact.
    let normalized_standard_error =
        (square_sum / (sample_count * (sample_count - 1.0))).sqrt();
    let standard_error = scale * normalized_standard_error;
    if !standard_error.is_finite()
        || (standard_error == 0.0 && normalized_standard_error != 0.0)
    {
        Err(ValidationError::InvalidInput)
    } else if standard_error == 0.0 {
        Ok(0.0)
    } else {
        Ok(standard_error)
    }
}

fn scaled_standard_error(values: &[f64], mean: f64) -> Result<f64, ValidationError> {
    let direct_deviations: Option<Vec<f64>> = values
        .iter()
        .map(|value| {
            let deviation = *value - mean;
            deviation.is_finite().then_some(deviation)
        })
        .collect();
    if let Some(deviations) = direct_deviations {
        return standard_error_from_deviations(&deviations);
    }

    let outer_scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if outer_scale == 0.0 {
        return Ok(0.0);
    }
    let normalized_values: Vec<_> = values.iter().map(|value| *value / outer_scale).collect();
    let normalized_mean = deterministic_representable_mean(&normalized_values)?;
    let normalized_deviations: Vec<_> = normalized_values
        .iter()
        .map(|value| *value - normalized_mean)
        .collect();
    let normalized_standard_error = standard_error_from_deviations(&normalized_deviations)?;
    let standard_error = outer_scale * normalized_standard_error;
    if !standard_error.is_finite()
        || (standard_error == 0.0 && normalized_standard_error != 0.0)
    {
        Err(ValidationError::InvalidInput)
    } else if standard_error == 0.0 {
        Ok(0.0)
    } else {
        Ok(standard_error)
    }
}

fn greatest_common_divisor(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

/// Return the exact rational scale implied by two-level sample counts.
///
/// For two exact residual levels with counts `m` and `n - m`,
/// `SE(mean)^2 = gap^2 * m(n-m) / (n^2(n-1))`. If the reduced count-only
/// factor is exactly the square of a rational `numerator / denominator` whose
/// roots fit the platform's exact integer count representation, callers can
/// preserve that algebraic scale without reconstructing it through rounded
/// sums, squares and sqrt.
fn exact_two_level_rational_scale(
    first_count: usize,
    second_count: usize,
) -> Option<(usize, usize)> {
    let sample_count = (first_count as u128).checked_add(second_count as u128)?;
    let count_product = (first_count as u128).checked_mul(second_count as u128)?;
    if count_product == 0 {
        return None;
    }
    let target = sample_count
        .checked_mul(sample_count)?
        .checked_mul(sample_count.checked_sub(1)?)?;

    let divisor = greatest_common_divisor(count_product, target);
    let reduced_numerator = count_product / divisor;
    let reduced_denominator = target / divisor;
    let numerator = reduced_numerator.isqrt();
    let denominator = reduced_denominator.isqrt();
    if numerator.checked_mul(numerator)? != reduced_numerator
        || denominator.checked_mul(denominator)? != reduced_denominator
        || numerator == 0
        || denominator == 0
        || numerator > usize::MAX as u128
        || denominator > usize::MAX as u128
    {
        return None;
    }
    Some((numerator as usize, denominator as usize))
}

/// Round `|gap| * numerator / denominator` directly in minimum-subnormal units
/// when the exact rational result lies at or below the normal/subnormal boundary.
///
/// A normalized binary64 quotient can be correctly rounded in its working
/// binade and still move by one ULP when an exact power-of-two restoration enters
/// the subnormal range. On supported targets a represented significand is at
/// most 53 bits and a `usize` factor at most 64 bits. The only potentially large
/// operation is the exponent shift; if it cannot fit `u128`, the exact result is
/// necessarily above this bounded subnormal path and the caller keeps its normal
/// overflow-safe implementation.
fn exact_subnormal_rational_scale(
    gap: f64,
    numerator: usize,
    denominator: usize,
) -> Option<Result<f64, ValidationError>> {
    if !gap.is_finite() || gap == 0.0 || numerator == 0 || denominator == 0 {
        return None;
    }

    let magnitude_bits = gap.abs().to_bits();
    let exponent = ((magnitude_bits >> 52) & 0x7ff) as u32;
    let fraction = magnitude_bits & 0x000f_ffff_ffff_ffff;
    let significand = if exponent == 0 {
        fraction as u128
    } else {
        ((1_u64 << 52) | fraction) as u128
    };
    let product = significand * numerator as u128;
    let unit_shift = if exponent == 0 { 0 } else { exponent - 1 };
    let scaled_numerator = product.checked_shl(unit_shift)?;
    let denominator = denominator as u128;

    let mut rounded_units = scaled_numerator / denominator;
    let remainder = scaled_numerator % denominator;
    let twice_remainder = remainder * 2;
    if twice_remainder > denominator
        || (twice_remainder == denominator && rounded_units & 1 == 1)
    {
        rounded_units += 1;
    }

    if rounded_units == 0 {
        return Some(Err(ValidationError::InvalidInput));
    }
    let minimum_normal_units = 1_u128 << 52;
    if rounded_units > minimum_normal_units {
        return None;
    }
    if rounded_units == minimum_normal_units {
        return Some(Ok(f64::MIN_POSITIVE));
    }
    Some(Ok(f64::from_bits(rounded_units as u64)))
}

fn translated_residuals_from_anchor(
    diffs: &[f64],
    roundoffs: &[f64],
    anchor_index: usize,
) -> Option<Vec<f64>> {
    let anchor_high = diffs[anchor_index];
    let anchor_low = roundoffs[anchor_index];
    let mut translated = Vec::with_capacity(diffs.len());

    for (&high, &low) in diffs.iter().zip(roundoffs) {
        let high_delta = high - anchor_high;
        if !high_delta.is_finite()
            || subtraction_roundoff(high, anchor_high, high_delta) != 0.0
        {
            return None;
        }

        let low_delta = low - anchor_low;
        if !low_delta.is_finite()
            || subtraction_roundoff(low, anchor_low, low_delta) != 0.0
        {
            return None;
        }

        let delta = high_delta + low_delta;
        if !delta.is_finite() || subtraction_roundoff(high_delta, -low_delta, delta) != 0.0 {
            return None;
        }
        translated.push(delta);
    }

    Some(translated)
}

fn canonical_exact_translated_residuals(diffs: &[f64], roundoffs: &[f64]) -> Option<Vec<f64>> {
    let mut best: Option<(usize, f64, Vec<f64>)> = None;

    for anchor_index in 0..diffs.len() {
        let Some(translated) = translated_residuals_from_anchor(diffs, roundoffs, anchor_index) else {
            continue;
        };
        let max_magnitude = translated
            .iter()
            .map(|value| value.abs())
            .fold(0.0, f64::max);

        let should_replace = match &best {
            None => true,
            Some((best_index, best_max_magnitude, _)) => max_magnitude
                .total_cmp(best_max_magnitude)
                .then_with(|| diffs[anchor_index].total_cmp(&diffs[*best_index]))
                .then_with(|| roundoffs[anchor_index].total_cmp(&roundoffs[*best_index]))
                .is_lt(),
        };
        if should_replace {
            best = Some((anchor_index, max_magnitude, translated));
        }
    }

    best.map(|(_, _, translated)| translated)
}

fn exact_translated_residual_standard_error(
    diffs: &[f64],
    roundoffs: &[f64],
) -> Result<Option<f64>, ValidationError> {
    // A translated second moment is order-invariant, so admission must not depend
    // on whichever observation happened to arrive first. Search every candidate
    // anchor that preserves exact high, low and recombined deltas, prefer the one
    // with the smallest maximum translated magnitude, then break ties in canonical
    // represented `(high, low)` order. This keeps the choice permutation-invariant
    // while minimizing the dynamic range exposed to the later square/sqrt path.
    let Some(translated) = canonical_exact_translated_residuals(diffs, roundoffs) else {
        return Ok(None);
    };

    let max_magnitude = translated
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    if max_magnitude == 0.0 {
        return Ok(Some(0.0));
    }

    let mut zero_count = 0_usize;
    let mut repeated_gap = None;
    let mut gap_count = 0_usize;
    let mut exactly_two_levels = true;
    for &value in &translated {
        if value == 0.0 {
            zero_count += 1;
        } else if let Some(gap) = repeated_gap {
            if value != gap {
                exactly_two_levels = false;
                break;
            }
            gap_count += 1;
        } else {
            repeated_gap = Some(value);
            gap_count = 1;
        }
    }
    if exactly_two_levels && let Some(gap) = repeated_gap {
        let rational_scale = exact_two_level_rational_scale(zero_count, gap_count);
        let standard_error = if zero_count == 1 || gap_count == 1 {
            // For an exactly translated two-level sample where either level
            // occurs once, SE(mean) simplifies to |gap| / n.
            gap.abs() / translated.len() as f64
        } else if let Some((numerator, denominator)) = rational_scale {
            // A normalized quotient can double-round when its exact power-of-two
            // restoration lands in the subnormal range. Round exact represented
            // minimum-subnormal units first when that bounded case applies;
            // otherwise retain the existing overflow-safe sum-over-count path.
            if let Some(subnormal_result) =
                exact_subnormal_rational_scale(gap, numerator, denominator)
            {
                subnormal_result?
            } else {
                let scaled_gap = vec![gap.abs(); numerator];
                deterministic_representable_sum_over_count(&scaled_gap, denominator)?
            }
        } else {
            0.0
        };

        if standard_error != 0.0 {
            return Ok(Some(standard_error));
        }
        if (zero_count == 1 || gap_count == 1 || rational_scale.is_some()) && gap != 0.0 {
            return Err(ValidationError::InvalidInput);
        }
    }

    // Keep the translated binary64 geometry on an exact dyadic scale. Using the
    // largest translated value itself can turn an exactly represented gap d into
    // rounded(1/3) * d after the square-root stage and move the final SE by one
    // ULP. A power-of-two scale changes only exponents, so restoring it is exact.
    let scale = exact_power_of_two_scale(max_magnitude);

    let normalized: Vec<_> = translated.iter().map(|value| *value / scale).collect();
    if translated
        .iter()
        .zip(&normalized)
        .any(|(value, normalized_value)| *value != 0.0 && *normalized_value == 0.0)
    {
        return Ok(None);
    }

    let normalized_sum = deterministic_compensated_sum(normalized.clone());
    let normalized_square_sum = deterministic_compensated_sum(
        normalized
            .iter()
            .map(|value| value * value)
            .collect(),
    );
    let sample_count = translated.len() as f64;
    let scaled_square_sum = sample_count * normalized_square_sum;
    let dispersion_numerator =
        (-normalized_sum).mul_add(normalized_sum, scaled_square_sum);
    if !dispersion_numerator.is_finite() || dispersion_numerator <= 0.0 {
        return Ok(None);
    }

    let denominator = sample_count * sample_count * (sample_count - 1.0);
    let normalized_standard_error = (dispersion_numerator / denominator).sqrt();
    let standard_error = scale * normalized_standard_error;
    if !standard_error.is_finite()
        || (standard_error == 0.0 && normalized_standard_error != 0.0)
    {
        Err(ValidationError::InvalidInput)
    } else {
        Ok(Some(standard_error))
    }
}

/// Mean signed bias `mean(recovered − truth)`.
///
/// Exactly representable finite signed residuals use the canonical
/// cancellation-safe mean directly. If a finite pairwise subtraction discards
/// represented low-order mass, or if the subtraction overflows even though the
/// final mean bias can remain representable, TEPP expands the same algebraic
/// numerator into recovered values plus negated truth values. The canonical
/// cancellation path then carries represented input mass through the original
/// paired-observation denominator instead of making a rounded pairwise residual
/// authoritative. This preserves the existing fast path when every subtraction
/// is exact while avoiding subtraction-rounding and overflow artifacts.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty, unequal-length, or
/// non-finite inputs, or for an unrepresentable nonzero final mean bias.
pub fn mean_bias(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    require_paired_finite(truth, recovered)?;

    let mut residuals = Vec::with_capacity(truth.len());
    let mut requires_expanded_numerator = false;
    for (truth_value, recovered_value) in truth.iter().zip(recovered) {
        let residual = recovered_value - truth_value;
        if !residual.is_finite() {
            requires_expanded_numerator = true;
            break;
        }
        requires_expanded_numerator |=
            subtraction_has_roundoff(*recovered_value, *truth_value, residual);
        residuals.push(residual);
    }
    if !requires_expanded_numerator {
        return deterministic_representable_mean(&residuals);
    }

    let mut expanded_terms = Vec::with_capacity(truth.len().saturating_mul(2));
    expanded_terms.extend_from_slice(recovered);
    expanded_terms.extend(truth.iter().map(|value| -*value));
    deterministic_representable_sum_over_count(&expanded_terms, truth.len())
}

/// Standard error of the mean signed bias under independent observations.
///
/// The signed-difference mean uses the same cancellation-safe deterministic
/// reference as [`mean_bias`]. Squared deviations are accumulated only after
/// scaling by their largest magnitude, and the standard error is formed from
/// `sqrt(sum(d²) / (n * (n - 1)))` so an avoidable intermediate variance or
/// separately rounded `sqrt(n)` cannot change the final represented result.
/// For any two finite represented residuals, the exact two-observation identity
/// `SE = |r₁-r₂| / 2` is evaluated before a rounded residual mean can distort
/// dispersion. If either pairwise residual subtraction discarded represented
/// low-order input mass, the same identity is evaluated from the expanded
/// recovered/truth inputs instead of the rounded residuals. For larger samples
/// whose rounded residuals all collapse to the same binary64 value, the common
/// high part cannot contribute to dispersion. TEPP therefore first evaluates a
/// translation-invariant second moment directly from the error-free subtraction
/// low terms when their anchor-relative differences are exactly representable;
/// only cases that cannot prove that exact translation retain the predecessor
/// rounded-low-term mean path. For other larger samples, including exact pairwise
/// residuals, TEPP uses the `high + low` decomposition and the same translated
/// second moment whenever every anchor-relative high delta, low delta, and
/// combined residual delta is exactly representable. Exact candidate translation
/// anchors are compared by their maximum translated magnitude first and canonical
/// represented `(high, low)` order second, preserving permutation invariance while
/// avoiding an unnecessarily wide square/sqrt working range. For an exactly
/// translated two-level sample where either level occurs once, the algebraic
/// identity `SE(mean) = |level_gap| / n` is evaluated directly. Non-singleton
/// two-level count geometries whose reduced count factor is an exact rational
/// square are likewise applied as a represented rational scale before moment
/// reconstruction; if that exact rational result is subnormal, TEPP rounds once
/// in represented minimum-subnormal units instead of normalizing and restoring
/// through a second binary64 rounding boundary. The general translated path
/// avoids making a rounded residual mean authoritative before dispersion is
/// evaluated. Its normalization uses an exact power-of-two scale so the
/// translated geometry is not re-rounded through an arbitrary magnitude before
/// the final SE is restored. Cases that cannot prove those translated deltas
/// representable retain the predecessor rounded-residual path. Individual signed
/// residuals must still be representable because their dispersion is itself part
/// of the requested scientific result.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for invalid pairs, `n < 2`, an
/// unrepresentable signed residual or mean, or an unrepresentable final
/// standard error.
pub fn bias_standard_error(truth: &[f64], recovered: &[f64]) -> Result<f64, ValidationError> {
    if truth.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }
    let diffs = signed_residuals(truth, recovered)?;
    let subtraction_roundoffs: Vec<_> = truth
        .iter()
        .zip(recovered)
        .zip(&diffs)
        .map(|((truth_value, recovered_value), residual)| {
            subtraction_roundoff(*recovered_value, *truth_value, *residual)
        })
        .collect();
    let has_subtraction_roundoff = subtraction_roundoffs.iter().any(|roundoff| *roundoff != 0.0);

    if diffs.len() == 2 {
        let half_difference = if has_subtraction_roundoff {
            let expanded_difference = [recovered[0], -truth[0], -recovered[1], truth[1]];
            deterministic_representable_sum_over_count(&expanded_difference, 2)?
        } else {
            let represented_difference = [diffs[0], -diffs[1]];
            deterministic_representable_sum_over_count(&represented_difference, 2)?
        };
        return Ok(half_difference.abs());
    }

    if has_subtraction_roundoff && diffs.iter().all(|residual| *residual == diffs[0]) {
        let zero_roundoffs = vec![0.0; subtraction_roundoffs.len()];
        if let Some(standard_error) =
            exact_translated_residual_standard_error(&subtraction_roundoffs, &zero_roundoffs)?
        {
            return Ok(standard_error);
        }
        let roundoff_mean = deterministic_representable_mean(&subtraction_roundoffs)?;
        return scaled_standard_error(&subtraction_roundoffs, roundoff_mean);
    }

    if diffs.len() > 2 {
        if let Some(standard_error) =
            exact_translated_residual_standard_error(&diffs, &subtraction_roundoffs)?
        {
            return Ok(standard_error);
        }
    }

    let mean = deterministic_representable_mean(&diffs)?;
    scaled_standard_error(&diffs, mean)
}

#[cfg(test)]
mod tests {
    use super::{
        bias_standard_error, exact_subnormal_rational_scale, mean_bias,
    };
    use crate::ValidationError;

    #[test]
    fn bias_oracle_and_degenerate_cases() {
        let truth = [1.0, 2.0, 3.0];
        let recovered = [2.0, 3.0, 4.0];
        assert!((mean_bias(&truth, &recovered).expect("bias") - 1.0).abs() < 1e-12);
        assert_eq!(mean_bias(&[1.0], &[1.0]), Ok(0.0));
        let se = bias_standard_error(&truth, &recovered).expect("se");
        assert_eq!(se, 0.0);
        assert_eq!(mean_bias(&[], &[]), Err(ValidationError::InvalidInput));
        assert_eq!(
            mean_bias(&[1.0], &[1.0, 2.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            mean_bias(&[f64::INFINITY], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            bias_standard_error(&[1.0], &[2.0]),
            Err(ValidationError::InvalidInput)
        );
        let se_var = bias_standard_error(&[0.0, 0.0], &[1.0, -1.0]).expect("se");
        assert_eq!(se_var, 1.0);
        assert_eq!(
            mean_bias(&[f64::MAX], &[-f64::MAX]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            bias_standard_error(&[f64::MAX, 0.0], &[-f64::MAX, 0.0]),
            Err(ValidationError::InvalidInput)
        );
    }

    #[test]
    fn representable_extreme_constant_bias_does_not_fail_on_raw_sum_overflow() {
        assert_eq!(
            mean_bias(&[0.0, 0.0], &[f64::MAX, f64::MAX]),
            Ok(f64::MAX)
        );
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[f64::MAX, f64::MAX]),
            Ok(0.0)
        );
    }

    #[test]
    fn exact_zero_and_unrepresentable_nonzero_bias_are_distinct() {
        let ulp = f64::from_bits(1);
        assert_eq!(mean_bias(&[0.0, 0.0], &[ulp, -ulp]), Ok(0.0));
        assert_eq!(
            mean_bias(&[0.0, 0.0], &[ulp, 0.0]),
            Err(ValidationError::InvalidInput)
        );
    }

    #[test]
    fn representable_standard_error_avoids_square_and_variance_overflow() {
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[f64::MAX, -f64::MAX]),
            Ok(f64::MAX)
        );
        let huge = 1e200;
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[huge, -huge]),
            Ok(huge)
        );
        let square_sum_overflows = 1e154;
        assert_eq!(
            bias_standard_error(&[0.0, 0.0], &[square_sum_overflows, -square_sum_overflows]),
            Ok(square_sum_overflows)
        );
        let three_point = bias_standard_error(
            &[0.0, 0.0, 0.0],
            &[square_sum_overflows, -square_sum_overflows, 0.0],
        )
        .expect("representable three-point standard error");
        let expected = square_sum_overflows / 3.0_f64.sqrt();
        assert!(((three_point - expected) / expected).abs() <= f64::EPSILON);
    }

    #[test]
    fn overflowing_direct_deviation_uses_scaled_reference() {
        let standard_error = bias_standard_error(
            &[0.0, 0.0, 0.0],
            &[f64::MAX, -f64::MAX, -f64::MAX],
        )
        .expect("scaled deviation path");
        assert!(standard_error.is_finite());
        assert!(standard_error > 0.0);
    }

    #[test]
    fn exact_subnormal_rational_scale_covers_projection_boundaries() {
        assert_eq!(exact_subnormal_rational_scale(0.0, 1, 1), None);
        assert_eq!(exact_subnormal_rational_scale(f64::INFINITY, 1, 1), None);
        assert_eq!(exact_subnormal_rational_scale(1.0, 0, 1), None);
        assert_eq!(exact_subnormal_rational_scale(1.0, 1, 0), None);

        assert_eq!(
            exact_subnormal_rational_scale(f64::from_bits(22), 3, 44),
            Some(Ok(f64::from_bits(2)))
        );
        assert_eq!(
            exact_subnormal_rational_scale(f64::from_bits(66), 3, 44),
            Some(Ok(f64::from_bits(4)))
        );
        assert_eq!(
            exact_subnormal_rational_scale(f64::from_bits(1), 3, 44),
            Some(Err(ValidationError::InvalidInput))
        );
        assert_eq!(
            exact_subnormal_rational_scale(f64::from_bits(0x004d_5555_5555_5555), 3, 44),
            Some(Ok(f64::MIN_POSITIVE))
        );
        assert_eq!(exact_subnormal_rational_scale(f64::MIN_POSITIVE * 2.0, 1, 1), None);
        assert_eq!(exact_subnormal_rational_scale(f64::MAX, 1, 1), None);
    }
}
