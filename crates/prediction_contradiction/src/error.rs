//! Fail-closed prediction-contradiction errors.

use std::fmt;

/// A fail-closed prediction-contradiction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PredictionContradictionError {
    /// Predicted and observed event-time intervals are Allen `before` or `after`.
    PredictionContradictsObservation,
    /// Predicted and observed intervals are adjacent and do not overlap in their interiors.
    PredictionLacksOverlappingSupport,
    /// Observed evidence overlaps the prediction but does not cover it.
    PredictionNotCoveredByObservation,
    /// Observed evidence became available after the analysis knowledge cutoff.
    EvidenceAfterCutoff,
    /// An interval is not a closed proper Allen input.
    InvalidIntervalPayload,
    /// An agreement-rate comparison used empty or length-mismatched slices.
    AgreementSliceMismatch,
}

impl fmt::Display for PredictionContradictionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::PredictionContradictsObservation => {
                "predicted interval contradicts observed evidence"
            }
            Self::PredictionLacksOverlappingSupport => {
                "predicted interval is adjacent to observation without overlapping support"
            }
            Self::PredictionNotCoveredByObservation => {
                "observed evidence does not cover the predicted interval"
            }
            Self::EvidenceAfterCutoff => {
                "observed evidence is available after the knowledge cutoff"
            }
            Self::InvalidIntervalPayload => "invalid prediction-contradiction payload",
            Self::AgreementSliceMismatch => "agreement slices are empty or length-mismatched",
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
                PredictionContradictionError::PredictionLacksOverlappingSupport,
                "predicted interval is adjacent to observation without overlapping support",
            ),
            (
                PredictionContradictionError::PredictionNotCoveredByObservation,
                "observed evidence does not cover the predicted interval",
            ),
            (
                PredictionContradictionError::EvidenceAfterCutoff,
                "observed evidence is available after the knowledge cutoff",
            ),
            (
                PredictionContradictionError::InvalidIntervalPayload,
                "invalid prediction-contradiction payload",
            ),
            (
                PredictionContradictionError::AgreementSliceMismatch,
                "agreement slices are empty or length-mismatched",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
