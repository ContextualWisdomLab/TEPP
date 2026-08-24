//! Evidence-status gates for TDT detection and CHRONOS prediction.

use crate::EventError;
use std::{collections::HashSet, hash::BuildHasher};

/// Epistemic layer of an event-intelligence output.
///
/// Only [`EventEvidenceLayer::PromotedTransition`] may enter the forward
/// state/input-process-outcome graph. TDT detections and CHRONOS predictions
/// remain measurement or hypothesis artifacts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EventEvidenceLayer {
    /// Fallible textual mention grounded in evidence.
    ObservedMention,
    /// TDT-style detection, link, or track output.
    TdtDetection,
    /// CHRONOS-style schema completion or predicted event.
    ChronosPrediction,
    /// Symbolic temporal-consistency judgment.
    TemporalConsistency,
    /// Independently promoted forward state transition.
    PromotedTransition,
}

impl EventEvidenceLayer {
    /// Stable wire name for this layer.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ObservedMention => "observed_mention",
            Self::TdtDetection => "tdt_detection",
            Self::ChronosPrediction => "chronos_prediction",
            Self::TemporalConsistency => "temporal_consistency",
            Self::PromotedTransition => "promoted_transition",
        }
    }

    /// Whether this layer may admit a forward state-transition edge.
    #[must_use]
    pub const fn may_admit_state_transition(self) -> bool {
        matches!(self, Self::PromotedTransition)
    }
}

/// Admit a layer into the forward state graph or fail closed.
///
/// # Errors
///
/// Returns [`EventError::PredictionIsNotFact`] for CHRONOS predictions and
/// [`EventError::DetectionIsNotTransition`] for every other non-promoted layer.
pub fn admit_state_transition(layer: EventEvidenceLayer) -> Result<(), EventError> {
    if layer.may_admit_state_transition() {
        Ok(())
    } else if matches!(layer, EventEvidenceLayer::ChronosPrediction) {
        Err(EventError::PredictionIsNotFact)
    } else {
        Err(EventError::DetectionIsNotTransition)
    }
}

/// First-story versus subsequent-track decision for one candidate story.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TdtStoryDecision {
    /// The story identity has not been seen in the stream.
    FirstStory,
    /// The story identity continues a previously seen event.
    Track,
}

/// Classify one candidate against previously seen story identities.
#[must_use]
pub fn classify_tdt_story<S: BuildHasher>(
    seen_story_ids: &HashSet<u64, S>,
    candidate_story_id: u64,
) -> TdtStoryDecision {
    if seen_story_ids.contains(&candidate_story_id) {
        TdtStoryDecision::Track
    } else {
        TdtStoryDecision::FirstStory
    }
}

/// Known-truth first-story detection counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirstStoryRates {
    hits: usize,
    misses: usize,
    false_alarms: usize,
    first_story_truth: usize,
    continuation_truth: usize,
}

impl FirstStoryRates {
    /// Correct first-story detections.
    #[must_use]
    pub const fn hits(self) -> usize {
        self.hits
    }

    /// Missed first stories.
    #[must_use]
    pub const fn misses(self) -> usize {
        self.misses
    }

    /// Continuations labeled as first stories.
    #[must_use]
    pub const fn false_alarms(self) -> usize {
        self.false_alarms
    }

    /// Miss rate among true first stories.
    #[must_use]
    pub fn miss_rate(self) -> f64 {
        if self.first_story_truth == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                self.misses as f64 / self.first_story_truth as f64
            }
        }
    }

    /// False-alarm rate among true continuations.
    #[must_use]
    pub fn false_alarm_rate(self) -> f64 {
        if self.continuation_truth == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                self.false_alarms as f64 / self.continuation_truth as f64
            }
        }
    }
}

/// Score a first-story detector against a known binary stream.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the streams are empty or
/// have unequal length.
pub fn first_story_detection_rates(
    truth_is_first: &[bool],
    predicted_is_first: &[bool],
) -> Result<FirstStoryRates, EventError> {
    if truth_is_first.is_empty() || truth_is_first.len() != predicted_is_first.len() {
        return Err(EventError::InvalidWirePayload);
    }
    let mut hits = 0;
    let mut misses = 0;
    let mut false_alarms = 0;
    let mut first_story_truth = 0;
    let mut continuation_truth = 0;
    for (&truth, &predicted) in truth_is_first.iter().zip(predicted_is_first) {
        if truth {
            first_story_truth += 1;
            if predicted {
                hits += 1;
            } else {
                misses += 1;
            }
        } else {
            continuation_truth += 1;
            if predicted {
                false_alarms += 1;
            }
        }
    }
    Ok(FirstStoryRates {
        hits,
        misses,
        false_alarms,
        first_story_truth,
        continuation_truth,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        EventEvidenceLayer, TdtStoryDecision, classify_tdt_story, first_story_detection_rates,
    };

    #[test]
    fn zero_denominator_rates_are_zero_and_track_is_not_first() {
        assert_eq!(
            classify_tdt_story(&HashSet::from([7]), 7),
            TdtStoryDecision::Track
        );
        assert_eq!(
            classify_tdt_story(&HashSet::new(), 8),
            TdtStoryDecision::FirstStory
        );
        assert!(first_story_detection_rates(&[], &[]).is_err());
        assert!(first_story_detection_rates(&[true], &[true, false]).is_err());
        let no_first_story = std::hint::black_box(
            first_story_detection_rates(&[false], &[false]).expect("no first"),
        );
        assert!(no_first_story.miss_rate() < 1e-15);
        let no_continuation =
            std::hint::black_box(first_story_detection_rates(&[true], &[true]).expect("no track"));
        assert!(no_continuation.false_alarm_rate() < 1e-15);
        let all_first = first_story_detection_rates(&[true, true], &[true, false]).expect("all");
        assert!((all_first.miss_rate() - 0.5).abs() < 1e-15);
        assert!(all_first.false_alarm_rate() < 1e-15);
        let continuations =
            first_story_detection_rates(&[false, false], &[true, false]).expect("continuations");
        assert_eq!(continuations.false_alarms(), 1);
        assert_eq!(continuations.misses(), 0);
        assert!(continuations.miss_rate() < 1e-15);
        assert!((continuations.false_alarm_rate() - 0.5).abs() < 1e-15);
        assert_eq!(
            EventEvidenceLayer::TdtDetection.wire_name(),
            "tdt_detection"
        );
    }
}
