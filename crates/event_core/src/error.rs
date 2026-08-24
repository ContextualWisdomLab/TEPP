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
    /// A TDT detection or mention was treated as a state transition.
    DetectionIsNotTransition,
    /// A CHRONOS prediction was treated as an observed or promoted fact.
    PredictionIsNotFact,
    /// A TDT track assignment was treated as an event instance.
    EventTrackIsNotEventInstance,
    /// A TDT track assignment was treated as a state transition.
    EventTrackIsNotStateTransition,
    /// An unknown event-track label was supplied.
    UnknownEventTrackLabel,
    /// A CHRONOS schema prediction was treated as an event instance.
    SchemaPredictionIsNotEventInstance,
    /// A CHRONOS schema prediction was treated as a state transition.
    SchemaPredictionIsNotStateTransition,
    /// An unknown schema-slot occupancy label was supplied.
    UnknownSchemaSlotLabel,
    /// A TDT story segmentation was treated as an event instance.
    StorySegmentationIsNotEventInstance,
    /// A TDT story segmentation was treated as a state transition.
    StorySegmentationIsNotStateTransition,
    /// An unknown story-boundary label was supplied.
    UnknownStoryBoundaryLabel,
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
            Self::DetectionIsNotTransition => "detection is not a state transition",
            Self::PredictionIsNotFact => "prediction is not an observed fact",
            Self::EventTrackIsNotEventInstance => "event track is not an event instance",
            Self::EventTrackIsNotStateTransition => "event track is not a state transition",
            Self::UnknownEventTrackLabel => "unknown event track label",
            Self::SchemaPredictionIsNotEventInstance => {
                "schema prediction is not an event instance"
            }
            Self::SchemaPredictionIsNotStateTransition => {
                "schema prediction is not a state transition"
            }
            Self::UnknownSchemaSlotLabel => "unknown schema slot label",
            Self::StorySegmentationIsNotEventInstance => {
                "story segmentation is not an event instance"
            }
            Self::StorySegmentationIsNotStateTransition => {
                "story segmentation is not a state transition"
            }
            Self::UnknownStoryBoundaryLabel => "unknown story boundary label",
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
                EventError::DetectionIsNotTransition,
                "detection is not a state transition",
            ),
            (
                EventError::PredictionIsNotFact,
                "prediction is not an observed fact",
                EventError::EventTrackIsNotEventInstance,
                "event track is not an event instance",
            ),
            (
                EventError::EventTrackIsNotStateTransition,
                "event track is not a state transition",
            ),
            (
                EventError::UnknownEventTrackLabel,
                "unknown event track label",
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
                EventError::StorySegmentationIsNotEventInstance,
                "story segmentation is not an event instance",
            ),
            (
                EventError::StorySegmentationIsNotStateTransition,
                "story segmentation is not a state transition",
            ),
            (
                EventError::UnknownStoryBoundaryLabel,
                "unknown story boundary label",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
