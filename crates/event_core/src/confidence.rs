//! Event confidence on the closed unit interval.

use crate::EventError;
use serde::{Deserialize, Serialize};

/// Finite confidence in `[0.0, 1.0]`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct EventConfidence(f64);

impl EventConfidence {
    /// Construct a validated confidence.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::InvalidEventConfidence`] outside `[0, 1]` or when
    /// non-finite.
    pub fn new(value: f64) -> Result<Self, EventError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(EventError::InvalidEventConfidence)
        }
    }

    /// Certain event confidence of one.
    ///
    /// # Errors
    ///
    /// Never fails for the constant one.
    pub fn certain() -> Result<Self, EventError> {
        Self::new(1.0)
    }

    /// Return the numeric confidence.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// Mean squared error of mention probabilities against binary truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the slices are empty or
/// have unequal length.
pub fn mention_brier_score(
    forecasts: &[EventConfidence],
    outcomes: &[bool],
) -> Result<f64, EventError> {
    if forecasts.is_empty() || forecasts.len() != outcomes.len() {
        return Err(EventError::InvalidWirePayload);
    }
    let mut square_sum = 0.0_f64;
    for (forecast, outcome) in forecasts.iter().zip(outcomes) {
        let target = if *outcome { 1.0 } else { 0.0 };
        let residual = forecast.value() - target;
        square_sum += residual * residual;
    }
    #[allow(clippy::cast_precision_loss)]
    Ok(square_sum / forecasts.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::EventConfidence;
    use crate::EventError;

    #[test]
    fn confidence_rejects_values_outside_the_unit_interval() {
        assert!(EventConfidence::new(0.0).is_ok());
        assert!(EventConfidence::certain().is_ok());
        assert_eq!(
            EventConfidence::new(1.1),
            Err(EventError::InvalidEventConfidence)
        );
        assert_eq!(
            EventConfidence::new(f64::NAN),
            Err(EventError::InvalidEventConfidence)
        );
        let one = EventConfidence::certain().expect("certain");
        assert!((one.value() - 1.0).abs() < 1e-15);
        let miss = super::mention_brier_score(&[one], &[false]).expect("miss");
        assert!((miss - 1.0).abs() < 1e-15);
    }
}
