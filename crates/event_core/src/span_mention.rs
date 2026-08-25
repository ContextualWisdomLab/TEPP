//! Span-grounded mentions stay distinct from promoted event instances.

use crate::{EventConfidence, EventError, EventEvidenceLayer, EventInstanceId, EventMentionId};
use evidence_core::{DocumentRecord, EvidenceId, SourceSpan};
use std::collections::BTreeSet;
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
};

/// Review status of one fallible mention.
///
/// Reviewed status is authorized inspection. It is not promotion into an
/// event instance or a forward state transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MentionReviewStatus {
    /// The mention is hypothesized and has not been reviewed.
    Proposed,
    /// An authorized reviewer inspected the mention.
    Reviewed,
}

impl MentionReviewStatus {
    /// Return the stable wire review-status name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Reviewed => "reviewed",
        }
    }

    /// Parse a stable wire review-status name.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::UnknownMentionReviewStatus`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, EventError> {
        match name {
            "proposed" => Ok(Self::Proposed),
            "reviewed" => Ok(Self::Reviewed),
            _ => Err(EventError::UnknownMentionReviewStatus),
        }
    }

    /// Return whether an authorized reviewer inspected the mention.
    #[must_use]
    pub const fn is_reviewed(self) -> bool {
        matches!(self, Self::Reviewed)
    }
}

/// Six-clock evidence bound to one mention.
///
/// Availability after the knowledge cutoff fails closed. Event time, assertion
/// time, and document time cannot replace availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MentionEvidenceClocks {
    event_time: EventTime,
    assertion_time: AssertionTime,
    document_time: DocumentTime,
    system_time: SystemTime,
    available_time: AvailableTime,
    knowledge_cutoff: KnowledgeCutoff,
}

impl MentionEvidenceClocks {
    /// Bind six typed clocks and refuse late availability.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::MentionIneligibleAtCutoff`] when availability is
    /// after the knowledge cutoff.
    pub fn new(
        event_time: EventTime,
        assertion_time: AssertionTime,
        document_time: DocumentTime,
        system_time: SystemTime,
        available_time: AvailableTime,
        knowledge_cutoff: KnowledgeCutoff,
    ) -> Result<Self, EventError> {
        if available_time.instant().as_nanosecond() > knowledge_cutoff.instant().as_nanosecond() {
            return Err(EventError::MentionIneligibleAtCutoff);
        }
        Ok(Self {
            event_time,
            assertion_time,
            document_time,
            system_time,
            available_time,
            knowledge_cutoff,
        })
    }

    /// Return the claimed event time.
    #[must_use]
    pub const fn event_time(self) -> EventTime {
        self.event_time
    }

    /// Return the assertion time.
    #[must_use]
    pub const fn assertion_time(self) -> AssertionTime {
        self.assertion_time
    }

    /// Return the document time.
    #[must_use]
    pub const fn document_time(self) -> DocumentTime {
        self.document_time
    }

    /// Return the system time.
    #[must_use]
    pub const fn system_time(self) -> SystemTime {
        self.system_time
    }

    /// Return the availability time.
    #[must_use]
    pub const fn available_time(self) -> AvailableTime {
        self.available_time
    }

    /// Return the knowledge cutoff.
    #[must_use]
    pub const fn knowledge_cutoff(self) -> KnowledgeCutoff {
        self.knowledge_cutoff
    }
}

/// A fallible mention grounded in one exact source extent.
///
/// The surface form is the document substring selected by the span. The
/// mention remains observed evidence and cannot promote an instance.
#[derive(Clone, Debug, PartialEq)]
pub struct SpanGroundedMention {
    mention_id: EventMentionId,
    evidence_id: EvidenceId,
    surface_form: String,
    confidence: EventConfidence,
    source_span: SourceSpan,
    clocks: MentionEvidenceClocks,
    extractor_version: String,
    review_status: MentionReviewStatus,
}

impl SpanGroundedMention {
    /// Bind a mention to one validated document span and six-clock evidence.
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
    _mention: &SpanGroundedMention,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::SpanMentionIsNotEventInstance)
}

/// Precision of recovered mention extents against known-truth extents.
///
/// An extent matches when document identity and exact byte bounds agree.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when either extent set is empty
/// or a `(document, byte_start, byte_end)` triple is duplicated.
pub fn mention_span_precision(
    truth: &[SourceSpan],
    recovered: &[SourceSpan],
) -> Result<f64, EventError> {
    let truth_spans = unique_extent_set(truth)?;
    let recovered_spans = unique_extent_set(recovered)?;
    counted_rate(
        recovered_spans.intersection(&truth_spans).count(),
        recovered_spans.len(),
    )
}

/// Recall of recovered mention extents against known-truth extents.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when either extent set is empty
/// or a `(document, byte_start, byte_end)` triple is duplicated.
pub fn mention_span_recall(
    truth: &[SourceSpan],
    recovered: &[SourceSpan],
) -> Result<f64, EventError> {
    let truth_spans = unique_extent_set(truth)?;
    let recovered_spans = unique_extent_set(recovered)?;
    counted_rate(
        recovered_spans.intersection(&truth_spans).count(),
        truth_spans.len(),
    )
}

fn unique_extent_set(spans: &[SourceSpan]) -> Result<BTreeSet<(u128, usize, usize)>, EventError> {
    if spans.is_empty() {
        return Err(EventError::InvalidWirePayload);
    }
    let mut extents = BTreeSet::new();
    for span in spans {
        if !extents.insert((
            span.document_id().as_uuid().as_u128(),
            span.byte_start(),
            span.byte_end(),
        )) {
            return Err(EventError::InvalidWirePayload);
        }
    }
    Ok(extents)
}

fn counted_rate(numerator: usize, denominator: usize) -> Result<f64, EventError> {
    let numerator = u32::try_from(numerator).map_err(|_| EventError::InvalidWirePayload)?;
    let denominator = u32::try_from(denominator).map_err(|_| EventError::InvalidWirePayload)?;
    if denominator == 0 {
        return Err(EventError::InvalidWirePayload);
    }
    Ok(f64::from(numerator) / f64::from(denominator))
}

#[cfg(test)]
mod tests {
    use super::{
        MentionEvidenceClocks, MentionReviewStatus, SpanGroundedMention, counted_rate,
        mention_span_precision, mention_span_recall, refuse_span_mention_as_instance,
        unique_extent_set,
    };
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

    fn clocks_at(available: &str, cutoff: &str) -> Result<MentionEvidenceClocks, EventError> {
        MentionEvidenceClocks::new(
            EventTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("event"),
            AssertionTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("assertion"),
            DocumentTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("document"),
            SystemTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("system"),
            AvailableTime::parse_rfc3339(available).expect("available"),
            KnowledgeCutoff::parse_rfc3339(cutoff).expect("cutoff"),
        )
    }

    #[test]
    fn local_helpers_cover_review_clock_and_rate_branches() {
        assert!(MentionReviewStatus::Reviewed.is_reviewed());
        assert!(!MentionReviewStatus::Proposed.is_reviewed());
        assert_eq!(
            MentionReviewStatus::from_wire_name("proposed").expect("parse"),
            MentionReviewStatus::Proposed
        );
        assert_eq!(MentionReviewStatus::Reviewed.wire_name(), "reviewed");
        let cutoff_equal =
            clocks_at("2026-03-31T00:00:00Z", "2026-03-31T00:00:00Z").expect("equal");
        assert_eq!(
            cutoff_equal.event_time(),
            EventTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("event")
        );
        assert_eq!(
            cutoff_equal.assertion_time(),
            AssertionTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("assertion")
        );
        assert_eq!(
            cutoff_equal.document_time(),
            DocumentTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("document")
        );
        assert_eq!(
            cutoff_equal.system_time(),
            SystemTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("system")
        );
        assert_eq!(
            cutoff_equal.knowledge_cutoff(),
            KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff")
        );
        let record = document("award filed");
        let mention = SpanGroundedMention::new(
            &record,
            span(&record, "award"),
            EventConfidence::certain().expect("certain"),
            cutoff_equal,
            "ace-extent-extractor/1",
            MentionReviewStatus::Reviewed,
        )
        .expect("mention");
        assert_eq!(
            refuse_span_mention_as_instance(&mention),
            Err(EventError::SpanMentionIsNotEventInstance)
        );
        let _ = mention.mention_id();
        assert_eq!(
            mention.evidence_layer(),
            EventEvidenceLayer::ObservedMention
        );
        let award = span(&record, "award");
        assert!(
            (mention_span_precision(&[award], &[award]).expect("p") - 1.0).abs() < f64::EPSILON
        );
        assert!((mention_span_recall(&[award], &[award]).expect("r") - 1.0).abs() < f64::EPSILON);
        assert_eq!(unique_extent_set(&[]), Err(EventError::InvalidWirePayload));
        assert_eq!(counted_rate(0, 0), Err(EventError::InvalidWirePayload));
        assert_eq!(
            counted_rate(usize::MAX, 1),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            counted_rate(1, usize::MAX),
            Err(EventError::InvalidWirePayload)
        );
        assert!((counted_rate(1, 2).expect("half") - 0.5).abs() < f64::EPSILON);
    }
}
