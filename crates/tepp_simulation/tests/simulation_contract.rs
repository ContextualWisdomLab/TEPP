//! Integration contracts for deterministic truth simulation.

use tepp_simulation::{
    DocumentMethodEffect, SimulationConfig, generate, validate_missingness_rate_bps,
};

#[test]
fn deterministic_seed_and_truth_manifest_contracts() {
    let config = SimulationConfig::ci_default(2026);
    let first = generate(config).expect("first");
    let second = generate(config).expect("second");
    assert_eq!(first.content_digest(), second.content_digest());
    first.verify_invariants().expect("invariants");

    assert!(first.event_count() >= 2);
    assert!(first.document_count() >= first.event_count());
    assert!(!first.true_relations().is_empty());

    let mut saw_original = false;
    let mut saw_derivative = false;
    for document in first.documents() {
        if document.method_effect() == DocumentMethodEffect::Original {
            saw_original = true;
            assert!(document.parent_document_id().is_none());
        } else {
            saw_derivative = true;
            assert!(document.parent_document_id().is_some());
        }
        assert!(document.memberships().len() >= 2);
        assert!(document.available_time().instant() >= document.document_time().instant());
    }
    assert!(saw_original);
    assert!(saw_derivative);

    // Event occurrence is separate from document/availability clocks: every
    // original report is at or after the latent event time for retrospective
    // reporting scenarios.
    for document in first.documents() {
        let event = first
            .events()
            .iter()
            .find(|event| event.event_id() == document.event_id())
            .expect("event");
        assert!(document.document_time().instant() >= event.event_time().instant());
    }
}

#[test]
fn missingness_and_relation_noise_are_parameterized() {
    validate_missingness_rate_bps(2_500).expect("rate");
    let noisy = SimulationConfig::new(
        99, 3, 1, 3, 12, 6, 10_000, 10_000, 10_000, 10_000, 10_000, 10_000,
    )
    .expect("noisy");
    let manifest = generate(noisy).expect("generate");
    assert!(
        manifest
            .documents()
            .iter()
            .all(|document| document.observed_event_time().is_none())
    );
    // Full false-negative rate yields no true-positive observations.
    assert!(
        manifest
            .observed_relations()
            .iter()
            .all(|relation| !relation.is_true_positive())
    );
}
