//! First-story detection scores stay distinct from promoted instances.

use crate::{EventConfidence, EventError, EventInstanceId, EventMentionId};

/// TDT first-story versus follow-up label.
///
/// A first-story decision is detection evidence. It is never a promoted event
/// instance and cannot create a forward state transition by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstStoryLabel {
    /// The mention is scored as the onset of a new story.
    FirstStory,
    /// The mention is scored as a continuation of an earlier story.
    FollowUp,
}

impl FirstStoryLabel {
    /// Return the stable wire label name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::FirstStory => "first_story",
            Self::FollowUp => "follow_up",
        }
    }

    /// Parse a stable wire first-story label.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::UnknownFirstStoryLabel`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, EventError> {
        match name {
            "first_story" => Ok(Self::FirstStory),
            "follow_up" => Ok(Self::FollowUp),
            _ => Err(EventError::UnknownFirstStoryLabel),
        }
    }

    /// Return whether this label is a first-story detection.
    #[must_use]
    pub const fn is_first_story(self) -> bool {
        matches!(self, Self::FirstStory)
    }

    /// Return the binary probability target used for RMSE.
    ///
    /// First-story truth is `1.0`; follow-up truth is `0.0`.
    #[must_use]
    pub const fn as_probability_target(self) -> f64 {
        match self {
            Self::FirstStory => 1.0,
            Self::FollowUp => 0.0,
        }
    }
}

/// Threshold a first-story probability into a detection label.
///
/// The threshold is inclusive: `probability >= threshold` is a first story.
#[must_use]
pub fn decide_first_story(
    probability: EventConfidence,
    threshold: EventConfidence,
) -> FirstStoryLabel {
    if probability.value() >= threshold.value() {
        FirstStoryLabel::FirstStory
    } else {
        FirstStoryLabel::FollowUp
    }
}

/// Explicit refusal to treat a first-story detection as an event instance.
///
/// # Errors
///
/// Always returns [`EventError::FirstStoryIsNotEventInstance`].
pub fn refuse_first_story_as_instance(
    _mention_id: EventMentionId,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::FirstStoryIsNotEventInstance)
}

/// False-alarm rate: follow-ups labeled first story, over follow-up truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when lengths differ, either
/// slice is empty, or the truth stream contains no follow-up.
pub fn first_story_false_alarm_rate(
    truth: &[FirstStoryLabel],
    decided: &[FirstStoryLabel],
) -> Result<f64, EventError> {
    rate_over_class(
        truth,
        decided,
        FirstStoryLabel::FollowUp,
        FirstStoryLabel::FirstStory,
    )
}

/// Miss rate: first stories labeled follow-up, over first-story truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when lengths differ, either
/// slice is empty, or the truth stream contains no first story.
pub fn first_story_miss_rate(
    truth: &[FirstStoryLabel],
    decided: &[FirstStoryLabel],
) -> Result<f64, EventError> {
    rate_over_class(
        truth,
        decided,
        FirstStoryLabel::FirstStory,
        FirstStoryLabel::FollowUp,
    )
}

fn rate_over_class(
    truth: &[FirstStoryLabel],
    decided: &[FirstStoryLabel],
    class: FirstStoryLabel,
    error_label: FirstStoryLabel,
) -> Result<f64, EventError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(EventError::InvalidWirePayload);
    }
    let mut class_count = 0_u32;
    let mut error_count = 0_u32;
    for (truth_label, decided_label) in truth.iter().zip(decided) {
        if *truth_label == class {
            class_count += 1;
            if *decided_label == error_label {
                error_count += 1;
            }
        }
    }
    if class_count == 0 {
        return Err(EventError::InvalidWirePayload);
    }
    Ok(f64::from(error_count) / f64::from(class_count))
}

#[cfg(test)]
mod tests {
    use super::{
        FirstStoryLabel, decide_first_story, first_story_false_alarm_rate, first_story_miss_rate,
        refuse_first_story_as_instance,
    };
    use crate::{EventConfidence, EventError, EventMentionId};

    #[test]
    fn first_story_helpers_cover_local_branches() {
        let mention = EventMentionId::new();
        assert_eq!(
            refuse_first_story_as_instance(mention),
            Err(EventError::FirstStoryIsNotEventInstance)
        );
        let high = EventConfidence::new(0.8).expect("high");
        let low = EventConfidence::new(0.2).expect("low");
        assert_eq!(decide_first_story(high, low), FirstStoryLabel::FirstStory);
        assert_eq!(decide_first_story(low, high), FirstStoryLabel::FollowUp);
        let mixed_truth = [FirstStoryLabel::FirstStory, FirstStoryLabel::FollowUp];
        let mixed_decided = [FirstStoryLabel::FollowUp, FirstStoryLabel::FirstStory];
        assert!(
            (first_story_false_alarm_rate(&mixed_truth, &mixed_decided).expect("far") - 1.0).abs()
                < f64::EPSILON
        );
        assert!(
            (first_story_miss_rate(&mixed_truth, &mixed_decided).expect("miss") - 1.0).abs()
                < f64::EPSILON
        );
    }
}
