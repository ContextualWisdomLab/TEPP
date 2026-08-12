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
    }
}
