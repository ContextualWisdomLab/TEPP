//! Span-grounded mentions recover exact ACE extents against known truth.

use event_core::{
    EventConfidence, EventError, EventEvidenceLayer, MentionEvidenceClocks, MentionReviewStatus,
    SpanGroundedMention, mention_span_precision, mention_span_recall,
    refuse_span_mention_as_instance,
};
use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
};

const DOCUMENT_TEXT: &str = "The procurement office awarded the river-crossing contract on 1 March 2026 after the earlier protest was withdrawn.";

fn documentary_record() -> DocumentRecord {
    let artifact = SourceArtifact::from_bytes(DOCUMENT_TEXT.as_bytes()).expect("artifact");
    DocumentRecord::from_text(artifact.id(), DOCUMENT_TEXT).expect("document")
}

fn span_for(document: &DocumentRecord, surface: &str) -> SourceSpan {
    let byte_start = document.text().find(surface).expect("surface present");
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

fn eligible_clocks() -> MentionEvidenceClocks {
    MentionEvidenceClocks::new(
        EventTime::parse_rfc3339("2026-03-01T12:00:00Z").expect("event"),
        AssertionTime::parse_rfc3339("2026-03-02T09:00:00Z").expect("assertion"),
        DocumentTime::parse_rfc3339("2026-03-02T08:00:00Z").expect("document"),
        SystemTime::parse_rfc3339("2026-03-03T00:00:00Z").expect("system"),
        AvailableTime::parse_rfc3339("2026-03-03T00:00:00Z").expect("available"),
        KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff"),
    )
    .expect("eligible clocks")
}

fn grounded(
    document: &DocumentRecord,
    surface: &str,
    review: MentionReviewStatus,
) -> SpanGroundedMention {
    SpanGroundedMention::new(
        document,
        span_for(document, surface),
        EventConfidence::new(0.91).expect("confidence"),
        eligible_clocks(),
        "ace-extent-extractor/1",
        review,
    )
    .expect("grounded mention")
}

fn occupancy(truth: &[SourceSpan], recovered: &[SourceSpan]) -> Vec<f64> {
    truth
        .iter()
        .map(|span| {
            if recovered.iter().any(|candidate| {
                candidate.document_id() == span.document_id()
                    && candidate.byte_start() == span.byte_start()
                    && candidate.byte_end() == span.byte_end()
            }) {
                1.0
            } else {
                0.0
            }
        })
        .collect()
}

fn computed_rmse(truth: &[f64], recovered: &[f64]) -> f64 {
    assert_eq!(truth.len(), recovered.len());
    let n = f64::from(u32::try_from(truth.len()).expect("tiny fixture"));
    let sse: f64 = truth
        .iter()
        .zip(recovered)
        .map(|(truth_value, recovered_value)| {
            let residual = truth_value - recovered_value;
            residual * residual
        })
        .sum();
    (sse / n).sqrt()
}

#[test]
fn span_mention_cannot_be_cast_to_an_instance() {
    let document = documentary_record();
    let mention = grounded(
        &document,
        "awarded the river-crossing contract",
        MentionReviewStatus::Reviewed,
    );
    assert_eq!(
        refuse_span_mention_as_instance(&mention),
        Err(EventError::SpanMentionIsNotEventInstance)
    );
    assert_eq!(
        mention.evidence_layer(),
        EventEvidenceLayer::ObservedMention
    );
}

#[test]
fn exact_extractor_recovers_known_truth_extents_better_than_whole_document() {
    let document = documentary_record();
    let award = span_for(&document, "awarded the river-crossing contract");
    let protest = span_for(&document, "protest");
    let truth = [award, protest];
    let exact = [award, protest];
    let whole_document = [span_for(&document, DOCUMENT_TEXT)];
    let shifted = [
        span_for(&document, "awarded the river-crossing"),
        span_for(&document, "protest was"),
    ];

    let exact_precision = mention_span_precision(&truth, &exact).expect("exact p");
    let exact_recall = mention_span_recall(&truth, &exact).expect("exact r");
    let naive_precision = mention_span_precision(&truth, &whole_document).expect("naive p");
    let naive_recall = mention_span_recall(&truth, &whole_document).expect("naive r");
    let shifted_precision = mention_span_precision(&truth, &shifted).expect("shifted p");
    let shifted_recall = mention_span_recall(&truth, &shifted).expect("shifted r");

    assert!((exact_precision - 1.0).abs() < f64::EPSILON);
    assert!((exact_recall - 1.0).abs() < f64::EPSILON);
    assert!(naive_precision.abs() < f64::EPSILON);
    assert!(naive_recall.abs() < f64::EPSILON);
    assert!(shifted_precision.abs() < f64::EPSILON);
    assert!(shifted_recall.abs() < f64::EPSILON);

    let ones = [1.0, 1.0];
    let exact_rmse = computed_rmse(&ones, &occupancy(&truth, &exact));
    let naive_rmse = computed_rmse(&ones, &occupancy(&truth, &whole_document));
    assert!(exact_rmse.abs() < 1e-15);
    assert!((naive_rmse - 1.0).abs() < 1e-15);
    assert!(exact_rmse < naive_rmse);
}

#[test]
fn grounded_mention_keeps_exact_surface_document_clocks_and_review_status() {
    let document = documentary_record();
    let mention = grounded(
        &document,
        "awarded the river-crossing contract",
        MentionReviewStatus::Proposed,
    );
    assert_eq!(
        mention.surface_form(),
        "awarded the river-crossing contract"
    );
    assert_eq!(mention.document_id(), document.id());
    assert_eq!(mention.evidence_id(), document.id());
    assert_eq!(
        mention.source_span().byte_start(),
        document
            .text()
            .find("awarded the river-crossing contract")
            .expect("offset")
    );
    assert_eq!(mention.extractor_version(), "ace-extent-extractor/1");
    assert_eq!(mention.review_status(), MentionReviewStatus::Proposed);
    assert!((mention.confidence().value() - 0.91).abs() < f64::EPSILON);
    assert_eq!(
        mention.clocks().available_time(),
        AvailableTime::parse_rfc3339("2026-03-03T00:00:00Z").expect("available")
    );
    assert_eq!(
        MentionReviewStatus::from_wire_name("reviewed").expect("parse"),
        MentionReviewStatus::Reviewed
    );
    assert_eq!(MentionReviewStatus::Proposed.wire_name(), "proposed");
}

#[test]
fn delayed_reporting_before_cutoff_is_kept_and_late_availability_fails_closed() {
    let document = documentary_record();
    let span = span_for(&document, "protest");
    let late = MentionEvidenceClocks::new(
        EventTime::parse_rfc3339("2026-03-01T12:00:00Z").expect("event"),
        AssertionTime::parse_rfc3339("2026-04-02T09:00:00Z").expect("assertion"),
        DocumentTime::parse_rfc3339("2026-04-02T08:00:00Z").expect("document"),
        SystemTime::parse_rfc3339("2026-04-03T00:00:00Z").expect("system"),
        AvailableTime::parse_rfc3339("2026-04-03T00:00:00Z").expect("available"),
        KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff"),
    );
    assert_eq!(late, Err(EventError::MentionIneligibleAtCutoff));

    let delayed = MentionEvidenceClocks::new(
        EventTime::parse_rfc3339("2026-03-01T12:00:00Z").expect("event"),
        AssertionTime::parse_rfc3339("2026-03-15T09:00:00Z").expect("assertion"),
        DocumentTime::parse_rfc3339("2026-03-15T08:00:00Z").expect("document"),
        SystemTime::parse_rfc3339("2026-03-16T00:00:00Z").expect("system"),
        AvailableTime::parse_rfc3339("2026-03-16T00:00:00Z").expect("available"),
        KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff"),
    )
    .expect("delayed but eligible");
    let mention = SpanGroundedMention::new(
        &document,
        span,
        EventConfidence::certain().expect("certain"),
        delayed,
        "ace-extent-extractor/1",
        MentionReviewStatus::Proposed,
    )
    .expect("delayed mention");
    assert_eq!(mention.surface_form(), "protest");
}

#[test]
fn empty_extractor_foreign_span_and_empty_or_duplicate_extent_sets_fail_closed() {
    let document = documentary_record();
    let other = documentary_record();
    let span = span_for(&document, "protest");
    assert_eq!(
        SpanGroundedMention::new(
            &document,
            span,
            EventConfidence::certain().expect("certain"),
            eligible_clocks(),
            "   ",
            MentionReviewStatus::Proposed,
        )
        .map(|_| ()),
        Err(EventError::EmptyExtractorVersion)
    );
    assert_eq!(
        SpanGroundedMention::new(
            &other,
            span,
            EventConfidence::certain().expect("certain"),
            eligible_clocks(),
            "ace-extent-extractor/1",
            MentionReviewStatus::Proposed,
        )
        .map(|_| ()),
        Err(EventError::MentionSpanDocumentMismatch)
    );
    assert_eq!(
        mention_span_precision(&[], &[span]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        mention_span_recall(&[span], &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        mention_span_precision(&[span, span], &[span]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        MentionReviewStatus::from_wire_name("promoted"),
        Err(EventError::UnknownMentionReviewStatus)
    );
}
