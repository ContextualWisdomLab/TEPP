//! Fail-closed event-ontology validation errors.

use std::fmt;

/// A fail-closed event-ontology error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EventError {
    /// Confidence was outside the closed unit interval or non-finite.
    InvalidEventConfidence,
    /// A mention was treated as an event instance without promotion.
    MentionIsNotEventInstance,
    /// An instance identity was reused for a different mention binding.
    DuplicateEventIdentity,
    /// A role assignment referenced an unknown instance.
    UnknownEventInstance,
    /// A wire payload was malformed or used an unsupported version.
    InvalidWirePayload,
    /// A wire payload used a schema version this crate does not support.
    UnsupportedWireVersion,
    /// An unknown event-role name was supplied.
    UnknownEventRole,
    /// A CHRONOS schema prediction was treated as an event instance.
    SchemaPredictionIsNotEventInstance,
    /// A CHRONOS schema prediction was treated as a state transition.
    SchemaPredictionIsNotStateTransition,
    /// An unknown schema-slot occupancy label was supplied.
    UnknownSchemaSlotLabel,
}

impl fmt::Display for EventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidEventConfidence => "invalid event confidence",
            Self::MentionIsNotEventInstance => "event mention is not an event instance",
            Self::DuplicateEventIdentity => "duplicate event identity",
            Self::UnknownEventInstance => "unknown event instance",
            Self::InvalidWirePayload => "invalid event wire payload",
            Self::UnsupportedWireVersion => "unsupported event wire version",
            Self::UnknownEventRole => "unknown event role",
            Self::SchemaPredictionIsNotEventInstance => {
                "schema prediction is not an event instance"
            }
            Self::SchemaPredictionIsNotStateTransition => {
                "schema prediction is not a state transition"
            }
            Self::UnknownSchemaSlotLabel => "unknown schema slot label",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for EventError {}

#[cfg(test)]
mod tests {
    use super::EventError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                EventError::InvalidEventConfidence,
                "invalid event confidence",
            ),
            (
                EventError::MentionIsNotEventInstance,
                "event mention is not an event instance",
            ),
            (
                EventError::DuplicateEventIdentity,
                "duplicate event identity",
            ),
            (EventError::UnknownEventInstance, "unknown event instance"),
            (EventError::InvalidWirePayload, "invalid event wire payload"),
            (
                EventError::UnsupportedWireVersion,
                "unsupported event wire version",
            ),
            (EventError::UnknownEventRole, "unknown event role"),
            (
                EventError::SchemaPredictionIsNotEventInstance,
                "schema prediction is not an event instance",
            ),
            (
                EventError::SchemaPredictionIsNotStateTransition,
                "schema prediction is not a state transition",
            ),
            (
                EventError::UnknownSchemaSlotLabel,
                "unknown schema slot label",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
