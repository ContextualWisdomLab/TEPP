//! Dual-clock observations and event-time lags.

use crate::IrregularTimeError;

/// One observation stamped with event time and system time in whole seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockedObservation {
    event_time_seconds: i64,
    system_time_seconds: i64,
}

impl ClockedObservation {
    /// Construct an observation with distinct TEPP clocks.
    ///
    /// Sequence checks happen in [`event_lag_seconds`].
    #[must_use]
    pub const fn new(event_time_seconds: i64, system_time_seconds: i64) -> Self {
        Self {
            event_time_seconds,
            system_time_seconds,
        }
    }

    /// Event/valid time in seconds.
    #[must_use]
    pub const fn event_time_seconds(self) -> i64 {
        self.event_time_seconds
    }

    /// System/record time in seconds.
    #[must_use]
    pub const fn system_time_seconds(self) -> i64 {
        self.system_time_seconds
    }
}

/// Consecutive event-time lags in seconds.
///
/// # Errors
///
/// Returns [`IrregularTimeError::InvalidObservationPayload`] when fewer than
/// two observations are supplied, or
/// [`IrregularTimeError::NonIncreasingEventTime`] when event time does not
/// strictly increase.
pub fn event_lag_seconds(
    observations: &[ClockedObservation],
) -> Result<Vec<i64>, IrregularTimeError> {
    if observations.len() < 2 {
        return Err(IrregularTimeError::InvalidObservationPayload);
    }
    let mut lags = Vec::with_capacity(observations.len() - 1);
    for window in observations.windows(2) {
        let delta = window[1].event_time_seconds - window[0].event_time_seconds;
        if delta <= 0 {
            return Err(IrregularTimeError::NonIncreasingEventTime);
        }
        lags.push(delta);
    }
    Ok(lags)
}

/// Refuse to treat equal system-time spacing as event-time spacing.
///
/// # Errors
///
/// Returns lag-construction errors, or
/// [`IrregularTimeError::SystemSpacingIsNotEventSpacing`] when system-time
/// deltas are constant while event-time deltas are not.
pub fn refuse_equal_system_spacing_as_event_spacing(
    observations: &[ClockedObservation],
) -> Result<(), IrregularTimeError> {
    let event_lags = event_lag_seconds(observations)?;
    let mut system_lags = Vec::with_capacity(event_lags.len());
    for window in observations.windows(2) {
        system_lags.push(window[1].system_time_seconds - window[0].system_time_seconds);
    }
    let system_constant = system_lags.windows(2).all(|pair| pair[0] == pair[1]);
    let event_varies = event_lags.windows(2).any(|pair| pair[0] != pair[1]);
    if system_constant && event_varies {
        return Err(IrregularTimeError::SystemSpacingIsNotEventSpacing);
    }
    Ok(())
}

/// RMSE of recovered lags against known-truth event lags.
///
/// # Errors
///
/// Returns [`IrregularTimeError::InvalidObservationPayload`] when either slice
/// is empty or the lengths differ.
pub fn lag_root_mean_square_error(
    truth: &[i64],
    decided: &[i64],
) -> Result<f64, IrregularTimeError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(IrregularTimeError::InvalidObservationPayload);
    }
    let mut sum_squares = 0.0_f64;
    for (truth_lag, decided_lag) in truth.iter().zip(decided) {
        let residual = *decided_lag as f64 - *truth_lag as f64;
        sum_squares += residual * residual;
    }
    Ok((sum_squares / truth.len() as f64).sqrt())
}

#[cfg(test)]
mod tests {
    use super::{
        ClockedObservation, event_lag_seconds, lag_root_mean_square_error,
        refuse_equal_system_spacing_as_event_spacing,
    };
    use crate::IrregularTimeError;

    #[test]
    fn matching_clocks_and_empty_rmse_cover_local_branches() {
        let regular = [
            ClockedObservation::new(0, 0),
            ClockedObservation::new(2, 2),
            ClockedObservation::new(4, 4),
        ];
        refuse_equal_system_spacing_as_event_spacing(&regular).expect("matching clocks");
        assert_eq!(event_lag_seconds(&regular).expect("lags"), vec![2, 2]);
        assert_eq!(regular[0].event_time_seconds(), 0);
        assert_eq!(regular[0].system_time_seconds(), 0);
        assert_eq!(
            lag_root_mean_square_error(&[], &[]),
            Err(IrregularTimeError::InvalidObservationPayload)
        );
        assert_eq!(
            lag_root_mean_square_error(&[2, 2], &[]),
            Err(IrregularTimeError::InvalidObservationPayload)
        );
        let matched = lag_root_mean_square_error(&[2, 2], &[2, 2]).expect("rmse");
        assert!(matched.abs() < f64::EPSILON);
        let irregular_system = [
            ClockedObservation::new(0, 0),
            ClockedObservation::new(10, 1),
            ClockedObservation::new(13, 5),
        ];
        refuse_equal_system_spacing_as_event_spacing(&irregular_system)
            .expect("non-constant system lags are not equal system spacing");
        let equal_system_irregular_event = [
            ClockedObservation::new(0, 0),
            ClockedObservation::new(10, 1),
            ClockedObservation::new(13, 2),
        ];
        assert_eq!(
            refuse_equal_system_spacing_as_event_spacing(&equal_system_irregular_event),
            Err(IrregularTimeError::SystemSpacingIsNotEventSpacing)
        );
    }
}
