//! Fail-closed interpretation-gateway errors.

use std::fmt;

/// A fail-closed interpretation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InterpretationError {
    /// An interpretation was proposed without at least one evidence span.
    MissingEvidenceSpan,
    /// An interpretation was treated as a statistical estimator result.
    InterpretationIsNotEstimatorResult,
    /// An interpretation was treated as an observed fact.
    InterpretationIsNotObservedFact,
    /// Support labels were empty or length-mismatched.
    InvalidSupportPayload,
}

impl fmt::Display for InterpretationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingEvidenceSpan => "interpretation is missing an evidence span",
            Self::InterpretationIsNotEstimatorResult => "interpretation is not an estimator result",
            Self::InterpretationIsNotObservedFact => "interpretation is not an observed fact",
            Self::InvalidSupportPayload => "invalid interpretation support payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for InterpretationError {}

#[cfg(test)]
mod tests {
    use super::InterpretationError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                InterpretationError::MissingEvidenceSpan,
                "interpretation is missing an evidence span",
            ),
            (
                InterpretationError::InterpretationIsNotEstimatorResult,
                "interpretation is not an estimator result",
            ),
            (
                InterpretationError::InterpretationIsNotObservedFact,
                "interpretation is not an observed fact",
            ),
            (
                InterpretationError::InvalidSupportPayload,
                "invalid interpretation support payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
