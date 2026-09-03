//! CWC-then-irregular residual log-rate on substantive event time.
//!
//! Unique evidence folded from Draft #327. Temporal composition belongs here,
//! not in a generic psychometric kernel. This is not DSEM, not Newton LS, and
//! not raw-process autoregressive drift.

use std::collections::BTreeMap;

use crate::{EventTimeInterval, LongitudinalError};

/// One unit's score at one event-time occasion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventTimedObservation {
    unit_index: u32,
    event_time: f64,
    score: f64,
}

impl EventTimedObservation {
    /// Construct an event-timed observation.
    ///
    /// The constructor stores the fields as given. Admission of finite scores,
    /// strictly positive consecutive intervals, and at least two lag-contributing
    /// units happens in [`center_within_unit_event_lags`].
    #[must_use]
    pub const fn new(unit_index: u32, event_time: f64, score: f64) -> Self {
        Self {
            unit_index,
            event_time,
            score,
        }
    }

    /// Return the unit index.
    #[must_use]
    pub const fn unit_index(self) -> u32 {
        self.unit_index
    }

    /// Return the event time.
    #[must_use]
    pub const fn event_time(self) -> f64 {
        self.event_time
    }

    /// Return the observed score.
    #[must_use]
    pub const fn score(self) -> f64 {
        self.score
    }
}

/// One already-formed lagged within residual pair on event time.
///
/// The interval is admitted event time. Residuals are stored as given; the
/// recover functions decide whether a pair is an admissible log-rate input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaggedWithinResidual {
    earlier_residual: f64,
    later_residual: f64,
    event_interval: EventTimeInterval,
}

impl LaggedWithinResidual {
    /// Construct a lagged within-residual pair on an admitted event interval.
    #[must_use]
    pub const fn new(
        earlier_residual: f64,
        later_residual: f64,
        event_interval: EventTimeInterval,
    ) -> Self {
        Self {
            earlier_residual,
            later_residual,
            event_interval,
        }
    }

    /// Return the earlier within residual.
    #[must_use]
    pub const fn earlier_residual(self) -> f64 {
        self.earlier_residual
    }

    /// Return the later within residual.
    #[must_use]
    pub const fn later_residual(self) -> f64 {
        self.later_residual
    }

    /// Return the admitted event-time interval.
    #[must_use]
    pub const fn event_interval(self) -> EventTimeInterval {
        self.event_interval
    }
}

/// Cluster-mean-center consecutive event-time lags inside each unit.
///
/// Stable between-unit means are removed first (CWC). Consecutive within-unit
/// residuals then become [`LaggedWithinResidual`] pairs on possibly irregular
/// event intervals. Singleton units do not contribute lags and therefore do not
/// count toward the two-unit longitudinal evidence floor. Curran and Bauer
/// (2011, pp. 583–619; PMC3059070 XML opened 2026-09-02; Eq. 36) show that
/// person-mean subtraction of a time-varying covariate related to time is
/// biased for the within-person effect. The returned pairs are therefore not a
/// license to recover raw-process drift `a`.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidObservationPayload`] for empty,
/// singleton-only, fewer-than-two lag-contributing units, or non-finite rows,
/// including non-representable stable unit means and overflowing CWC residuals
/// after a finite mean, and [`LongitudinalError::NonPositiveEventInterval`]
/// when any consecutive event interval is not strictly positive.
pub fn center_within_unit_event_lags(
    rows: &[EventTimedObservation],
) -> Result<Vec<LaggedWithinResidual>, LongitudinalError> {
    if rows.len() < 2 {
        return Err(LongitudinalError::InvalidObservationPayload);
    }
    let mut groups: BTreeMap<u32, Vec<EventTimedObservation>> = BTreeMap::new();
    for &row in rows {
        if !row.event_time().is_finite() || !row.score().is_finite() {
            return Err(LongitudinalError::InvalidObservationPayload);
        }
        groups.entry(row.unit_index()).or_default().push(row);
    }
    if groups.len() < 2 {
        return Err(LongitudinalError::InvalidObservationPayload);
    }
    let lag_contributing_units = groups
        .values()
        .filter(|occasions| occasions.len() >= 2)
        .count();
    if lag_contributing_units < 2 {
        return Err(LongitudinalError::InvalidObservationPayload);
    }
    let mut pairs = Vec::new();
    for occasions in groups.values_mut() {
        if occasions.len() < 2 {
            continue;
        }
        occasions.sort_by(|left, right| left.event_time().total_cmp(&right.event_time()));
        let scores: Vec<f64> = occasions.iter().map(|row| row.score()).collect();
        let mean = scaled_compensated_mean(&scores)
            .map_err(|_| LongitudinalError::InvalidObservationPayload)?;
        for window in occasions.windows(2) {
            let earlier_residual = window[0].score() - mean;
            let later_residual = window[1].score() - mean;
            if !earlier_residual.is_finite() || !later_residual.is_finite() {
                return Err(LongitudinalError::InvalidObservationPayload);
            }
            let event_delta = window[1].event_time() - window[0].event_time();
            let event_interval = EventTimeInterval::new(event_delta)?;
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

/// Pairwise-mean exact log-rate after CWC on irregular event intervals.
///
/// This is [`center_within_unit_event_lags`] then the pairwise mean of the
/// Driver, Oud, and Voelkle (2017, Eq. 3) scalar inverse
/// `a = ln(|later| / |earlier|) / Δt` on nonzero same-sign residuals. When
/// `|later| / |earlier|` is finite and positive the finite ratio logarithm is
/// used so near-equal large residuals do not collapse to zero. Overflowed or
/// underflowed ratios fall back to `ln|later| − ln|earlier|`. Opposite-sign
/// and zero residuals have no real logarithm and are skipped. The pairwise
/// mean cancels opposite-signed finite rates from largest magnitude downward
/// before averaging the surviving same-sign residuals. This avoids both
/// overflowing a raw same-sign sum and destroying representable subnormal
/// terms by pre-scaling them. An empty admissible set fails closed. This is
/// not Newton LS and does not recover raw-process drift from CWC of a raw AR
/// path (Curran & Bauer, 2011, pp. 583–619; Eq. 36).
///
/// # Errors
///
/// Propagates centering errors from [`center_within_unit_event_lags`]. A
/// non-finite log-rate or an empty admissible list is
/// [`LongitudinalError::InvalidTemporalTransformInput`].
pub fn recover_within_unit_irregular_residual_log_rate(
    rows: &[EventTimedObservation],
) -> Result<f64, LongitudinalError> {
    let lagged = center_within_unit_event_lags(rows)?;
    pairwise_same_sign_log_rate(&lagged)
}

/// Mean exact scalar log-rate on already-centered residuals.
///
/// Each pair is `a = ln(|later| / |earlier|) / Δt` (Driver et al., 2017,
/// Eq. 3 inverse). The function does **not** center again. Residuals must be
/// finite, nonzero, and of equal sign. A finite positive direct ratio is used
/// when representable; ratio overflow or underflow falls back to the equivalent
/// `ln|later| - ln|earlier|` so a representable final log-rate is not rejected
/// because of a non-representable intermediate. This is the known-truth path
/// that recovers `ln(0.5)` from already-centered pairs `(1, 0.5)` over unit
/// event time.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidObservationPayload`] for an empty
/// series or non-finite residuals, and
/// [`LongitudinalError::InvalidTemporalTransformInput`] for zero or
/// opposite-sign residuals, a non-finite log-rate, or a non-representable
/// final mean.
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
    scaled_compensated_mean(&rates)
}

/// Refuse treating a CWC residual log-rate as raw-process AR drift.
///
/// Always fails closed. Curran and Bauer (2011, pp. 583–619; PMC3059070 XML
/// opened 2026-09-02) show that person-mean centering of a time-varying
/// covariate related to time is biased for the within-person effect. Licensed
/// detrend is the person-specific OLS residual of the covariate on time
/// (Eq. 36). Use [`recover_centered_irregular_residual_log_rate`] on
/// already-centered residuals for the raw-process estimand.
///
/// # Errors
///
/// Always returns [`LongitudinalError::CwcResidualLogRateIsNotRawProcessDrift`].
pub fn refuse_cwc_residual_log_rate_as_raw_process_drift(
    cwc_log_rate: f64,
    raw_process_drift: f64,
) -> Result<f64, LongitudinalError> {
    let _ = (cwc_log_rate, raw_process_drift);
    Err(LongitudinalError::CwcResidualLogRateIsNotRawProcessDrift)
}

fn pairwise_same_sign_log_rate(lagged: &[LaggedWithinResidual]) -> Result<f64, LongitudinalError> {
    let mut rates = Vec::with_capacity(lagged.len());
    for pair in lagged {
        if !same_sign_nonzero(pair.earlier_residual(), pair.later_residual()) {
            continue;
        }
        rates.push(driver_same_sign_log_rate(
            pair.earlier_residual(),
            pair.later_residual(),
            pair.event_interval(),
        )?);
    }
    scaled_compensated_mean(&rates)
}

/// Overflow-safe mean for finite values across binary64 scales.
///
/// Same-sign inputs are normalized by an exact power-of-two scale derived from
/// their maximum magnitude before a deterministic compensated sum. Power-of-two
/// scaling avoids the extra division rounding that can turn an exact halfway
/// subnormal mean into the wrong even-neighbour result. Mixed-sign inputs are
/// partitioned by sign and sorted from largest magnitude downward. Opposite
/// signs are cancelled before any scale reduction, so a subnormal addend is
/// never divided into zero merely to protect an unrelated extreme term. Each
/// cancellation is an opposite-sign addition and therefore cannot overflow.
/// The remaining terms have one sign and are normalized and summed before the
/// original sample-count denominator is applied. This avoids rounding a
/// retained-only mean and then weighting that rounded intermediate, which can
/// move a representable mixed-sign subnormal result by one ULP.
pub(crate) fn scaled_compensated_mean(values: &[f64]) -> Result<f64, LongitudinalError> {
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
        return same_sign_mean_over_total(values, values.len());
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
    same_sign_mean_over_total(&residuals, values.len())
}

fn same_sign_mean_over_total(
    values: &[f64],
    total_count: usize,
) -> Result<f64, LongitudinalError> {
    let max_magnitude = values
        .iter()
        .map(|value| value.abs())
        .max_by(f64::total_cmp)
        .ok_or(LongitudinalError::InvalidTemporalTransformInput)?;
    if max_magnitude == 0.0 {
        return Ok(0.0);
    }

    let scale = exact_power_of_two_scale(max_magnitude);
    let mut normalized: Vec<f64> = values.iter().map(|value| *value / scale).collect();
    normalized.sort_by(f64::total_cmp);
    let mut sum = 0.0_f64;
    let mut compensation = 0.0_f64;
    for value in normalized {
        let adjusted = value - compensation;
        let next = sum + adjusted;
        compensation = (next - sum) - adjusted;
        sum = next;
    }
    let mean = (sum / total_count as f64) * scale;
    require_finite(mean)
}

fn exact_power_of_two_scale(max_magnitude: f64) -> f64 {
    let bits = max_magnitude.to_bits();
    let exponent = (bits >> 52) & 0x7ff;
    if exponent == 0 {
        let significand = bits & 0x000f_ffff_ffff_ffff;
        let highest_bit = 63 - significand.leading_zeros();
        f64::from_bits(1_u64 << highest_bit)
    } else {
        f64::from_bits(exponent << 52)
    }
}

/// Nonzero residuals of equal sign admit a real Driver Eq. 3 logarithm.
pub(crate) fn same_sign_nonzero(earlier: f64, later: f64) -> bool {
    earlier != 0.0 && later != 0.0 && earlier.is_sign_positive() == later.is_sign_positive()
}

/// Driver et al. (2017, Eq. 3) inverse `a = ln(|later| / |earlier|) / Δt`.
///
/// Caller already established same-sign nonzero residuals and an admitted
/// event interval. Prefer the finite ratio logarithm so near-equal large
/// residuals keep a nonzero rate. Fall back to `ln|later| − ln|earlier|`
/// only when that ratio overflows or underflows. A represented zero rate is
/// accepted only when the residual magnitudes are exactly equal; otherwise it
/// is a non-representable nonzero change and fails closed.
pub(crate) fn driver_same_sign_log_rate(
    earlier: f64,
    later: f64,
    event_interval: EventTimeInterval,
) -> Result<f64, LongitudinalError> {
    let earlier_magnitude = earlier.abs();
    let later_magnitude = later.abs();
    let ratio = later_magnitude / earlier_magnitude;
    let log_ratio = if ratio.is_finite() && ratio > 0.0 {
        ratio.ln()
    } else {
        later_magnitude.ln() - earlier_magnitude.ln()
    };
    let rate = log_ratio / event_interval.as_f64();
    if !rate.is_finite() || (rate == 0.0 && later_magnitude != earlier_magnitude) {
        Err(LongitudinalError::InvalidTemporalTransformInput)
    } else {
        Ok(rate)
    }
}

fn require_finite(value: f64) -> Result<f64, LongitudinalError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(LongitudinalError::InvalidTemporalTransformInput)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EventTimedObservation, LaggedWithinResidual, center_within_unit_event_lags,
        driver_same_sign_log_rate, recover_centered_irregular_residual_log_rate,
        recover_within_unit_irregular_residual_log_rate,
        refuse_cwc_residual_log_rate_as_raw_process_drift, same_sign_nonzero,
        scaled_compensated_mean,
    };
    use crate::{EventTimeInterval, LongitudinalError};

    fn timed(unit: u32, event_time: f64, score: f64) -> EventTimedObservation {
        EventTimedObservation::new(unit, event_time, score)
    }

    fn unit_interval() -> EventTimeInterval {
        EventTimeInterval::new(1.0).expect("unit interval")
    }

    fn lagged(earlier: f64, later: f64, delta: f64) -> LaggedWithinResidual {
        LaggedWithinResidual::new(
            earlier,
            later,
            EventTimeInterval::new(delta).expect("test interval"),
        )
    }

    fn decaying_scores(drift: f64) -> [EventTimedObservation; 8] {
        [
            timed(0, 0.0, 10.0 + 1.0),
            timed(0, 1.0, 10.0 + drift.exp()),
            timed(0, 2.0, 10.0 + (drift * 2.0).exp()),
            timed(0, 3.0, 10.0 + (drift * 3.0).exp()),
            timed(1, 0.0, 4.0 + 1.0),
            timed(1, 1.0, 4.0 + drift.exp()),
            timed(1, 2.0, 4.0 + (drift * 2.0).exp()),
            timed(1, 3.0, 4.0 + (drift * 3.0).exp()),
        ]
    }

    #[test]
    fn already_centered_irregular_pairs_recover_true_log_rate() {
        let drift = -0.35_f64;
        let pairs = [
            lagged(1.4, 1.4 * (drift * 0.4).exp(), 0.4),
            lagged(0.9, 0.9 * (drift * 1.6).exp(), 1.6),
            lagged(-0.7, -0.7 * (drift * 2.2).exp(), 2.2),
        ];
        let recovered = recover_centered_irregular_residual_log_rate(&pairs).expect("centered");
        assert!((recovered - drift).abs() < 1e-12);
        let half = recover_centered_irregular_residual_log_rate(&[lagged(1.0, 0.5, 1.0)])
            .expect("ln(0.5)");
        assert!((half - 0.5_f64.ln()).abs() < 1e-15);
    }

    #[test]
    fn cwc_of_raw_ar_does_not_recover_process_drift() {
        let drift = -0.3_f64;
        let rows = decaying_scores(drift);
        let extracted = center_within_unit_event_lags(&rows).expect("cwc pairs");
        let composed =
            recover_within_unit_irregular_residual_log_rate(&rows).expect("cwc pairwise");
        assert!((composed - drift).abs() > 1e-6);
        assert_eq!(
            refuse_cwc_residual_log_rate_as_raw_process_drift(composed, drift),
            Err(LongitudinalError::CwcResidualLogRateIsNotRawProcessDrift)
        );
        let admissible: Vec<LaggedWithinResidual> = extracted
            .iter()
            .copied()
            .filter(|pair| same_sign_nonzero(pair.earlier_residual(), pair.later_residual()))
            .collect();
        let from_pairs =
            recover_centered_irregular_residual_log_rate(&admissible).expect("admissible");
        assert!((composed - from_pairs).abs() < 1e-15);
        assert_eq!(
            extracted[0].event_interval().as_f64().to_bits(),
            1.0_f64.to_bits()
        );
        assert_eq!(timed(9, 8.0, 0.0).unit_index(), 9);
        assert_eq!(timed(9, 8.0, 0.0).event_time().to_bits(), 8.0_f64.to_bits());
        assert_eq!(timed(9, 8.0, 0.0).score().to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            lagged(1.0, 0.5, 1.0).earlier_residual().to_bits(),
            1.0_f64.to_bits()
        );
        assert_eq!(
            lagged(1.0, 0.5, 1.0).later_residual().to_bits(),
            0.5_f64.to_bits()
        );
    }

    #[test]
    fn two_occasion_cwc_is_empty_admissible() {
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(0, 0.0, 1.0),
                timed(0, 1.0, 0.5),
                timed(1, 0.0, 2.0),
                timed(1, 1.0, 1.0),
            ]),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }

    #[test]
    fn arithmetic_progression_has_zero_residual_and_fails_closed() {
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(0, 0.0, 1.0),
                timed(0, 1.0, 2.0),
                timed(0, 2.0, 3.0),
                timed(1, 0.0, 4.0),
                timed(1, 1.0, 5.0),
                timed(1, 2.0, 6.0),
            ]),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }

    #[test]
    fn same_sign_nonzero_rejects_zero_and_opposite_signs() {
        assert!(!same_sign_nonzero(0.0, 1.0));
        assert!(!same_sign_nonzero(1.0, 0.0));
        assert!(!same_sign_nonzero(-1.0, 2.0));
        assert!(same_sign_nonzero(1e-160, 1e160));
        assert!(same_sign_nonzero(-0.4, -1.2));
    }

    #[test]
    fn driver_same_sign_prefers_finite_ratio_ln_for_near_equal_large_residuals() {
        let earlier = 1e20_f64;
        let later = earlier * (-1e-12_f64).exp();
        let ratio = later.abs() / earlier.abs();
        assert!(ratio.is_finite() && ratio > 0.0);
        let from_ratio = ratio.ln();
        let rate = driver_same_sign_log_rate(earlier, later, unit_interval()).expect("near-equal");
        assert_eq!(rate.to_bits(), from_ratio.to_bits());
        let overflow_rate = driver_same_sign_log_rate(f64::from_bits(1), f64::MAX, unit_interval())
            .expect("overflow arm");
        let overflow_logs = f64::MAX.ln() - f64::from_bits(1).ln();
        assert_eq!(overflow_rate.to_bits(), overflow_logs.to_bits());
        let underflow_rate =
            driver_same_sign_log_rate(1e300_f64, 1e-300_f64, unit_interval()).expect("underflow");
        let underflow_logs = (1e-300_f64).ln() - (1e300_f64).ln();
        assert_eq!(underflow_rate.to_bits(), underflow_logs.to_bits());
    }

    #[test]
    fn scaled_compensated_mean_keeps_extreme_cancellation_and_overflow_safe_mean() {
        let large = 1.45e308_f64;
        assert!(!(large + large).is_finite());
        let same_sign = scaled_compensated_mean(&[large, large]).expect("same-sign mean");
        assert!(same_sign.is_finite());
        assert!((same_sign - large).abs() < 1.0);
        let negative = scaled_compensated_mean(&[-4.0, -2.0]).expect("negative same-sign mean");
        assert_eq!(negative, -3.0);
        let mixed = scaled_compensated_mean(&[large, -large]).expect("mixed mean");
        assert!(mixed.abs() < 1.0);

        let recovered = scaled_compensated_mean(&[1.0e100, 1.0, -1.0e100])
            .expect("small signal survives cancellation");
        assert!((recovered - (1.0 / 3.0)).abs() <= 1.0e-12);
        let full_range = scaled_compensated_mean(&[f64::MAX, 1.0e-16, -f64::MAX])
            .expect("full exponent range cancellation");
        assert_eq!(full_range.to_bits(), (1.0e-16_f64 / 3.0).to_bits());
        let minimum_subnormal = f64::from_bits(1);
        let subnormal = scaled_compensated_mean(&[
            f64::MAX,
            f64::from_bits(2),
            f64::from_bits(2),
            -f64::MAX,
        ])
        .expect("subnormal cancellation residue");
        assert_eq!(subnormal.to_bits(), minimum_subnormal.to_bits());
        let finite_after_mass_overflow = scaled_compensated_mean(&[large, large, -1.0, -1.0])
            .expect("representable final mean after retained-mass overflow");
        assert!(finite_after_mass_overflow.is_finite());
        assert!(
            (finite_after_mass_overflow - large / 2.0).abs()
                <= (large / 2.0) * 4.0 * f64::EPSILON
        );
        assert_eq!(
            scaled_compensated_mean(&[]),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
        assert_eq!(scaled_compensated_mean(&[0.0, 0.0]), Ok(0.0));
        assert_eq!(
            scaled_compensated_mean(&[f64::INFINITY]),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }

    fn overflowing_cwc_rate_rows(unit: u32, growing: bool) -> [EventTimedObservation; 3] {
        let delta = 1e-305_f64;
        let (first, second) = if growing {
            (f64::from_bits(1), f64::MAX)
        } else {
            (f64::MAX, f64::from_bits(1))
        };
        [
            timed(unit, 0.0, first),
            timed(unit, delta, second),
            timed(unit, 2.0 * delta, -f64::MAX),
        ]
    }

    #[test]
    fn cwc_pairwise_keeps_overflowed_rate_sum_via_compensated_mean() {
        let mut rows = overflowing_cwc_rate_rows(1, true).to_vec();
        rows.extend(overflowing_cwc_rate_rows(2, true));
        let recovered =
            recover_within_unit_irregular_residual_log_rate(&rows).expect("compensated");
        assert!(recovered.is_finite());
        let extracted = center_within_unit_event_lags(&rows).expect("extract");
        let mut rates = Vec::new();
        for pair in extracted {
            if !same_sign_nonzero(pair.earlier_residual(), pair.later_residual()) {
                continue;
            }
            rates.push(
                driver_same_sign_log_rate(
                    pair.earlier_residual(),
                    pair.later_residual(),
                    pair.event_interval(),
                )
                .expect("pair rate"),
            );
        }
        assert_eq!(rates.len(), 2);
        assert!(!(rates[0] + rates[1]).is_finite());
        let expected = scaled_compensated_mean(&rates).expect("reference compensated mean");
        assert!((recovered - expected).abs() <= expected.abs() * 1e-15);
        let mut mixed = overflowing_cwc_rate_rows(1, true).to_vec();
        mixed.extend(overflowing_cwc_rate_rows(2, false));
        let mixed_mean =
            recover_within_unit_irregular_residual_log_rate(&mixed).expect("mixed-sign");
        assert!(mixed_mean.abs() < recovered.abs() * 1e-12);
    }

    #[test]
    fn cwc_pairwise_tiny_interval_with_huge_log_ratio_fails_closed() {
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(1, 0.0, 1e-160),
                timed(1, f64::from_bits(1), 1e160),
                timed(1, 1.0, -1e160),
                timed(2, 0.0, 1.0),
                timed(2, 1.0, 0.5),
            ]),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
    }

    #[test]
    fn cwc_pairwise_skips_zero_residuals_and_keeps_same_sign_pairs() {
        let recovered = recover_within_unit_irregular_residual_log_rate(&[
            timed(1, 0.0, 7.0),
            timed(1, 1.0, 5.0),
            timed(1, 2.0, 4.0),
            timed(1, 3.0, 4.0),
            timed(2, 0.0, -1.2),
            timed(2, 1.0, -0.4),
            timed(2, 2.0, -0.8),
        ])
        .expect("skip zeros keep same-sign");
        assert!(recovered.is_finite());
        let extracted = center_within_unit_event_lags(&[
            timed(1, 0.0, 7.0),
            timed(1, 1.0, 5.0),
            timed(1, 2.0, 4.0),
            timed(1, 3.0, 4.0),
            timed(2, 0.0, -1.2),
            timed(2, 1.0, -0.4),
            timed(2, 2.0, -0.8),
        ])
        .expect("extract");
        assert!(
            extracted
                .iter()
                .any(|pair| pair.later_residual().to_bits() == 0.0_f64.to_bits())
        );
        assert!(
            extracted
                .iter()
                .any(|pair| pair.earlier_residual().to_bits() == 0.0_f64.to_bits())
        );
        assert!(extracted.iter().any(|pair| same_sign_nonzero(
            pair.earlier_residual(),
            pair.later_residual()
        ) && !pair.earlier_residual().is_sign_positive()));
    }

    #[test]
    fn cwc_observation_payload_paths_fail_closed() {
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[timed(1, 0.0, 1.0)]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(1, 0.0, 1.0),
                timed(1, 1.0, 0.5),
            ]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(1, 0.0, 1.0),
                timed(2, 1.0, 0.5),
            ]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            center_within_unit_event_lags(&[
                timed(1, 0.0, 1.0),
                timed(1, 0.0, 1.2),
                timed(2, 0.0, 2.0),
                timed(2, 1.0, 1.5),
            ]),
            Err(LongitudinalError::NonPositiveEventInterval)
        );
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(1, 0.0, f64::NAN),
                timed(1, 1.0, 1.0),
                timed(2, 0.0, 1.0),
                timed(2, 1.0, 0.5),
            ]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(1, 0.0, f64::INFINITY),
                timed(1, 1.0, 1.0),
                timed(2, 0.0, 1.0),
                timed(2, 1.0, 0.5),
            ]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(1, 0.0, f64::MAX),
                timed(1, 1.0, f64::MAX),
                timed(2, 0.0, 1.0),
                timed(2, 1.0, 0.5),
            ]),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
        assert_eq!(
            center_within_unit_event_lags(&[
                timed(1, f64::MAX, 1.0),
                timed(1, -f64::MAX, 0.5),
                timed(2, 0.0, 1.0),
                timed(2, 1.0, 0.5),
            ]),
            Err(LongitudinalError::NonPositiveEventInterval)
        );
    }

    #[test]
    fn already_centered_and_curran_refusal_paths_fail_closed() {
        assert_eq!(
            recover_centered_irregular_residual_log_rate(&[]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            recover_centered_irregular_residual_log_rate(&[lagged(f64::NAN, 0.5, 1.0)]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            recover_centered_irregular_residual_log_rate(&[lagged(1.0, f64::NAN, 1.0)]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            recover_centered_irregular_residual_log_rate(&[lagged(0.0, 0.5, 1.0)]),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
        assert_eq!(
            recover_centered_irregular_residual_log_rate(&[lagged(1.0, -0.5, 1.0)]),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
        let overflow_ratio = recover_centered_irregular_residual_log_rate(&[lagged(
            f64::from_bits(1),
            f64::MAX,
            1.0,
        )])
        .expect("finite log-domain overflow fallback");
        let expected_overflow = f64::MAX.ln() - f64::from_bits(1).ln();
        assert_eq!(overflow_ratio.to_bits(), expected_overflow.to_bits());
        let underflow_ratio = recover_centered_irregular_residual_log_rate(&[lagged(
            f64::MAX,
            f64::from_bits(1),
            1.0,
        )])
        .expect("finite log-domain underflow fallback");
        let expected_underflow = f64::from_bits(1).ln() - f64::MAX.ln();
        assert_eq!(underflow_ratio.to_bits(), expected_underflow.to_bits());
        assert_eq!(
            recover_centered_irregular_residual_log_rate(&[lagged(
                1e-160,
                1e160,
                f64::from_bits(1)
            )]),
            Err(LongitudinalError::InvalidTemporalTransformInput)
        );
        assert_eq!(
            refuse_cwc_residual_log_rate_as_raw_process_drift(f64::NAN, f64::INFINITY),
            Err(LongitudinalError::CwcResidualLogRateIsNotRawProcessDrift)
        );
    }

    #[test]
    fn cwc_orders_unsorted_event_times_before_lag_pairs() {
        let pairs = center_within_unit_event_lags(&[
            timed(1, 2.0, 0.4),
            timed(2, 3.0, -0.8),
            timed(1, 0.5, 1.6),
            timed(2, 1.0, 0.2),
        ])
        .expect("unsorted");
        assert_eq!(pairs.len(), 2);
        assert!((pairs[0].event_interval().as_f64() - 1.5).abs() < 1e-15);
        assert!((pairs[1].event_interval().as_f64() - 2.0).abs() < 1e-15);
        let cluster_one_mean = f64::midpoint(1.6, 0.4);
        assert!((pairs[0].earlier_residual() - (1.6 - cluster_one_mean)).abs() < 1e-15);
        assert!((pairs[0].later_residual() - (0.4 - cluster_one_mean)).abs() < 1e-15);
    }

    #[test]
    fn singleton_second_unit_does_not_satisfy_longitudinal_unit_floor() {
        let drift = -0.2_f64;
        let mixed = [
            timed(1, 0.0, 10.0 + 1.0),
            timed(1, 1.0, 10.0 + drift.exp()),
            timed(1, 2.0, 10.0 + (drift * 2.0).exp()),
            timed(1, 3.0, 10.0 + (drift * 3.0).exp()),
            timed(2, 0.0, 4.0),
        ];
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&mixed),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(1, 0.0, 1.0),
                timed(2, 1.0, 0.5)
            ]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
    }

    #[test]
    fn overflowing_same_sign_cwc_pairs_keep_stable_log() {
        let overflowed_both = recover_within_unit_irregular_residual_log_rate(&[
            timed(1, 0.0, f64::from_bits(1)),
            timed(1, 1.0, f64::MAX),
            timed(1, 2.0, -f64::MAX),
            timed(2, 0.0, f64::from_bits(1)),
            timed(2, 1.0, f64::MAX),
            timed(2, 2.0, -f64::MAX),
        ])
        .expect("stable log of overflowed same-sign CWC pairs");
        let overflow_rate = driver_same_sign_log_rate(f64::from_bits(1), f64::MAX, unit_interval())
            .expect("tiny/MAX");
        assert!((overflowed_both - overflow_rate).abs() < 1e-9);
    }

    #[test]
    fn one_sided_residual_overflow_fails_closed() {
        assert_eq!(
            recover_within_unit_irregular_residual_log_rate(&[
                timed(1, 0.0, -f64::MAX),
                timed(1, 1.0, -f64::MAX),
                timed(1, 2.0, -f64::MAX),
                timed(1, 3.0, f64::MAX),
                timed(2, 0.0, 1.0),
                timed(2, 1.0, 0.8),
            ]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
        assert_eq!(
            center_within_unit_event_lags(&[
                timed(1, 0.0, f64::MAX),
                timed(1, 1.0, -f64::MAX),
                timed(1, 2.0, -f64::MAX / 2.0),
                timed(2, 0.0, 1.0),
                timed(2, 1.0, 0.5),
            ]),
            Err(LongitudinalError::InvalidObservationPayload)
        );
    }
}
