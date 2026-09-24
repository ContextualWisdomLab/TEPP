//! Public contract for canonical deterministic reference-estimator seed manifests.

use topic_measurement::{ReferenceTopicModelConfig, TopicMeasurementError};

#[test]
fn reference_seed_manifest_requires_nonzero_unique_seeds_without_reordering() {
    assert!(ReferenceTopicModelConfig::new(2, vec![1, 11], 10, 1.0e-6).is_ok());
    assert!(ReferenceTopicModelConfig::new(2, vec![11, 1], 10, 1.0e-6).is_ok());

    for seeds in [vec![0, 11], vec![7, 11, 7]] {
        assert_eq!(
            ReferenceTopicModelConfig::new(2, seeds, 10, 1.0e-6),
            Err(TopicMeasurementError::InvalidModelInput)
        );
    }
}
