//! First-story detections are not instances; FAR/miss are computed from truth.

use event_core::{
    EventConfidence, EventError, EventMention, EventMentionId, FirstStoryLabel,
    MentionEvidenceClocks, MentionReviewStatus, decide_first_story, first_story_false_alarm_rate,
    first_story_miss_rate, refuse_first_story_as_instance,
};
use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
};

fn grounded_mention_id() -> EventMentionId {
    let text = "first story onset";
    let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
    let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
    let span =
        SourceSpan::new(&document, 0, text.len(), 0, text.chars().count(), None).expect("span");
    let clocks = MentionEvidenceClocks::new(
        EventTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("event"),
        AssertionTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("assertion"),
        DocumentTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("document"),
        SystemTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("system"),
        AvailableTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("available"),
        KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff"),
    )
    .expect("clocks");
    EventMention::new(
        &document,
        span,
        EventConfidence::new(0.8).expect("confidence"),
        clocks,
        "ace-extent-extractor/1",
        MentionReviewStatus::Proposed,
    )
    .expect("grounded mention")
    .mention_id()
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

fn decide_all(scores: &[f64], threshold: f64) -> Vec<FirstStoryLabel> {
    let cut = EventConfidence::new(threshold).expect("threshold");
    scores
        .iter()
        .map(|score| decide_first_story(EventConfidence::new(*score).expect("score"), cut))
        .collect()
}

#[test]
fn first_story_detection_cannot_be_cast_to_an_instance() {
    assert_eq!(
        refuse_first_story_as_instance(grounded_mention_id()),
        Err(EventError::FirstStoryIsNotEventInstance)
    );
}

#[test]
fn false_alarm_and_miss_rates_are_computed_from_known_truth() {
    let truth = [
        FirstStoryLabel::FirstStory,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FirstStory,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FollowUp,
    ];
    let calibrated = decide_all(&[0.90, 0.10, 0.15, 0.85, 0.20, 0.05], 0.50);
    let always_first = decide_all(&[1.0, 1.0, 1.0, 1.0, 1.0, 1.0], 0.50);

    let calibrated_far = first_story_false_alarm_rate(&truth, &calibrated).expect("far");
    let naive_far = first_story_false_alarm_rate(&truth, &always_first).expect("naive far");
    let calibrated_miss = first_story_miss_rate(&truth, &calibrated).expect("miss");
    let naive_miss = first_story_miss_rate(&truth, &always_first).expect("naive miss");

    assert!(
        calibrated_far < naive_far,
        "computed FAR {calibrated_far} must be below always-first FAR {naive_far}"
    );
    assert!(calibrated_miss <= naive_miss);
    assert!(
        calibrated_far.abs() < 1e-15 && calibrated_miss.abs() < 1e-15,
        "calibrated stream must recover FAR 0 and miss 0; far={calibrated_far} miss={calibrated_miss}"
    );
}

#[test]
fn mixed_detection_errors_recover_half_far_and_half_miss() {
    let truth = [
        FirstStoryLabel::FirstStory,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FirstStory,
    ];
    let decided = [
        FirstStoryLabel::FirstStory,
        FirstStoryLabel::FirstStory,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FollowUp,
    ];
    let far = first_story_false_alarm_rate(&truth, &decided).expect("far");
    let miss = first_story_miss_rate(&truth, &decided).expect("miss");
    let recovered = [far, miss];
    let truth_rates = [0.5_f64, 0.5];
    let rmse = computed_rmse(&truth_rates, &recovered);
    assert!(
        rmse < 1e-15,
        "known-truth FAR/miss RMSE {rmse} (far={far} miss={miss})"
    );
}

#[test]
fn calibrated_first_story_scores_have_lower_rmse_than_always_first() {
    let truth_labels = [
        FirstStoryLabel::FirstStory,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FirstStory,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FollowUp,
    ];
    let truth: Vec<f64> = truth_labels
        .iter()
        .copied()
        .map(FirstStoryLabel::as_probability_target)
        .collect();
    let calibrated = [0.90_f64, 0.10, 0.15, 0.85, 0.20, 0.05];
    let always_first = [1.0_f64, 1.0, 1.0, 1.0, 1.0, 1.0];
    let calibrated_rmse = computed_rmse(&truth, &calibrated);
    let naive_rmse = computed_rmse(&truth, &always_first);
    assert!(
        calibrated_rmse < naive_rmse,
        "computed calibrated RMSE {calibrated_rmse} must be below always-first RMSE {naive_rmse}"
    );
}

#[test]
fn rate_helpers_fail_closed_on_empty_mismatch_and_missing_class() {
    let first = [FirstStoryLabel::FirstStory];
    let follow = [FirstStoryLabel::FollowUp];
    assert_eq!(
        first_story_false_alarm_rate(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        first_story_miss_rate(&first, &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        first_story_false_alarm_rate(&first, &first),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        first_story_miss_rate(&follow, &follow),
        Err(EventError::InvalidWirePayload)
    );
}

#[test]
fn labels_round_trip_and_threshold_is_inclusive() {
    assert_eq!(FirstStoryLabel::FirstStory.wire_name(), "first_story");
    assert_eq!(FirstStoryLabel::FollowUp.wire_name(), "follow_up");
    assert_eq!(
        FirstStoryLabel::from_wire_name("first_story").expect("parse"),
        FirstStoryLabel::FirstStory
    );
    assert_eq!(
        FirstStoryLabel::from_wire_name("follow_up").expect("parse"),
        FirstStoryLabel::FollowUp
    );
    assert_eq!(
        FirstStoryLabel::from_wire_name("maybe_new"),
        Err(EventError::UnknownFirstStoryLabel)
    );
    assert!(FirstStoryLabel::FirstStory.is_first_story());
    assert!(!FirstStoryLabel::FollowUp.is_first_story());
    assert!((FirstStoryLabel::FirstStory.as_probability_target() - 1.0).abs() < f64::EPSILON);
    assert!((FirstStoryLabel::FollowUp.as_probability_target() - 0.0).abs() < f64::EPSILON);

    let half = EventConfidence::new(0.5).expect("half");
    assert_eq!(decide_first_story(half, half), FirstStoryLabel::FirstStory);
    assert_eq!(
        decide_first_story(EventConfidence::new(0.49).expect("below"), half),
        FirstStoryLabel::FollowUp
    );
}
