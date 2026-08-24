//! Fail-closed input-process-outcome order errors.

use std::fmt;

/// A fail-closed input-process-outcome order error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OutcomeOrderError {
    /// An `input_to` or `process_to` edge moved backward in event time.
    ReverseIpoOrder,
    /// A transition IPO edge used equal event-time ranks.
    UncertainIpoOrder,
    /// An `outcome_of` provenance edge was treated as a state transition.
    OutcomeOfIsNotTransition,
    /// A recovery slice was empty or length-mismatched.
    InvalidEdgePayload,
}

impl fmt::Display for OutcomeOrderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ReverseIpoOrder => {
                "input-process-outcome transitions cannot move backward in event time"
            }
            Self::UncertainIpoOrder => {
                "input-process-outcome transitions require a strict event-time order"
            }
            Self::OutcomeOfIsNotTransition => "outcome_of is not a state transition",
            Self::InvalidEdgePayload => "invalid outcome-order payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for OutcomeOrderError {}

#[cfg(test)]
mod tests {
    use super::OutcomeOrderError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                OutcomeOrderError::ReverseIpoOrder,
                "input-process-outcome transitions cannot move backward in event time",
            ),
            (
                OutcomeOrderError::UncertainIpoOrder,
                "input-process-outcome transitions require a strict event-time order",
            ),
            (
                OutcomeOrderError::OutcomeOfIsNotTransition,
                "outcome_of is not a state transition",
            ),
            (
                OutcomeOrderError::InvalidEdgePayload,
                "invalid outcome-order payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
