//! Fallible event mentions grounded in one exact source extent.

use crate::span_mention::{MentionEvidenceClocks, MentionReviewStatus};
use crate::{EventConfidence, EventError, EventEvidenceLayer, EventInstanceId, EventMentionId};
use evidence_core::{DocumentRecord, EvidenceId, SourceSpan};

/// A fallible textual event mention that is **not** an event instance.
///
/// Mentions may be wrong, incomplete, or contradictory. Every mention cites
/// one exact [`SourceSpan`], document identity, six-clock evidence, extractor
/// or model version, and proposed-or-reviewed inspection status. The surface
/// form is the document substring selected by that span. Psychometric and
/// temporal estimators must not treat a mention as a ground-truth event without
/// an explicit promotion step that creates a distinct [`crate::EventInstance`].
#[derive(Clone, Debug, PartialEq)]
pub struct EventMention {
    mention_id: EventMentionId,
    evidence_id: EvidenceId,
    surface_form: String,
    confidence: EventConfidence,
    source_span: SourceSpan,
    clocks: MentionEvidenceClocks,
    extractor_version: String,
    review_status: MentionReviewStatus,
}

impl EventMention {
    /// Bind a mention to one validated document span and six-clock evidence.
    ///
    /// The surface form is the document substring at `source_span`. Reviewed
    /// status is authorized inspection and is not instance promotion.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::MentionSpanDocumentMismatch`] when the span does
    /// not belong to `document`, or [`EventError::EmptyExtractorVersion`] when
    /// the extractor version is empty or whitespace-only.
    pub fn new(
        document: &DocumentRecord,
        source_span: SourceSpan,
        confidence: EventConfidence,
        clocks: MentionEvidenceClocks,
        extractor_version: impl Into<String>,
        review_status: MentionReviewStatus,
    ) -> Result<Self, EventError> {
        if document.id() != source_span.document_id() {
            return Err(EventError::MentionSpanDocumentMismatch);
        }
        let extractor_version = extractor_version.into();
        if extractor_version.trim().is_empty() {
            return Err(EventError::EmptyExtractorVersion);
        }
        let surface_form =
            document.text()[source_span.byte_start()..source_span.byte_end()].to_string();
        Ok(Self {
            mention_id: EventMentionId::new(),
            evidence_id: source_span.document_id(),
            surface_form,
            confidence,
            source_span,
            clocks,
            extractor_version,
            review_status,
        })
    }

    /// Return the mention identifier.
    #[must_use]
    pub const fn mention_id(&self) -> EventMentionId {
        self.mention_id
    }

    /// Return the grounding document identifier.
    #[must_use]
    pub const fn document_id(&self) -> EvidenceId {
        self.source_span.document_id()
    }

    /// Return the grounding evidence identifier.
    #[must_use]
    pub const fn evidence_id(&self) -> EvidenceId {
        self.evidence_id
    }

    /// Return the exact document substring selected by the span.
    #[must_use]
    pub fn surface_form(&self) -> &str {
        &self.surface_form
    }

    /// Return mention confidence.
    #[must_use]
    pub const fn confidence(&self) -> EventConfidence {
        self.confidence
    }

    /// Return the exact source extent.
    #[must_use]
    pub const fn source_span(&self) -> SourceSpan {
        self.source_span
    }

    /// Return the six-clock evidence.
    #[must_use]
    pub const fn clocks(&self) -> MentionEvidenceClocks {
        self.clocks
    }

    /// Return the extractor or model version.
    #[must_use]
    pub fn extractor_version(&self) -> &str {
        &self.extractor_version
    }

    /// Return the review status.
    #[must_use]
    pub const fn review_status(&self) -> MentionReviewStatus {
        self.review_status
    }

    /// Return the epistemic layer retained by the mention.
    #[must_use]
    pub const fn evidence_layer(&self) -> EventEvidenceLayer {
        EventEvidenceLayer::ObservedMention
    }
}

/// Explicit refusal to treat a span-grounded mention as an event instance.
///
/// # Errors
///
/// Always returns [`EventError::SpanMentionIsNotEventInstance`].
pub fn refuse_span_mention_as_instance(
    _mention: &EventMention,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::SpanMentionIsNotEventInstance)
}

#[cfg(test)]
mod tests {
    use super::{EventMention, refuse_span_mention_as_instance};
    use crate::span_mention::{MentionEvidenceClocks, MentionReviewStatus};
    use crate::{EventConfidence, EventError, EventEvidenceLayer};
    use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
    use temporal_core::{
        AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
    };

    fn document(text: &str) -> DocumentRecord {
        let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
        DocumentRecord::from_text(artifact.id(), text).expect("document")
    }

    fn span(document: &DocumentRecord, surface: &str) -> SourceSpan {
        let byte_start = document.text().find(surface).expect("surface");
        let byte_end = byte_start + surface.len();
        let scalar_start = document.text()[..byte_start].chars().count();
        let scalar_end = scalar_start + surface.chars().count();
        SourceSpan::new(
            document,
            byte_start,
            byte_end,
            scalar_start,
            scalar_end,
            None,
        )
        .expect("span")
    }

    fn clocks() -> MentionEvidenceClocks {
        MentionEvidenceClocks::new(
            EventTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("event"),
            AssertionTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("assertion"),
            DocumentTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("document"),
            SystemTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("system"),
            AvailableTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("available"),
            KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff"),
        )
        .expect("clocks")
    }

    #[test]
    fn grounded_mention_accessors_and_refusals_are_covered() {
        let record = document("award filed");
        let mention = EventMention::new(
            &record,
            span(&record, "award"),
            EventConfidence::certain().expect("certain"),
            clocks(),
            "ace-extent-extractor/1",
            MentionReviewStatus::Reviewed,
        )
        .expect("mention");
        let _ = mention.mention_id();
        assert_eq!(mention.document_id(), record.id());
        assert_eq!(mention.evidence_id(), record.id());
        assert_eq!(mention.surface_form(), "award");
        assert!((mention.confidence().value() - 1.0).abs() < f64::EPSILON);
        assert_eq!(mention.source_span().byte_start(), 0);
        assert_eq!(mention.clocks().available_time(), clocks().available_time());
        assert_eq!(mention.extractor_version(), "ace-extent-extractor/1");
        assert_eq!(mention.review_status(), MentionReviewStatus::Reviewed);
        assert_eq!(
            mention.evidence_layer(),
            EventEvidenceLayer::ObservedMention
        );
        assert_eq!(
            refuse_span_mention_as_instance(&mention),
            Err(EventError::SpanMentionIsNotEventInstance)
        );
        let other = document("award filed");
        assert_eq!(
            EventMention::new(
                &other,
                span(&record, "award"),
                EventConfidence::certain().expect("certain"),
                clocks(),
                "ace-extent-extractor/1",
                MentionReviewStatus::Proposed,
            )
            .map(|_| ()),
            Err(EventError::MentionSpanDocumentMismatch)
        );
        assert_eq!(
            EventMention::new(
                &record,
                span(&record, "award"),
                EventConfidence::certain().expect("certain"),
                clocks(),
                "",
                MentionReviewStatus::Proposed,
            )
            .map(|_| ()),
            Err(EventError::EmptyExtractorVersion)
        );
    }
}
