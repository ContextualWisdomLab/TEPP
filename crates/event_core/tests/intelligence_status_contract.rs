//! TDT/CHRONOS outputs cannot become transitions or historical facts.

use event_core::{
    EventError, EventEvidenceLayer, TdtStoryDecision, admit_state_transition, classify_tdt_story,
    first_story_detection_rates,
};
use std::collections::HashSet;

#[test]
fn only_promoted_transitions_enter_the_state_graph() {
    admit_state_transition(EventEvidenceLayer::PromotedTransition).expect("promoted");
    assert!(EventEvidenceLayer::PromotedTransition.may_admit_state_transition());

    assert_eq!(
        admit_state_transition(EventEvidenceLayer::TdtDetection),
        Err(EventError::DetectionIsNotTransition)
    );
    assert_eq!(
        admit_state_transition(EventEvidenceLayer::ObservedMention),
        Err(EventError::DetectionIsNotTransition)
    );
    assert_eq!(
        admit_state_transition(EventEvidenceLayer::ChronosPrediction),
        Err(EventError::PredictionIsNotFact)
    );
    assert_eq!(
        admit_state_transition(EventEvidenceLayer::TemporalConsistency),
        Err(EventError::DetectionIsNotTransition)
    );

    for layer in [
        EventEvidenceLayer::ObservedMention,
        EventEvidenceLayer::TdtDetection,
        EventEvidenceLayer::ChronosPrediction,
        EventEvidenceLayer::TemporalConsistency,
        EventEvidenceLayer::PromotedTransition,
    ] {
        assert!(!layer.wire_name().is_empty());
    }
}

#[test]
fn first_story_detector_recovers_known_stream_with_computed_rates() {
    // Appearance order of news stories: first occurrence is a first story.
    let stream = [10_u64, 20, 10, 30, 20];
    let mut seen = HashSet::new();
    let mut predicted = Vec::new();
    let mut truth = Vec::new();
    for story in stream {
        let decision = classify_tdt_story(&seen, story);
        predicted.push(matches!(decision, TdtStoryDecision::FirstStory));
        truth.push(!seen.contains(&story));
        seen.insert(story);
    }
    let rates = first_story_detection_rates(&truth, &predicted).expect("rates");
    assert_eq!(rates.hits(), 3);
    assert_eq!(rates.misses(), 0);
    assert_eq!(rates.false_alarms(), 0);
    assert!(rates.miss_rate() < 1e-15);
    assert!(rates.false_alarm_rate() < 1e-15);

    let always_first = [true, true, true, true, true];
    let noisy = first_story_detection_rates(&truth, &always_first).expect("noisy");
    assert_eq!(noisy.false_alarms(), 2);
    let expected_fa = 2.0 / 2.0;
    assert!((noisy.false_alarm_rate() - expected_fa).abs() < 1e-15);
    assert_eq!(noisy.misses(), 0);
}

#[test]
fn first_story_rates_fail_closed_on_empty_or_mismatched_streams() {
    assert_eq!(
        first_story_detection_rates(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        first_story_detection_rates(&[true], &[true, false]),
        Err(EventError::InvalidWirePayload)
    );
}
