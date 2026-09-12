//! Event-time value objects for longitudinal composition.

use crate::LongitudinalError;

/// A finite, strictly positive interval on TEPP's substantive event-time clock.
///
/// Constructing this value is the admission boundary between generic numeric
/// durations and Longitudinal Modeling operations that are explicitly defined
/// on event time. Measurement-occasion, assertion, document, system, and
/// availability-clock durations must not be re-labelled as this type without
/// an owning-context conversion that proves the semantic mapping.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventTimeInterval(f64);

impl EventTimeInterval {
    /// Admit a finite, strictly positive interval as substantive event time.
    ///
    /// # Errors
    ///
    /// Returns [`LongitudinalError::NonPositiveEventInterval`] when `value` is
    /// non-finite, zero, or negative.
    pub fn new(value: f64) -> Result<Self, LongitudinalError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(LongitudinalError::NonPositiveEventInterval);
        }
        Ok(Self(value))
    }

    /// Return the admitted interval in the model's declared event-time unit.
    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::EventTimeInterval;
    use crate::LongitudinalError;

    #[test]
    fn admits_only_finite_positive_event_intervals() {
        assert_eq!(
            EventTimeInterval::new(0.0),
            Err(LongitudinalError::NonPositiveEventInterval)
        );
        assert_eq!(
            EventTimeInterval::new(-1.0),
            Err(LongitudinalError::NonPositiveEventInterval)
        );
        assert_eq!(
            EventTimeInterval::new(f64::NAN),
            Err(LongitudinalError::NonPositiveEventInterval)
        );
        assert_eq!(
            EventTimeInterval::new(f64::INFINITY),
            Err(LongitudinalError::NonPositiveEventInterval)
        );
        assert_eq!(
            EventTimeInterval::new(0.25).map(EventTimeInterval::as_f64),
            Ok(0.25)
        );
    }
}
