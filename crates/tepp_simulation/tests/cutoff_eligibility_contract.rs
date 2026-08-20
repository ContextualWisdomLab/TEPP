//! Delayed-reporting documents cannot enter a historical fit before they exist.

use temporal_core::KnowledgeCutoff;
use tepp_simulation::{SimulationConfig, SimulationError, generate, refuse_unavailable_document};

#[test]
fn delayed_documents_are_excluded_and_counts_match_known_truth() {
    let config = SimulationConfig::new(21, 4, 1, 2, 48, 24, 0, 0, 0, 0, 0, 0).expect("cfg");
    let manifest = generate(config).expect("corpus");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-01-03T00:00:00Z").expect("cutoff");

    let eligible = manifest.documents_eligible_at_cutoff(&cutoff);
    let truth_count = manifest
        .documents()
        .iter()
        .filter(|document| document.available_time().instant() <= cutoff.instant())
        .count();
    assert_eq!(eligible.len(), truth_count);
    assert!(
        truth_count < manifest.document_count(),
        "cutoff must exclude at least one delayed document"
    );

    let boundary_document = manifest.documents().first().expect("boundary document");
    let boundary_cutoff =
        KnowledgeCutoff::parse_rfc3339(&boundary_document.available_time().to_rfc3339())
            .expect("boundary cutoff");
    let boundary_eligible = manifest.documents_eligible_at_cutoff(&boundary_cutoff);
    assert!(
        boundary_eligible
            .iter()
            .any(|document| document.document_id() == boundary_document.document_id()),
        "a document available exactly at cutoff must be eligible"
    );
    refuse_unavailable_document(boundary_document, &boundary_cutoff)
        .expect("availability equal to cutoff is valid");

    for document in eligible {
        refuse_unavailable_document(document, &cutoff).expect("eligible");
    }
    let late = manifest
        .documents()
        .iter()
        .find(|document| document.available_time().instant() > cutoff.instant())
        .expect("late document");
    assert_eq!(
        refuse_unavailable_document(late, &cutoff),
        Err(SimulationError::TemporalInvariantViolation)
    );
}
