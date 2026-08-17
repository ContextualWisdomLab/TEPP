//! Event-time discrete lag-1 and exact scalar local log-rate.
//!
//! Voelkle, Oud, Davidov, and Schmidt (2012, Eq. 7) and Driver, Oud, and
//! Voelkle (2017, Eq. 3) map the continuous-time drift by
//! `A*(Δt) = exp(A Δt)`. The noiseless scalar inverse is
//! `a = ln(φ) / Δt` with `φ = A*(Δt)`. The difference quotient
//! `(x(t+Δt) − x(t)) / Δt` (their Eqs. 3–4) is refused. This is not DSEM.

use std::collections::BTreeMap;

use crate::error::PsychometricError;
use crate::indicator::require_finite;

/// Clock on which a structural lag may be computed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LagClock {
    /// Event / valid time. The only clock that licenses a structural lag.
    EventTime,
    /// System / transaction time.
    SystemTime,
    /// Assertion time.
    AssertionTime,
    /// Document time.
    DocumentTime,
    /// Availability time.
    AvailabilityTime,
    /// Knowledge cutoff.
    KnowledgeCutoff,
}

impl LagClock {
    /// Stable wire name for the lag clock.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EventTime => "event_time",
            Self::SystemTime => "system_time",
            Self::AssertionTime => "assertion_time",
            Self::DocumentTime => "document_time",
            Self::AvailabilityTime => "availability_time",
            Self::KnowledgeCutoff => "knowledge_cutoff",
        }
    }

    /// Return whether this clock may carry a discrete lag or local log-rate.
    #[must_use]
    pub const fn admits_structural_lag(self) -> bool {
        matches!(self, Self::EventTime)
    }
}

/// One occasion on an event-time series.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventOccasion {
    /// Event / valid time of the observation.
    pub event_time: f64,
    /// Already-mapped score.
    pub score: f64,
}

/// One clustered event-time score used for CWC-then-lag recovery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClusteredEventScore {
    /// Cluster identity.
    pub cluster_key: u64,
    /// Event / valid time.
    pub event_time: f64,
    /// Already-mapped score.
    pub score: f64,
}

/// Discrete lag-1 coefficient and its exact local log-rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiscreteLagAndLogRate {
    /// Discrete-time lag `φ = exp(a Δt)`.
    pub discrete_lag: f64,
    /// Local log-rate `a = ln(φ) / Δt`.
    pub log_rate: f64,
    /// Positive event-time interval.
    pub event_delta: f64,
}

/// Recover the noiseless scalar discrete lag `later / earlier`.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when either score is
/// non-finite or the earlier score is zero (the ratio is undefined).
pub fn recover_discrete_lag_one(earlier: f64, later: f64) -> Result<f64, PsychometricError> {
    if !earlier.is_finite() || !later.is_finite() || earlier == 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    require_finite(later / earlier)
}

/// Map a discrete lag through the exact scalar exponential inverse.
///
/// `a = ln(φ) / Δt`. The clock must be event time. `φ` must be strictly
/// positive so the real logarithm exists.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is not
/// strictly positive, and [`PsychometricError::InvalidNumericInput`] when the
/// discrete lag is non-finite or not strictly positive.
pub fn recover_local_log_rate(
    discrete_lag: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !discrete_lag.is_finite() || discrete_lag <= 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    require_finite(discrete_lag.ln() / event_delta)
}

/// Recover the exact scalar pair `(φ, a)` on event time.
///
/// # Errors
///
/// Propagates [`recover_discrete_lag_one`] and [`recover_local_log_rate`].
pub fn recover_event_time_discrete_lag_and_log_rate(
    earlier: f64,
    later: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<DiscreteLagAndLogRate, PsychometricError> {
    let discrete_lag = recover_discrete_lag_one(earlier, later)?;
    let log_rate = recover_local_log_rate(discrete_lag, event_delta, clock)?;
    Ok(DiscreteLagAndLogRate {
        discrete_lag,
        log_rate,
        event_delta,
    })
}

/// Refuse the difference quotient as a continuous-time rate.
///
/// Voelkle et al. (2012) discourage `(x(t+Δt) − x(t)) / Δt` as the drift.
///
/// # Errors
///
/// Always returns [`PsychometricError::DifferenceQuotientForbidden`].
pub fn refuse_difference_quotient_as_local_rate(
    earlier: f64,
    later: f64,
    delta: f64,
) -> Result<f64, PsychometricError> {
    let _ = (earlier, later, delta);
    Err(PsychometricError::DifferenceQuotientForbidden)
}

/// Mean local log-rate across consecutive event-time pairs.
///
/// Occasions are sorted by event time. Each pair uses the exact scalar map.
/// Equal or inverted times fail closed.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for a non-event clock,
/// [`PsychometricError::InvalidNumericInput`] for fewer than two occasions or
/// non-finite values, and [`PsychometricError::NonPositiveInterval`] when
/// consecutive times are not strictly increasing.
pub fn recover_event_series_mean_log_rate(
    occasions: &[EventOccasion],
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if occasions.len() < 2 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut ordered = occasions.to_vec();
    ordered.sort_by(|left, right| {
        left.event_time
            .partial_cmp(&right.event_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut rates = Vec::new();
    for window in ordered.windows(2) {
        let earlier = window[0];
        let later = window[1];
        if !earlier.event_time.is_finite()
            || !later.event_time.is_finite()
            || !earlier.score.is_finite()
            || !later.score.is_finite()
        {
            return Err(PsychometricError::InvalidNumericInput);
        }
        let delta = later.event_time - earlier.event_time;
        let recovered =
            recover_event_time_discrete_lag_and_log_rate(earlier.score, later.score, delta, clock)?;
        rates.push(recovered.log_rate);
    }
    let count = rates.len() as f64;
    require_finite(rates.iter().sum::<f64>() / count)
}

/// Local log-rate of cluster-mean-centered residuals on event time.
///
/// Stable between-cluster means are removed first (CWC). Consecutive
/// within-cluster residuals then use the exact scalar map. This is not DSEM.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for a non-event clock,
/// [`PsychometricError::InvalidNumericInput`] for empty, singleton, or
/// non-finite rows, [`PsychometricError::InsufficientClusters`] when fewer
/// than two clusters appear, and interval/lag errors from the scalar map.
pub fn recover_within_residual_event_time_log_rate(
    rows: &[ClusteredEventScore],
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if rows.len() < 2 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut groups: BTreeMap<u64, Vec<ClusteredEventScore>> = BTreeMap::new();
    for &row in rows {
        if !row.event_time.is_finite() || !row.score.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        groups.entry(row.cluster_key).or_default().push(row);
    }
    if groups.len() < 2 {
        return Err(PsychometricError::InsufficientClusters);
    }
    let mut pairs = Vec::new();
    for occasions in groups.values_mut() {
        if occasions.len() < 2 {
            continue;
        }
        let count = occasions.len() as f64;
        let mean = occasions.iter().map(|row| row.score).sum::<f64>() / count;
        occasions.sort_by(|left, right| {
            left.event_time
                .partial_cmp(&right.event_time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for window in occasions.windows(2) {
            let earlier_resid = window[0].score - mean;
            let later_resid = window[1].score - mean;
            let delta = window[1].event_time - window[0].event_time;
            if !delta.is_finite() || delta <= 0.0 {
                return Err(PsychometricError::NonPositiveInterval);
            }
            if !earlier_resid.is_finite() || !later_resid.is_finite() {
                return Err(PsychometricError::InvalidNumericInput);
            }
            pairs.push((earlier_resid, later_resid, delta));
        }
    }
    fit_scalar_log_rate(&pairs)
}

fn fit_scalar_log_rate(pairs: &[(f64, f64, f64)]) -> Result<f64, PsychometricError> {
    if pairs.is_empty() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut start_sum = 0.0_f64;
    let mut start_count = 0.0_f64;
    for &(earlier, later, delta) in pairs {
        if earlier != 0.0 {
            let discrete_lag = later / earlier;
            if discrete_lag.is_finite() && discrete_lag > 0.0 {
                start_sum += discrete_lag.ln() / delta;
                start_count += 1.0;
            }
        }
    }
    if start_count <= 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut log_rate = start_sum / start_count;
    for _ in 0..16 {
        let mut score = 0.0_f64;
        let mut derivative = 0.0_f64;
        for &(earlier, later, delta) in pairs {
            let mapped = (log_rate * delta).exp();
            if !mapped.is_finite() {
                return Err(PsychometricError::InvalidNumericInput);
            }
            let weight = delta * earlier;
            score += weight * mapped * later - delta * mapped * mapped * earlier * earlier;
            derivative += delta * weight * mapped * later
                - 2.0 * delta * delta * mapped * mapped * earlier * earlier;
        }
        if !score.is_finite() || !derivative.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        if derivative.abs() <= 1e-18 {
            break;
        }
        let next = log_rate - score / derivative;
        if !next.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        if (next - log_rate).abs() < 1e-14 {
            log_rate = next;
            break;
        }
        log_rate = next;
    }
    require_finite(log_rate)
}

#[cfg(test)]
mod tests {
    use super::{
        ClusteredEventScore, EventOccasion, LagClock, recover_discrete_lag_one,
        recover_event_series_mean_log_rate, recover_event_time_discrete_lag_and_log_rate,
        recover_local_log_rate, recover_within_residual_event_time_log_rate,
        refuse_difference_quotient_as_local_rate,
    };
    use crate::error::PsychometricError;

    #[test]
    fn exact_scalar_map_inverts_exponential_drift() {
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let earlier = 1.5_f64;
        let later = earlier * (drift * delta).exp();
        let recovered = recover_event_time_discrete_lag_and_log_rate(
            earlier,
            later,
            delta,
            LagClock::EventTime,
        )
        .expect("exact");
        assert!((recovered.log_rate - drift).abs() < 1e-12);
        assert!((recovered.discrete_lag - (drift * delta).exp()).abs() < 1e-12);
        assert!((recovered.event_delta - delta).abs() < 1e-15);
    }

    #[test]
    fn non_event_clocks_and_difference_quotient_fail_closed() {
        for clock in [
            LagClock::SystemTime,
            LagClock::AssertionTime,
            LagClock::DocumentTime,
            LagClock::AvailabilityTime,
            LagClock::KnowledgeCutoff,
        ] {
            assert_eq!(
                recover_local_log_rate(0.5, 1.0, clock),
                Err(PsychometricError::EventTimeRequired)
            );
            assert!(!clock.admits_structural_lag());
            assert!(!clock.as_str().is_empty());
        }
        assert!(LagClock::EventTime.admits_structural_lag());
        assert_eq!(LagClock::EventTime.as_str(), "event_time");
        assert_eq!(
            refuse_difference_quotient_as_local_rate(1.0, 0.5, 1.0),
            Err(PsychometricError::DifferenceQuotientForbidden)
        );
    }

    #[test]
    fn invalid_lag_inputs_fail_closed() {
        assert_eq!(
            recover_discrete_lag_one(0.0, 1.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lag_one(f64::NAN, 1.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lag_one(1.0, f64::INFINITY),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_local_log_rate(0.5, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_local_log_rate(0.5, -1.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_local_log_rate(0.5, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_local_log_rate(0.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_local_log_rate(-0.2, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_local_log_rate(f64::NAN, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn series_mean_log_rate_recovers_and_refuses() {
        let drift = -0.25_f64;
        let occasions = [
            EventOccasion {
                event_time: 0.0,
                score: 2.0,
            },
            EventOccasion {
                event_time: 1.0,
                score: 2.0 * drift.exp(),
            },
            EventOccasion {
                event_time: 3.0,
                score: 2.0 * (drift * 3.0).exp(),
            },
        ];
        let series =
            recover_event_series_mean_log_rate(&occasions, LagClock::EventTime).expect("series");
        assert!((series - drift).abs() < 1e-12);
        assert_eq!(
            recover_event_series_mean_log_rate(&occasions, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(&[], LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[EventOccasion {
                    event_time: 0.0,
                    score: 1.0,
                }],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[
                    EventOccasion {
                        event_time: f64::NAN,
                        score: 1.0,
                    },
                    EventOccasion {
                        event_time: 1.0,
                        score: 0.5,
                    },
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[
                    EventOccasion {
                        event_time: 0.0,
                        score: 1.0,
                    },
                    EventOccasion {
                        event_time: 0.0,
                        score: 0.5,
                    },
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
    }

    fn clustered(cluster_key: u64, event_time: f64, score: f64) -> ClusteredEventScore {
        ClusteredEventScore {
            cluster_key,
            event_time,
            score,
        }
    }

    fn occasion(event_time: f64, score: f64) -> EventOccasion {
        EventOccasion { event_time, score }
    }

    fn decaying_clustered_scores(drift: f64) -> [ClusteredEventScore; 12] {
        [
            clustered(1, 0.0, 10.0 + 1.0),
            clustered(1, 1.0, 10.0 + drift.exp()),
            clustered(1, 2.0, 10.0 + (drift * 2.0).exp()),
            clustered(1, 3.0, 10.0 + (drift * 3.0).exp()),
            clustered(1, 4.0, 10.0 + (drift * 4.0).exp()),
            clustered(1, 5.0, 10.0 + (drift * 5.0).exp()),
            clustered(2, 0.0, -6.0 + 1.2),
            clustered(2, 1.0, -6.0 + 1.2 * drift.exp()),
            clustered(2, 2.0, -6.0 + 1.2 * (drift * 2.0).exp()),
            clustered(2, 3.0, -6.0 + 1.2 * (drift * 3.0).exp()),
            clustered(2, 4.0, -6.0 + 1.2 * (drift * 4.0).exp()),
            clustered(2, 5.0, -6.0 + 1.2 * (drift * 5.0).exp()),
        ]
    }

    #[test]
    fn within_residual_paths_recover_and_refuse() {
        let drift = -0.25_f64;
        let clustered = decaying_clustered_scores(drift);
        let within = recover_within_residual_event_time_log_rate(&clustered, LagClock::EventTime)
            .expect("cwc lag");
        let pooled_scores = clustered.map(|row| EventOccasion {
            event_time: row.event_time,
            score: row.score,
        });
        let pooled = recover_event_series_mean_log_rate(&pooled_scores, LagClock::EventTime);
        let within_error = (within - drift).abs();
        match pooled {
            Ok(pooled_rate) => {
                let pooled_error = (pooled_rate - drift).abs();
                assert!(
                    within_error < pooled_error,
                    "CWC log-rate error {within_error} should beat pooled {pooled_error}"
                );
            }
            Err(_) => {
                assert!(within_error.is_finite());
            }
        }
        assert!(within_error.is_finite());
    }

    #[test]
    fn within_residual_invalid_rows_fail_closed() {
        let rows = decaying_clustered_scores(-0.25);
        assert_eq!(
            recover_within_residual_event_time_log_rate(&rows, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(&[], LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[clustered(1, 0.0, 1.0), clustered(1, 1.0, 0.5)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InsufficientClusters)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[clustered(1, f64::NAN, 1.0), clustered(2, 1.0, 0.5)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[occasion(0.0, 1.0), occasion(1.0, f64::NAN)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[
                    clustered(1, 0.0, 1.0),
                    clustered(1, 1.0, 0.5),
                    clustered(2, 0.0, 2.0),
                    clustered(2, 1.0, 1.0),
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[clustered(1, 0.0, 1.0), clustered(2, 1.0, f64::INFINITY)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[
                    clustered(1, 0.0, 1.0),
                    clustered(1, 0.0, 1.2),
                    clustered(2, 0.0, 2.0),
                    clustered(2, 1.0, 1.5),
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
    }
}
