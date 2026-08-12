//! Fallible event mentions grounded in evidence.

use crate::{EventConfidence, EventError, EventMentionId};
use evidence_core::EvidenceId;

/// A fallible textual event mention that is **not** an event instance.
///
/// Mentions may be wrong, incomplete, or contradictory. Psychometric and
/// temporal estimators must not treat a mention as a ground-truth event without
/// an explicit promotion step that creates a distinct [`crate::EventInstance`].
#[derive(Clone, Debug, PartialEq)]
pub struct EventMention {
    mention_id: EventMentionId,
    evidence_id: EvidenceId,
    surface_form: String,
    confidence: EventConfidence,
}

impl EventMention {
    /// Construct a validated event mention.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::InvalidEventConfidence`] when confidence is invalid.
    /// Empty surface forms are rejected as invalid wire/domain payloads.
    pub fn new(
        evidence_id: EvidenceId,
        surface_form: impl Into<String>,
        confidence: EventConfidence,
    ) -> Result<Self, EventError> {
        let surface_form = surface_form.into();
        if surface_form.trim().is_empty() {
            return Err(EventError::InvalidWirePayload);
        }
        Ok(Self {
            mention_id: EventMentionId::new(),
            evidence_id,
            surface_form,
            confidence,
        })
    }

    /// Return the mention identifier.
    #[must_use]
    pub const fn mention_id(&self) -> EventMentionId {
        self.mention_id
    }

    /// Return the grounding evidence identifier.
    #[must_use]
    pub const fn evidence_id(&self) -> EvidenceId {
        self.evidence_id
    }

    /// Return the surface text.
    #[must_use]
    pub fn surface_form(&self) -> &str {
        &self.surface_form
    }

    /// Return mention confidence.
    #[must_use]
    pub const fn confidence(&self) -> EventConfidence {
        self.confidence
    }
}
