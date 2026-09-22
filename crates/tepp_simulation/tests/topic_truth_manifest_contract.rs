//! Integration contract for deterministic known-topic simulation truth.

use std::collections::BTreeSet;

use tepp_simulation::{SimulationConfig, TopicDgpConfig, generate};

#[test]
fn generated_manifest_owns_digest_bound_topic_truth() {
    let topic_dgp = TopicDgpConfig::new(3, 12, 96, 6_500, 800, 1_200, 400, 300)
        .expect("topic DGP");
    let config = SimulationConfig::ci_default(2026)
        .with_topic_dgp(topic_dgp)
        .expect("simulation config");

    let manifest = generate(config).expect("known-topic corpus");
    manifest.verify_invariants().expect("manifest invariants");
    let truth = manifest.topic_truth();

    assert_eq!(truth.true_k(), 3);
    assert_eq!(truth.vocabulary_size(), 12);
    assert_eq!(truth.document_length(), 96);
    assert_eq!(truth.topic_term_probabilities().len(), 3);
    assert_eq!(truth.prevalence_intercepts().len(), 2);
    assert_eq!(truth.prevalence_time_slopes().len(), 2);
    assert_eq!(truth.prevalence_covariance().len(), 2);

    for probabilities in truth.topic_term_probabilities() {
        assert_eq!(probabilities.len(), 12);
        let total: f64 = probabilities.iter().sum();
        assert!((total - 1.0).abs() < 1.0e-12);
        assert!(probabilities.iter().all(|value| *value > 0.0));
    }

    let document_ids: BTreeSet<_> = manifest
        .documents()
        .iter()
        .map(tepp_simulation::SimulatedDocument::document_id)
        .collect();
    let truth_ids: BTreeSet<_> = truth
        .document_states()
        .iter()
        .map(tepp_simulation::DocumentTopicTruth::document_id)
        .collect();
    assert_eq!(truth_ids, document_ids);

    for state in truth.document_states() {
        assert_eq!(state.logistic_normal_coordinates().len(), 2);
        assert_eq!(state.topic_mixture().len(), 3);
        assert_eq!(state.term_counts().len(), 12);
        assert_eq!(state.term_counts().iter().sum::<u32>(), 96);
        let mixture_total: f64 = state.topic_mixture().iter().sum();
        assert!((mixture_total - 1.0).abs() < 1.0e-12);
    }

    let changed = generate(
        SimulationConfig::ci_default(2026)
            .with_topic_dgp(
                TopicDgpConfig::new(3, 12, 96, 6_500, 1_600, 1_200, 400, 300)
                    .expect("changed topic DGP"),
            )
            .expect("changed simulation config"),
    )
    .expect("changed corpus");
    assert_ne!(manifest.config_digest(), changed.config_digest());
    assert_ne!(manifest.content_digest(), changed.content_digest());
}
