//! Event-transition projection must not promote derivative document variants.

use std::collections::BTreeMap;

use tepp_simulation::{
    DocumentMethodEffect, SimulatedDocument, SimulationConfig, SimulationError, TruthManifest,
    generate,
};
use uuid::Uuid;

fn manifest_with_documents(generated: &TruthManifest, documents: Vec<SimulatedDocument>) -> TruthManifest {
    TruthManifest::new(
        generated.seed(),
        generated.config_digest().to_owned(),
        generated.events().to_vec(),
        documents,
        generated.true_relations().to_vec(),
        generated.observed_relations().to_vec(),
        generated.topic_truth().clone(),
    )
}

#[test]
fn transition_projection_uses_originals_and_refuses_derivative_only_events() {
    let generated = generate(
        SimulationConfig::new(2030, 2, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0)
            .expect("simulation config"),
    )
    .expect("generated truth");
    let source_event = generated.events()[0].event_id();
    let source_original = generated
        .documents()
        .iter()
        .find(|document| document.event_id() == source_event)
        .expect("source original");
    assert_eq!(source_original.method_effect(), DocumentMethodEffect::Original);

    let derivative = SimulatedDocument::new(
        Uuid::nil(),
        source_original.event_id(),
        source_original.document_time(),
        source_original.available_time(),
        DocumentMethodEffect::Revision,
        Some(source_original.document_id()),
        source_original.observed_event_time(),
        source_original.memberships().to_vec(),
    )
    .expect("hostile derivative");
    assert!(derivative.document_id() < source_original.document_id());

    let mut augmented_documents = generated.documents().to_vec();
    augmented_documents.push(derivative.clone());
    let augmented = manifest_with_documents(&generated, augmented_documents);
    let by_id: BTreeMap<_, _> = augmented
        .documents()
        .iter()
        .map(|document| (document.document_id(), document.method_effect()))
        .collect();
    let projected = augmented
        .document_transition_pairs()
        .expect("projection with original available");
    assert!(projected.iter().all(|(source, target)| {
        by_id[source] == DocumentMethodEffect::Original
            && by_id[target] == DocumentMethodEffect::Original
    }));

    let derivative_only_documents = generated
        .documents()
        .iter()
        .filter(|document| document.event_id() != source_event)
        .cloned()
        .chain(std::iter::once(derivative))
        .collect();
    let derivative_only = manifest_with_documents(&generated, derivative_only_documents);
    assert_eq!(
        derivative_only.document_transition_pairs(),
        Err(SimulationError::ManifestInvariantViolation)
    );
}
