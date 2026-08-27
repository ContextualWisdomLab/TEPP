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
    /// A TDT link detection was treated as an event instance.
    EventLinkIsNotEventInstance,
    /// A TDT link detection was treated as a state transition.
    EventLinkIsNotStateTransition,
    /// An unknown event-link label was supplied.
    UnknownEventLinkLabel,
    /// A first-story detection was treated as an event instance.
    FirstStoryIsNotEventInstance,
    /// An unknown first-story label name was supplied.
    UnknownFirstStoryLabel,
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
    /// A CHRONOS occurrence prediction was treated as an event instance.
    PredictionIsNotEventInstance,
    /// An unknown occurrence-truth label was supplied.
    UnknownOccurrenceTruth,
    /// A span-grounded mention was treated as an event instance.
    SpanMentionIsNotEventInstance,
    /// Mention availability is after the knowledge cutoff.
    MentionIneligibleAtCutoff,
    /// The extractor or model version was empty or whitespace-only.
    EmptyExtractorVersion,
    /// The mention span does not belong to the supplied document.
    MentionSpanDocumentMismatch,
    /// An unknown mention-review status name was supplied.
    UnknownMentionReviewStatus,
    /// A TDT/CHRONOS composition was treated as an event instance.
    IntelligenceWorkflowIsNotEventInstance,
    /// A TDT/CHRONOS composition was treated as a state transition.
    IntelligenceWorkflowIsNotStateTransition,
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
            Self::EventLinkIsNotEventInstance => "event link is not an event instance",
            Self::EventLinkIsNotStateTransition => "event link is not a state transition",
            Self::UnknownEventLinkLabel => "unknown event link label",
            Self::FirstStoryIsNotEventInstance => "first-story detection is not an event instance",
            Self::UnknownFirstStoryLabel => "unknown first-story label",
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
            Self::PredictionIsNotEventInstance => "CHRONOS prediction is not an event instance",
            Self::UnknownOccurrenceTruth => "unknown occurrence truth label",
            Self::SpanMentionIsNotEventInstance => "span-grounded mention is not an event instance",
            Self::MentionIneligibleAtCutoff => "mention availability is after the knowledge cutoff",
            Self::EmptyExtractorVersion => "empty extractor version",
            Self::MentionSpanDocumentMismatch => "mention span does not belong to the document",
            Self::UnknownMentionReviewStatus => "unknown mention review status",
            Self::IntelligenceWorkflowIsNotEventInstance => {
                "intelligence workflow is not an event instance"
            }
            Self::IntelligenceWorkflowIsNotStateTransition => {
                "intelligence workflow is not a state transition"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for EventError {}

#[cfg(test)]
mod tests {
    use super::EventError;

    #[test]
    #[allow(clippy::too_many_lines)]
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
            ),
            (
                EventError::EventLinkIsNotEventInstance,
                "event link is not an event instance",
            ),
            (
                EventError::EventLinkIsNotStateTransition,
                "event link is not a state transition",
            ),
            (
                EventError::UnknownEventLinkLabel,
                "unknown event link label",
            ),
            (
                EventError::FirstStoryIsNotEventInstance,
                "first-story detection is not an event instance",
            ),
            (
                EventError::UnknownFirstStoryLabel,
                "unknown first-story label",
            ),
            (
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
            ),
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
            (
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
            (
                EventError::PredictionIsNotEventInstance,
                "CHRONOS prediction is not an event instance",
            ),
            (
                EventError::UnknownOccurrenceTruth,
                "unknown occurrence truth label",
            ),
            (
                EventError::SpanMentionIsNotEventInstance,
                "span-grounded mention is not an event instance",
            ),
            (
                EventError::MentionIneligibleAtCutoff,
                "mention availability is after the knowledge cutoff",
            ),
            (EventError::EmptyExtractorVersion, "empty extractor version"),
            (
                EventError::MentionSpanDocumentMismatch,
                "mention span does not belong to the document",
            ),
            (
                EventError::UnknownMentionReviewStatus,
                "unknown mention review status",
            ),
            (
                EventError::IntelligenceWorkflowIsNotEventInstance,
                "intelligence workflow is not an event instance",
            ),
            (
                EventError::IntelligenceWorkflowIsNotStateTransition,
                "intelligence workflow is not a state transition",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
