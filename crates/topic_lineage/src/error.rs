//! Fail-closed topic-lineage errors.

use std::fmt;

/// A fail-closed topic-lineage error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TopicLineageError {
    /// A reactivation tried to mint a new topic identity.
    ReactivationIsNotNewTopic,
    /// An activity transition was not allowed from the current state.
    InvalidActivityTransition,
    /// Identity slices were empty or length-mismatched.
    InvalidIdentityPayload,
}

impl fmt::Display for TopicLineageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ReactivationIsNotNewTopic => "reactivation is not a new topic",
            Self::InvalidActivityTransition => "invalid topic activity transition",
            Self::InvalidIdentityPayload => "invalid topic identity payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TopicLineageError {}

#[cfg(test)]
mod tests {
    use super::TopicLineageError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                TopicLineageError::ReactivationIsNotNewTopic,
                "reactivation is not a new topic",
            ),
            (
                TopicLineageError::InvalidActivityTransition,
                "invalid topic activity transition",
            ),
            (
                TopicLineageError::InvalidIdentityPayload,
                "invalid topic identity payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
