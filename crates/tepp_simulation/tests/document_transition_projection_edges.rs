//! Edge contracts for the simulation-owned event-to-document transition projection.

use tepp_simulation::{SimulationConfig, SimulationError, TruthManifest, generate};

#[test]
fn projection_fails_closed_when_a_transition_event_has_no_owned_document() {
    let generated = generate(
        SimulationConfig::new(2029, 3, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0)
            .expect("simulation config"),
    )
    .expect("generated truth");
    let missing_event = generated.events()[1].event_id();
    let documents = generated
        .documents()
        .iter()
        .filter(|document| document.event_id() != missing_event)
        .cloned()
        .collect();
    let malformed = TruthManifest::new(
        generated.seed(),
        generated.config_digest().to_owned(),
        generated.events().to_vec(),
        documents,
        generated.true_relations().to_vec(),
        generated.observed_relations().to_vec(),
        generated.topic_truth().clone(),
    );

    assert_eq!(
        malformed.document_transition_pairs(),
        Err(SimulationError::ManifestInvariantViolation)
    );
}
