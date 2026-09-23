//! Recovery fitting must consume the noisy observed transition channel, not latent truth.

use tepp_simulation::{
    DocumentMethodEffect, ObservedRelation, SimulatedRelationKind, SimulationConfig, TruthManifest,
    generate,
};
use uuid::Uuid;

#[test]
fn observed_transition_projection_respects_false_negatives_and_ignores_reference_noise() {
    let all_transitions_missing = generate(
        SimulationConfig::new(2041, 3, 1, 2, 0, 0, 0, 10_000, 10_000, 0, 0, 0)
            .expect("simulation config"),
    )
    .expect("generated truth");

    assert!(
        all_transitions_missing
            .true_relations()
            .iter()
            .any(|relation| relation.kind().is_transition())
    );
    assert!(
        all_transitions_missing
            .observed_relations()
            .iter()
            .all(|relation| !relation.kind().is_transition())
    );
    assert!(
        all_transitions_missing
            .observed_relations()
            .iter()
            .any(|relation| relation.kind() == SimulatedRelationKind::References)
    );
    assert!(
        all_transitions_missing
            .observed_document_transition_pairs()
            .expect("observed transition projection")
            .is_empty()
    );

    let fully_observed = generate(
        SimulationConfig::new(2042, 3, 1, 2, 0, 0, 0, 0, 10_000, 0, 0, 0)
            .expect("simulation config"),
    )
    .expect("generated truth");
    let projected = fully_observed
        .observed_document_transition_pairs()
        .expect("observed transition projection");

    assert_eq!(
        projected,
        fully_observed
            .document_transition_pairs()
            .expect("latent truth projection")
    );
    assert!(!projected.is_empty());
    assert!(projected.iter().all(|(source, target)| {
        let source = fully_observed
            .documents()
            .iter()
            .find(|document| document.document_id() == *source)
            .expect("source document");
        let target = fully_observed
            .documents()
            .iter()
            .find(|document| document.document_id() == *target)
            .expect("target document");
        source.method_effect() == DocumentMethodEffect::Original
            && target.method_effect() == DocumentMethodEffect::Original
    }));
}

#[test]
fn observed_transition_projection_does_not_use_truth_marker_as_an_oracle_filter() {
    let generated = generate(
        SimulationConfig::new(2043, 3, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0)
            .expect("simulation config"),
    )
    .expect("generated truth");
    let transition = generated
        .true_relations()
        .iter()
        .find(|relation| relation.kind().is_transition())
        .expect("true transition");
    let false_marked_observation = ObservedRelation::new(
        Uuid::from_u128(0x699),
        SimulatedRelationKind::TransitionsTo,
        transition.source_id(),
        transition.target_id(),
        false,
    );
    let observed_only = TruthManifest::new(
        generated.seed(),
        generated.config_digest().to_owned(),
        generated.events().to_vec(),
        generated.documents().to_vec(),
        generated.true_relations().to_vec(),
        vec![false_marked_observation],
        generated.topic_truth().clone(),
    );

    let projected = observed_only
        .observed_document_transition_pairs()
        .expect("observed transition projection");
    let latent = observed_only
        .document_transition_pairs()
        .expect("latent truth projection");

    assert_eq!(projected.len(), 1);
    assert!(latent.contains(&projected[0]));
}
