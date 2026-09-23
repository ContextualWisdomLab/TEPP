//! Recovery fitting must consume the noisy observed transition channel, not latent truth.

use tepp_simulation::{DocumentMethodEffect, SimulatedRelationKind, SimulationConfig, generate};

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
