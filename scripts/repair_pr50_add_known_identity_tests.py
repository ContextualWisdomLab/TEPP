"""Add the PR 50 known-identity baseline regression before implementation."""

from pathlib import Path


CONTRACT = r'''//! Event-intelligence status gates and an explicitly oracle-assisted baseline.

use event_core::{
    EventError, EventEvidenceLayer, FirstStoryRates, KnownIdentityStoryDecision,
    admit_state_transition, classify_known_identity_baseline, first_story_detection_rates,
};

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
fn known_identity_baseline_is_scored_against_an_independent_truth_fixture() {
    // This is an oracle-assisted identity baseline, not a raw-text first-story detector.
    let stream = [10_u64, 20, 10, 30, 20];
    let truth_is_first = [true, true, false, true, false];
    let mut seen = Vec::new();
    let mut predicted_is_first = Vec::new();

    for story_identity in stream {
        let decision = classify_known_identity_baseline(&seen, story_identity);
        predicted_is_first.push(matches!(
            decision,
            KnownIdentityStoryDecision::FirstOccurrence
        ));
        if matches!(decision, KnownIdentityStoryDecision::FirstOccurrence) {
            seen.push(story_identity);
        }
    }

    let rates = first_story_detection_rates(&truth_is_first, &predicted_is_first)
        .expect("independent truth fixture");
    assert_eq!(rates.hits(), 3);
    assert_eq!(rates.misses(), 0);
    assert_eq!(rates.false_alarms(), 0);
    assert!(rates.miss_rate() < 1e-15);
    assert!(rates.false_alarm_rate() < 1e-15);

    let always_first = [true, true, true, true, true];
    let false_alarm_rates = first_story_detection_rates(&truth_is_first, &always_first)
        .expect("false-alarm fixture");
    assert_eq!(false_alarm_rates.false_alarms(), 2);
    assert!((false_alarm_rates.false_alarm_rate() - 1.0).abs() < 1e-15);

    let always_continuation = [false, false, false, false, false];
    let miss_rates = first_story_detection_rates(&truth_is_first, &always_continuation)
        .expect("miss fixture");
    assert_eq!(miss_rates.misses(), 3);
    assert!((miss_rates.miss_rate() - 1.0).abs() < 1e-15);
    assert!(miss_rates.false_alarm_rate() < 1e-15);
}

#[test]
fn baseline_and_rate_contracts_fail_closed_at_their_boundaries() {
    assert_eq!(
        classify_known_identity_baseline(&[7], 7),
        KnownIdentityStoryDecision::RepeatedIdentity
    );
    assert_eq!(
        classify_known_identity_baseline(&[7], 8),
        KnownIdentityStoryDecision::FirstOccurrence
    );
    assert_eq!(
        first_story_detection_rates(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        first_story_detection_rates(&[true], &[true, false]),
        Err(EventError::InvalidWirePayload)
    );

    let empty_classes = FirstStoryRates::empty_for_test();
    assert!(empty_classes.miss_rate() < 1e-15);
    assert!(empty_classes.false_alarm_rate() < 1e-15);
}
'''

path = Path("crates/event_core/tests/intelligence_status_contract.rs")
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text(CONTRACT, encoding="utf-8")
