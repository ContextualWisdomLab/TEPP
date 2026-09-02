//! Numerically stable public composition for irregular residual log-rates.
//!
//! This module keeps the public longitudinal estimand unchanged while making
//! the final count weighting robust when an intermediate `mean * count` would
//! overflow even though the mathematical weighted mean is representable.

use crate::irregular_residual::{
    EventTimedObservation, LaggedWithinResidual, center_within_unit_event_lags,
    driver_same_sign_log_rate, same_sign_nonzero,
};
use crate::LongitudinalError;

/// Pairwise-mean exact log-rate after CWC on irregular event intervals.
///
/// Temporal admission and CWC remain owned by the longitudinal composition in
/// [`center_within_unit_event_lags`]. Same-sign finite pair rates are then
/// averaged without requiring an overflowing intermediate sum or count product.
///
/// # Errors
///
/// Propagates temporal-admission errors and returns
/// [`LongitudinalError::InvalidTemporalTransformInput`] when no admissible
/// same-sign pair exists or a finite rate cannot be represented.
pub fn recover_within_unit_irregular_residual_log_rate(
    rows: &[EventTimedObservation],
) -> Result<f64, LongitudinalError> {
    let lagged = center_within_unit_event_lags(rows)?;
    let mut rates = Vec::with_capacity(lagged.len());
    for pair in &lagged {
        if !same_sign_nonzero(pair.earlier_residual(), pair.later_residual()) {
            continue;
        }
        rates.push(driver_same_sign_log_rate(
            pair.earlier_residual(),
            pair.later_residual(),
            pair.event_interval(),
        )?);
    }
    stable_mean(&rates)
}

/// Mean exact scalar log-rate on already-centered residuals.
///
/// Each pair is `ln(|later| / |earlier|) / Δt` and therefore requires finite,
/// nonzero residuals of equal sign. Direct division is used when its positive
/// ratio is representable; otherwise the equivalent log-domain difference is
/// used so ratio overflow or underflow cannot reject a finite log-rate.
/// Opposing extreme finite rates are cancelled before count weighting so a
/// representable final mean is not rejected merely because an intermediate
/// multiplication overflows.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidObservationPayload`] for an empty input
/// or non-finite residuals, and
/// [`LongitudinalError::InvalidTemporalTransformInput`] for opposite-sign or
/// zero residuals, a non-finite rate, or a non-representable final mean.
pub fn recover_centered_irregular_residual_log_rate(
    pairs: &[LaggedWithinResidual],
) -> Result<f64, LongitudinalError> {
    if pairs.is_empty() {
        return Err(LongitudinalError::InvalidObservationPayload);
    }

    let mut rates = Vec::with_capacity(pairs.len());
    for pair in pairs {
        if !pair.earlier_residual().is_finite() || !pair.later_residual().is_finite() {
            return Err(LongitudinalError::InvalidObservationPayload);
        }
        if !same_sign_nonzero(pair.earlier_residual(), pair.later_residual()) {
            return Err(LongitudinalError::InvalidTemporalTransformInput);
        }
        rates.push(driver_same_sign_log_rate(
            pair.earlier_residual(),
            pair.later_residual(),
            pair.event_interval(),
        )?);
    }

    stable_mean(&rates)
}

fn stable_mean(values: &[f64]) -> Result<f64, LongitudinalError> {
    if values.is_empty() {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }

    let mut positives = Vec::new();
    let mut negatives = Vec::new();
    for &value in values {
        if !value.is_finite() {
            return Err(LongitudinalError::InvalidTemporalTransformInput);
        }
        if value > 0.0 {
            positives.push(value);
        } else if value < 0.0 {
            negatives.push(value);
        }
    }

    if positives.is_empty() && negatives.is_empty() {
        return Ok(0.0);
    }
    if positives.is_empty() || negatives.is_empty() {
        return same_sign_mean(values);
    }

    positives.sort_by(|left, right| right.total_cmp(left));
    negatives.sort_by(|left, right| left.total_cmp(right));

    let mut positive_index = 0_usize;
    let mut negative_index = 0_usize;
    let mut positive = positives[0];
    let mut negative = negatives[0];
    let mut residuals = Vec::with_capacity(values.len());

    loop {
        let residual = positive + negative;
        if residual > 0.0 {
            positive = residual;
            negative_index += 1;
            if negative_index == negatives.len() {
                residuals.push(positive);
                residuals.extend_from_slice(&positives[positive_index + 1..]);
                break;
            }
            negative = negatives[negative_index];
        } else if residual < 0.0 {
            negative = residual;
            positive_index += 1;
            if positive_index == positives.len() {
                residuals.push(negative);
                residuals.extend_from_slice(&negatives[negative_index + 1..]);
                break;
            }
            positive = positives[positive_index];
        } else {
            positive_index += 1;
            negative_index += 1;
            if positive_index == positives.len() || negative_index == negatives.len() {
                residuals.extend_from_slice(&positives[positive_index..]);
                residuals.extend_from_slice(&negatives[negative_index..]);
                break;
            }
            positive = positives[positive_index];
            negative = negatives[negative_index];
        }
    }

    if residuals.is_empty() {
        return Ok(0.0);
    }

    let residual_mean = same_sign_mean(&residuals)?;
    scale_mean_by_count_ratio(residual_mean, residuals.len(), values.len())
}

fn scale_mean_by_count_ratio(
    mean: f64,
    retained_count: usize,
    total_count: usize,
) -> Result<f64, LongitudinalError> {
    let retained_count = retained_count as f64;
    let total_count = total_count as f64;

    let retained_mass = mean * retained_count;
    let result = if retained_mass.is_finite() {
        retained_mass / total_count
    } else {
        // Overflow here implies a large finite mean. Divide first only on this
        // branch; unlike unconditional `mean * (k / n)`, it cannot erase the
        // subnormal retained-mass cases that motivated the preceding repair.
        (mean / total_count) * retained_count
    };

    if result.is_finite() {
        Ok(result)
    } else {
        Err(LongitudinalError::InvalidTemporalTransformInput)
    }
}

fn same_sign_mean(values: &[f64]) -> Result<f64, LongitudinalError> {
    let mut mean = 0.0_f64;
    for (index, &value) in values.iter().enumerate() {
        let count = (index + 1) as f64;
        mean += (value - mean) / count;
    }
    if mean.is_finite() {
        Ok(mean)
    } else {
        Err(LongitudinalError::InvalidTemporalTransformInput)
    }
}

#[cfg(test)]
mod tests {
    use super::stable_mean;

    #[test]
    fn preserves_subnormal_retained_mass_without_overflowing_large_case() {
        let minimum_subnormal = f64::from_bits(1);
        let subnormal = stable_mean(&[
            f64::MAX,
            f64::from_bits(2),
            f64::from_bits(2),
            -f64::MAX,
        ])
        .expect("subnormal retained mass");
        assert_eq!(subnormal.to_bits(), minimum_subnormal.to_bits());

        let large = 1.45e308_f64;
        assert!(!(large * 2.0).is_finite());
        let finite = stable_mean(&[large, large, -1.0, -1.0])
            .expect("representable final mean");
        assert!(finite.is_finite());
        assert!((finite - large / 2.0).abs() <= (large / 2.0) * 4.0 * f64::EPSILON);
    }
}
