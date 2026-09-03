//! Occasion-mean event-time composition for longitudinal modeling.
//!
//! Hamaker, Kuiper, and Grasman (2015, Eq. 1a) decompose an observed score as
//! `x_it = mu_t + p_it`, where `mu_t` is the occasion-specific group mean.
//! These deviations are distinct from person-mean CWC residuals and still
//! contain stable between-person differences, so they are not within-person
//! effects and are not RI-CLPM residuals.

use std::collections::{BTreeMap, BTreeSet};

use crate::irregular_residual::{
    EventTimedObservation, LaggedWithinResidual, scaled_compensated_mean,
};
use crate::stable_irregular_rate::recover_centered_irregular_residual_log_rate;
use crate::{EventTimeInterval, LongitudinalError};

/// Form consecutive event-time lags after subtracting each occasion's group mean.
///
/// Numeric event-time identity is used for occasion membership. In particular,
/// `-0.0` and `+0.0` are one occasion rather than two binary encodings. Every
/// admitted occasion must contain at least two distinct units, and at least two
/// units must contribute a consecutive lag. Occasion means use the same
/// Longitudinal-local overflow-safe, cancellation-safe, halfway-rounding
/// numerical authority as CWC means and are bit-stable under row permutation.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidObservationPayload`] for empty, sparse,
/// duplicated unit-occasion, or non-finite observations, and
/// [`LongitudinalError::NonPositiveEventInterval`] when a unit's consecutive
/// event times do not form a finite strictly positive interval.
pub fn center_occasion_mean_event_lags(
    rows: &[EventTimedObservation],
) -> Result<Vec<LaggedWithinResidual>, LongitudinalError> {
    if rows.len() < 2 {
        return Err(LongitudinalError::InvalidObservationPayload);
    }

    let mut by_time: BTreeMap<u64, Vec<EventTimedObservation>> = BTreeMap::new();
    let mut by_unit: BTreeMap<u32, Vec<EventTimedObservation>> = BTreeMap::new();
    for &row in rows {
        if !row.event_time().is_finite() || !row.score().is_finite() {
            return Err(LongitudinalError::InvalidObservationPayload);
        }
        by_time
            .entry(canonical_event_time_key(row.event_time()))
            .or_default()
            .push(row);
        by_unit.entry(row.unit_index()).or_default().push(row);
    }

    let lag_contributing_units = by_unit
        .values()
        .filter(|occasions| occasions.len() >= 2)
        .count();
    if lag_contributing_units < 2 {
        return Err(LongitudinalError::InvalidObservationPayload);
    }

    let mut occasion_means = BTreeMap::new();
    for (&time_key, occasion_rows) in &by_time {
        let mut seen_units = BTreeSet::new();
        let mut scores = Vec::with_capacity(occasion_rows.len());
        for row in occasion_rows {
            if !seen_units.insert(row.unit_index()) {
                return Err(LongitudinalError::InvalidObservationPayload);
            }
            scores.push(row.score());
        }
        if seen_units.len() < 2 {
            return Err(LongitudinalError::InvalidObservationPayload);
        }
        occasion_means.insert(time_key, occasion_mean(&scores)?);
    }

    let mut pairs = Vec::new();
    for occasions in by_unit.values_mut() {
        if occasions.len() < 2 {
            continue;
        }
        occasions.sort_by(|left, right| left.event_time().total_cmp(&right.event_time()));
        for window in occasions.windows(2) {
            let earlier = window[0];
            let later = window[1];
            let earlier_mean = occasion_means[&canonical_event_time_key(earlier.event_time())];
            let later_mean = occasion_means[&canonical_event_time_key(later.event_time())];
            let earlier_residual = earlier.score() - earlier_mean;
            let later_residual = later.score() - later_mean;
            if !earlier_residual.is_finite() || !later_residual.is_finite() {
                return Err(LongitudinalError::InvalidObservationPayload);
            }
            let event_interval = EventTimeInterval::new(later.event_time() - earlier.event_time())?;
            pairs.push(LaggedWithinResidual::new(
                earlier_residual,
                later_residual,
                event_interval,
            ));
        }
    }

    if pairs.is_empty() {
        return Err(LongitudinalError::InvalidObservationPayload);
    }
    Ok(pairs)
}

/// Recover the mean exact scalar log-rate of occasion-mean residuals.
///
/// This composes [`center_occasion_mean_event_lags`] with the existing
/// longitudinal exact-log-rate boundary. It is an event-time association of
/// Hamaker Eq. 1a deviations, not a within-person autoregressive effect.
///
/// # Errors
///
/// Propagates centering and exact-log-rate admission failures.
pub fn recover_occasion_mean_centered_irregular_residual_log_rate(
    rows: &[EventTimedObservation],
) -> Result<f64, LongitudinalError> {
    let pairs = center_occasion_mean_event_lags(rows)?;
    recover_centered_irregular_residual_log_rate(&pairs)
}

/// Refuse treating an occasion-mean residual log-rate as within-person change.
///
/// Hamaker Eq. 1a deviations retain between-person differences unless a
/// person-specific stable component is removed by a model that identifies it.
///
/// # Errors
///
/// Always returns [`LongitudinalError::BetweenIsNotWithinChange`].
pub fn refuse_occasion_mean_centered_log_rate_as_within_person_lag(
    log_rate: f64,
) -> Result<f64, LongitudinalError> {
    let _ = log_rate;
    Err(LongitudinalError::BetweenIsNotWithinChange)
}

fn canonical_event_time_key(event_time: f64) -> u64 {
    if event_time == 0.0 {
        0.0_f64.to_bits()
    } else {
        event_time.to_bits()
    }
}

fn occasion_mean(values: &[f64]) -> Result<f64, LongitudinalError> {
    scaled_compensated_mean(values).map_err(|_| LongitudinalError::InvalidObservationPayload)
}
