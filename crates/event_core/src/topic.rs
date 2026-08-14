//! Topic-detection scores stay distinct from promoted event instances.

use crate::{EventConfidence, EventError, EventInstanceId};

/// Opaque TDT topic-cluster identity.
///
/// Cluster identifiers label unsupervised topic assignments. They are never
/// event-instance identifiers and cannot create a state transition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TopicClusterId(u64);

impl TopicClusterId {
    /// Construct a topic-cluster identity from a raw numeric label.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Return the raw numeric cluster label.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// TDT new-topic versus existing-topic label.
///
/// Topic detection clusters stories and flags previously unseen topics. That
/// decision is detection evidence. It is not a first-story onset, a promoted
/// event instance, or a forward state transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopicDetectionLabel {
    /// The story opens a previously unseen topic cluster.
    NewTopic,
    /// The story is assigned to a topic already observed in the stream.
    ExistingTopic,
}

impl TopicDetectionLabel {
    /// Return the stable wire label name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::NewTopic => "new_topic",
            Self::ExistingTopic => "existing_topic",
        }
    }

    /// Parse a stable wire topic-detection label.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::UnknownTopicDetectionLabel`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, EventError> {
        match name {
            "new_topic" => Ok(Self::NewTopic),
            "existing_topic" => Ok(Self::ExistingTopic),
            _ => Err(EventError::UnknownTopicDetectionLabel),
        }
    }

    /// Return whether this label is a new-topic detection.
    #[must_use]
    pub const fn is_new_topic(self) -> bool {
        matches!(self, Self::NewTopic)
    }

    /// Return the binary probability target used for RMSE.
    ///
    /// New-topic truth is `1.0`; existing-topic truth is `0.0`.
    #[must_use]
    pub const fn as_probability_target(self) -> f64 {
        match self {
            Self::NewTopic => 1.0,
            Self::ExistingTopic => 0.0,
        }
    }
}

/// Threshold a new-topic probability into a detection label.
///
/// The threshold is inclusive: `probability >= threshold` is a new topic.
#[must_use]
pub fn decide_topic_detection(
    probability: EventConfidence,
    threshold: EventConfidence,
) -> TopicDetectionLabel {
    if probability.value() >= threshold.value() {
        TopicDetectionLabel::NewTopic
    } else {
        TopicDetectionLabel::ExistingTopic
    }
}

/// Explicit refusal to treat a topic cluster as an event instance.
///
/// # Errors
///
/// Always returns [`EventError::TopicClusterIsNotEventInstance`].
pub fn refuse_topic_cluster_as_event_instance(
    _cluster_id: TopicClusterId,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::TopicClusterIsNotEventInstance)
}

/// False-alarm rate: existing topics labeled new, over existing-topic truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when lengths differ, either
/// slice is empty, or the truth stream contains no existing topic.
pub fn new_topic_false_alarm_rate(
    truth: &[TopicDetectionLabel],
    decided: &[TopicDetectionLabel],
) -> Result<f64, EventError> {
    rate_over_class(
        truth,
        decided,
        TopicDetectionLabel::ExistingTopic,
        TopicDetectionLabel::NewTopic,
    )
}

/// Miss rate: new topics labeled existing, over new-topic truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when lengths differ, either
/// slice is empty, or the truth stream contains no new topic.
pub fn new_topic_miss_rate(
    truth: &[TopicDetectionLabel],
    decided: &[TopicDetectionLabel],
) -> Result<f64, EventError> {
    rate_over_class(
        truth,
        decided,
        TopicDetectionLabel::NewTopic,
        TopicDetectionLabel::ExistingTopic,
    )
}

/// Pair precision of recovered co-topic assignments against known clusters.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when lengths differ, either
/// slice is empty, or the recovered stream contains no co-topic pair.
pub fn topic_cluster_pair_precision(
    truth: &[TopicClusterId],
    recovered: &[TopicClusterId],
) -> Result<f64, EventError> {
    let (true_positive, _truth_pairs, recovered_pairs) = pair_counts(truth, recovered)?;
    if recovered_pairs == 0 {
        return Err(EventError::InvalidWirePayload);
    }
    Ok(f64::from(true_positive) / f64::from(recovered_pairs))
}

/// Pair recall of recovered co-topic assignments against known clusters.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when lengths differ, either
/// slice is empty, or the truth stream contains no co-topic pair.
pub fn topic_cluster_pair_recall(
    truth: &[TopicClusterId],
    recovered: &[TopicClusterId],
) -> Result<f64, EventError> {
    let (true_positive, truth_pairs, _recovered_pairs) = pair_counts(truth, recovered)?;
    if truth_pairs == 0 {
        return Err(EventError::InvalidWirePayload);
    }
    Ok(f64::from(true_positive) / f64::from(truth_pairs))
}

fn rate_over_class(
    truth: &[TopicDetectionLabel],
    decided: &[TopicDetectionLabel],
    class: TopicDetectionLabel,
    error_label: TopicDetectionLabel,
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

fn pair_counts(
    truth: &[TopicClusterId],
    recovered: &[TopicClusterId],
) -> Result<(u32, u32, u32), EventError> {
    if truth.is_empty() || truth.len() != recovered.len() {
        return Err(EventError::InvalidWirePayload);
    }
    let mut truth_pairs = 0_u32;
    let mut recovered_pairs = 0_u32;
    let mut true_positive = 0_u32;
    for left in 0..truth.len() {
        for right in (left + 1)..truth.len() {
            let truth_pair = truth[left] == truth[right];
            let recovered_pair = recovered[left] == recovered[right];
            if truth_pair {
                truth_pairs += 1;
            }
            if recovered_pair {
                recovered_pairs += 1;
            }
            if truth_pair && recovered_pair {
                true_positive += 1;
            }
        }
    }
    Ok((true_positive, truth_pairs, recovered_pairs))
}

#[cfg(test)]
mod tests {
    use super::{
        TopicClusterId, TopicDetectionLabel, decide_topic_detection, new_topic_false_alarm_rate,
        new_topic_miss_rate, refuse_topic_cluster_as_event_instance, topic_cluster_pair_precision,
        topic_cluster_pair_recall,
    };
    use crate::{EventConfidence, EventError};

    #[test]
    fn topic_detection_helpers_cover_local_branches() {
        let cluster = TopicClusterId::from_raw(4);
        assert_eq!(cluster.raw(), 4);
        assert_eq!(
            refuse_topic_cluster_as_event_instance(cluster),
            Err(EventError::TopicClusterIsNotEventInstance)
        );
        let high = EventConfidence::new(0.8).expect("high");
        let low = EventConfidence::new(0.2).expect("low");
        assert_eq!(
            decide_topic_detection(high, low),
            TopicDetectionLabel::NewTopic
        );
        assert_eq!(
            decide_topic_detection(low, high),
            TopicDetectionLabel::ExistingTopic
        );
        let mixed_truth = [
            TopicDetectionLabel::NewTopic,
            TopicDetectionLabel::ExistingTopic,
        ];
        let mixed_decided = [
            TopicDetectionLabel::ExistingTopic,
            TopicDetectionLabel::NewTopic,
        ];
        assert!(
            (new_topic_false_alarm_rate(&mixed_truth, &mixed_decided).expect("far") - 1.0).abs()
                < f64::EPSILON
        );
        assert!(
            (new_topic_miss_rate(&mixed_truth, &mixed_decided).expect("miss") - 1.0).abs()
                < f64::EPSILON
        );

        let truth = [
            TopicClusterId::from_raw(1),
            TopicClusterId::from_raw(1),
            TopicClusterId::from_raw(2),
        ];
        let disjoint = [
            TopicClusterId::from_raw(1),
            TopicClusterId::from_raw(2),
            TopicClusterId::from_raw(2),
        ];
        assert!(
            (topic_cluster_pair_precision(&truth, &disjoint).expect("precision") - 0.0).abs()
                < f64::EPSILON
        );
        assert!(
            (topic_cluster_pair_recall(&truth, &disjoint).expect("recall") - 0.0).abs()
                < f64::EPSILON
        );
        let split = [
            TopicClusterId::from_raw(1),
            TopicClusterId::from_raw(3),
            TopicClusterId::from_raw(2),
        ];
        assert_eq!(
            topic_cluster_pair_precision(&truth, &split),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            topic_cluster_pair_recall(
                &[TopicClusterId::from_raw(1), TopicClusterId::from_raw(2)],
                &[TopicClusterId::from_raw(1), TopicClusterId::from_raw(1)],
            ),
            Err(EventError::InvalidWirePayload)
        );
    }
}
