//! Fail-closed prediction-contradiction errors.

use std::fmt;

/// A fail-closed prediction-contradiction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PredictionContradictionError {
    /// A predicted interval was disjoint from later-observed evidence.
    PredictionContradictsObservation,
    /// An interval or recovery slice was empty, inverted, or length-mismatched.
    InvalidIntervalPayload,
}

impl fmt::Display for PredictionContradictionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::PredictionContradictsObservation => {
                "predicted interval contradicts observed evidence"
            }
            Self::InvalidIntervalPayload => "invalid prediction-contradiction payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PredictionContradictionError {}

#[cfg(test)]
mod tests {
    use super::PredictionContradictionError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                PredictionContradictionError::PredictionContradictsObservation,
                "predicted interval contradicts observed evidence",
            ),
            (
                PredictionContradictionError::InvalidIntervalPayload,
                "invalid prediction-contradiction payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
