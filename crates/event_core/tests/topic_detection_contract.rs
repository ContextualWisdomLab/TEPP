//! Topic clusters are not instances; new-topic FAR/miss and pair recovery use truth.

use event_core::{
    EventConfidence, EventError, TopicClusterId, TopicDetectionLabel, decide_topic_detection,
    new_topic_false_alarm_rate, new_topic_miss_rate, refuse_topic_cluster_as_event_instance,
    topic_cluster_pair_precision, topic_cluster_pair_recall,
};

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

fn decide_all(scores: &[f64], threshold: f64) -> Vec<TopicDetectionLabel> {
    let cut = EventConfidence::new(threshold).expect("threshold");
    scores
        .iter()
        .map(|score| decide_topic_detection(EventConfidence::new(*score).expect("score"), cut))
        .collect()
}

#[test]
fn topic_cluster_cannot_be_cast_to_an_event_instance() {
    assert_eq!(
        refuse_topic_cluster_as_event_instance(TopicClusterId::from_raw(7)),
        Err(EventError::TopicClusterIsNotEventInstance)
    );
}

#[test]
fn new_topic_false_alarm_and_miss_rates_are_computed_from_known_truth() {
    let truth = [
        TopicDetectionLabel::NewTopic,
        TopicDetectionLabel::ExistingTopic,
        TopicDetectionLabel::ExistingTopic,
        TopicDetectionLabel::NewTopic,
        TopicDetectionLabel::ExistingTopic,
        TopicDetectionLabel::ExistingTopic,
    ];
    let calibrated = decide_all(&[0.90, 0.10, 0.15, 0.85, 0.20, 0.05], 0.50);
    let always_new = decide_all(&[1.0, 1.0, 1.0, 1.0, 1.0, 1.0], 0.50);

    let calibrated_far = new_topic_false_alarm_rate(&truth, &calibrated).expect("far");
    let naive_far = new_topic_false_alarm_rate(&truth, &always_new).expect("naive far");
    let calibrated_miss = new_topic_miss_rate(&truth, &calibrated).expect("miss");
    let naive_miss = new_topic_miss_rate(&truth, &always_new).expect("naive miss");

    assert!(
        calibrated_far < naive_far,
        "computed FAR {calibrated_far} must be below always-new FAR {naive_far}"
    );
    assert!(calibrated_miss <= naive_miss);
}

#[test]
fn calibrated_new_topic_scores_have_lower_rmse_than_always_new() {
    let truth_labels = [
        TopicDetectionLabel::NewTopic,
        TopicDetectionLabel::ExistingTopic,
        TopicDetectionLabel::ExistingTopic,
        TopicDetectionLabel::NewTopic,
        TopicDetectionLabel::ExistingTopic,
        TopicDetectionLabel::ExistingTopic,
    ];
    let truth: Vec<f64> = truth_labels
        .iter()
        .copied()
        .map(TopicDetectionLabel::as_probability_target)
        .collect();
    let calibrated = [0.90_f64, 0.10, 0.15, 0.85, 0.20, 0.05];
    let always_new = [1.0_f64, 1.0, 1.0, 1.0, 1.0, 1.0];
    let calibrated_rmse = computed_rmse(&truth, &calibrated);
    let naive_rmse = computed_rmse(&truth, &always_new);
    assert!(
        calibrated_rmse < naive_rmse,
        "computed calibrated RMSE {calibrated_rmse} must be below always-new RMSE {naive_rmse}"
    );
}

#[test]
fn pair_precision_and_recall_recover_known_topic_clusters() {
    let truth = [
        TopicClusterId::from_raw(1),
        TopicClusterId::from_raw(1),
        TopicClusterId::from_raw(2),
        TopicClusterId::from_raw(2),
        TopicClusterId::from_raw(3),
    ];
    let recovered = [
        TopicClusterId::from_raw(10),
        TopicClusterId::from_raw(10),
        TopicClusterId::from_raw(20),
        TopicClusterId::from_raw(20),
        TopicClusterId::from_raw(30),
    ];
    let collapsed = [
        TopicClusterId::from_raw(1),
        TopicClusterId::from_raw(1),
        TopicClusterId::from_raw(1),
        TopicClusterId::from_raw(1),
        TopicClusterId::from_raw(1),
    ];

    let recovered_precision = topic_cluster_pair_precision(&truth, &recovered).expect("p");
    let collapsed_precision = topic_cluster_pair_precision(&truth, &collapsed).expect("naive p");
    let recovered_recall = topic_cluster_pair_recall(&truth, &recovered).expect("r");

    assert!(
        (recovered_precision - 1.0).abs() < f64::EPSILON,
        "isomorphic cluster labels must have pair precision 1, got {recovered_precision}"
    );
    assert!((recovered_recall - 1.0).abs() < f64::EPSILON);
    assert!(
        recovered_precision > collapsed_precision,
        "computed pair precision {recovered_precision} must beat collapsed {collapsed_precision}"
    );
}

#[test]
fn rate_and_pair_helpers_fail_closed_on_empty_mismatch_and_missing_class() {
    let new_topic = [TopicDetectionLabel::NewTopic];
    let existing = [TopicDetectionLabel::ExistingTopic];
    assert_eq!(
        new_topic_false_alarm_rate(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        new_topic_miss_rate(&new_topic, &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        new_topic_false_alarm_rate(&new_topic, &new_topic),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        new_topic_miss_rate(&existing, &existing),
        Err(EventError::InvalidWirePayload)
    );

    let one = [TopicClusterId::from_raw(1)];
    let two = [TopicClusterId::from_raw(1), TopicClusterId::from_raw(2)];
    assert_eq!(
        topic_cluster_pair_precision(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        topic_cluster_pair_recall(&one, &two),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        topic_cluster_pair_precision(&two, &two),
        Err(EventError::InvalidWirePayload)
    );
}

#[test]
fn labels_round_trip_and_threshold_is_inclusive() {
    assert_eq!(TopicDetectionLabel::NewTopic.wire_name(), "new_topic");
    assert_eq!(
        TopicDetectionLabel::ExistingTopic.wire_name(),
        "existing_topic"
    );
    assert_eq!(
        TopicDetectionLabel::from_wire_name("new_topic").expect("parse"),
        TopicDetectionLabel::NewTopic
    );
    assert_eq!(
        TopicDetectionLabel::from_wire_name("existing_topic").expect("parse"),
        TopicDetectionLabel::ExistingTopic
    );
    assert_eq!(
        TopicDetectionLabel::from_wire_name("maybe_new"),
        Err(EventError::UnknownTopicDetectionLabel)
    );
    assert!(TopicDetectionLabel::NewTopic.is_new_topic());
    assert!(!TopicDetectionLabel::ExistingTopic.is_new_topic());
    assert!((TopicDetectionLabel::NewTopic.as_probability_target() - 1.0).abs() < f64::EPSILON);
    assert!(
        (TopicDetectionLabel::ExistingTopic.as_probability_target() - 0.0).abs() < f64::EPSILON
    );

    let half = EventConfidence::new(0.5).expect("half");
    assert_eq!(
        decide_topic_detection(half, half),
        TopicDetectionLabel::NewTopic
    );
    assert_eq!(
        decide_topic_detection(EventConfidence::new(0.49).expect("below"), half),
        TopicDetectionLabel::ExistingTopic
    );
    assert_eq!(TopicClusterId::from_raw(3).raw(), 3);
}
