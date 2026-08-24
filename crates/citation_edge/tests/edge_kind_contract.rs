//! Citation and retrospective edges cannot be promoted to state transitions.

use citation_edge::{
    CitationEdgeError, ProvenanceKind, edge_kind_recovery_rate, refuse_provenance_as_transition,
};

#[test]
fn provenance_kinds_cannot_become_state_transitions() {
    assert_eq!(
        refuse_provenance_as_transition(ProvenanceKind::Citation),
        Err(CitationEdgeError::ProvenanceIsNotTransition)
    );
    assert_eq!(
        refuse_provenance_as_transition(ProvenanceKind::Translation),
        Err(CitationEdgeError::ProvenanceIsNotTransition)
    );
    assert_eq!(
        refuse_provenance_as_transition(ProvenanceKind::Revision),
        Err(CitationEdgeError::ProvenanceIsNotTransition)
    );
    assert_eq!(
        refuse_provenance_as_transition(ProvenanceKind::RetrospectiveReport),
        Err(CitationEdgeError::ProvenanceIsNotTransition)
    );
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_transition_collapse() {
    let truth = [
        ProvenanceKind::Citation,
        ProvenanceKind::Translation,
        ProvenanceKind::Revision,
    ];
    let recovered = truth;
    let collapsed = [
        ProvenanceKind::Citation,
        ProvenanceKind::Citation,
        ProvenanceKind::Citation,
    ];
    let recovered_rate = edge_kind_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = edge_kind_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_kind, decided_kind) in truth.iter().zip(recovered.iter()) {
            if truth_kind == decided_kind {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_kind_payloads_fail_closed() {
    assert_eq!(
        edge_kind_recovery_rate(&[], &[]),
        Err(CitationEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        edge_kind_recovery_rate(&[ProvenanceKind::Citation], &[]),
        Err(CitationEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        edge_kind_recovery_rate(
            &[ProvenanceKind::Citation, ProvenanceKind::Revision],
            &[ProvenanceKind::Citation]
        ),
        Err(CitationEdgeError::InvalidEdgePayload)
    );
}
