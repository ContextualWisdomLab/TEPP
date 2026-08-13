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
