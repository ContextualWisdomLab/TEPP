//! Edge contracts for the simulation-owned event-to-document transition projection.

use tepp_simulation::{SimulationConfig, SimulationError, TruthManifest, generate};
use uuid::Uuid;

fn without_event_documents(generated: &TruthManifest, missing_event: Uuid) -> TruthManifest {
    let documents = generated
        .documents()
        .iter()
        .filter(|document| document.event_id() != missing_event)
        .cloned()
        .collect();
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
fn projection_fails_closed_when_either_transition_endpoint_has_no_owned_document() {
    let generated = generate(
        SimulationConfig::new(2029, 3, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0)
            .expect("simulation config"),
    )
    .expect("generated truth");

    let missing_source = without_event_documents(&generated, generated.events()[0].event_id());
    assert_eq!(
        missing_source.document_transition_pairs(),
        Err(SimulationError::ManifestInvariantViolation)
    );
    assert_eq!(
        missing_source.observed_document_transition_pairs(),
        Err(SimulationError::ManifestInvariantViolation)
    );

    let missing_target = without_event_documents(&generated, generated.events()[2].event_id());
    assert_eq!(
        missing_target.document_transition_pairs(),
        Err(SimulationError::ManifestInvariantViolation)
    );
    assert_eq!(
        missing_target.observed_document_transition_pairs(),
        Err(SimulationError::ManifestInvariantViolation)
    );
}
