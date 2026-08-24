//! Fail-closed ESEM/DSEM fit errors.

use std::fmt;

/// A fail-closed psychometric-fit error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PsychometricFitError {
    /// Raw simplex proportions were offered as Euclidean fit inputs.
    RawProportionForbidden,
    /// Empty, rank-unsupported, unequal-length, or non-finite numeric input.
    InvalidNumericInput,
    /// A predictor matrix has a singular Gram matrix.
    SingularDesign,
    /// A lagged path would move backward or stay put in event time.
    ReverseEventTimePath,
    /// A good global fit was used to reinterpret a formative or network
    /// construct as reflective.
    FormativeReinterpretationForbidden,
    /// The construct class is unresolved, so reflective interpretation is
    /// unavailable.
    UnresolvedConstruct,
}

impl fmt::Display for PsychometricFitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::RawProportionForbidden => {
                "raw topic proportions are forbidden psychometric fit inputs"
            }
            Self::InvalidNumericInput => "invalid psychometric fit numeric input",
            Self::SingularDesign => "singular psychometric fit design matrix",
            Self::ReverseEventTimePath => "DSEM lagged paths cannot move backward in event time",
            Self::FormativeReinterpretationForbidden => {
                "formative or network constructs cannot be reinterpreted as reflective"
            }
            Self::UnresolvedConstruct => "construct class is unresolved",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PsychometricFitError {}

#[cfg(test)]
mod tests {
    use super::PsychometricFitError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                PsychometricFitError::RawProportionForbidden,
                "raw topic proportions are forbidden psychometric fit inputs",
            ),
            (
                PsychometricFitError::InvalidNumericInput,
                "invalid psychometric fit numeric input",
            ),
            (
                PsychometricFitError::SingularDesign,
                "singular psychometric fit design matrix",
            ),
            (
                PsychometricFitError::ReverseEventTimePath,
                "DSEM lagged paths cannot move backward in event time",
            ),
            (
                PsychometricFitError::FormativeReinterpretationForbidden,
                "formative or network constructs cannot be reinterpreted as reflective",
            ),
            (
                PsychometricFitError::UnresolvedConstruct,
                "construct class is unresolved",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
