//! Support, contradiction, summary, and `outcome_of` are not state transitions.

use support_edge::{
    EvidenceKind, SupportEdgeError, edge_kind_recovery_rate, refuse_evidence_as_transition,
};

#[test]
fn evidential_kinds_cannot_become_state_transitions() {
    assert_eq!(
        refuse_evidence_as_transition(EvidenceKind::Support),
        Err(SupportEdgeError::EvidenceIsNotTransition)
    );
    assert_eq!(
        refuse_evidence_as_transition(EvidenceKind::Contradiction),
        Err(SupportEdgeError::EvidenceIsNotTransition)
    );
    assert_eq!(
        refuse_evidence_as_transition(EvidenceKind::Summarizes),
        Err(SupportEdgeError::EvidenceIsNotTransition)
    );
    assert_eq!(
        refuse_evidence_as_transition(EvidenceKind::OutcomeOf),
        Err(SupportEdgeError::EvidenceIsNotTransition)
    );
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_support_collapse() {
    let truth = [
        EvidenceKind::Support,
        EvidenceKind::Contradiction,
        EvidenceKind::Summarizes,
        EvidenceKind::OutcomeOf,
    ];
    let recovered = truth;
    let collapsed = [
        EvidenceKind::Support,
        EvidenceKind::Support,
        EvidenceKind::Support,
        EvidenceKind::Support,
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
        Err(SupportEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        edge_kind_recovery_rate(&[EvidenceKind::Support], &[]),
        Err(SupportEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        edge_kind_recovery_rate(
            &[EvidenceKind::Support, EvidenceKind::Contradiction],
            &[EvidenceKind::Support]
        ),
        Err(SupportEdgeError::InvalidEdgePayload)
    );
}
