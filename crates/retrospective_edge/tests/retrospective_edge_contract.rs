//! Retrospective reporting is not a transition and not a translation.

use retrospective_edge::{
    identity_recovery_rate, refuse_retrospective_as_transition,
    refuse_retrospective_as_translation, RetrospectiveEdgeError, RetrospectiveKind,
};

#[test]
fn retrospective_reporting_cannot_become_a_transition_or_a_translation() {
    assert_eq!(
        refuse_retrospective_as_transition(RetrospectiveKind::RetrospectiveReport),
        Err(RetrospectiveEdgeError::RetrospectiveIsNotTransition)
    );
    assert_eq!(
        refuse_retrospective_as_translation(RetrospectiveKind::RetrospectiveReport),
        Err(RetrospectiveEdgeError::RetrospectiveIsNotTranslation)
    );
    refuse_retrospective_as_transition(RetrospectiveKind::ForwardReport).expect("forward");
    refuse_retrospective_as_translation(RetrospectiveKind::ForwardReport).expect("forward");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_translation_collapse() {
    let truth = [
        RetrospectiveKind::RetrospectiveReport,
        RetrospectiveKind::ForwardReport,
        RetrospectiveKind::RetrospectiveReport,
    ];
    let recovered = truth;
    let collapsed = [
        RetrospectiveKind::ForwardReport,
        RetrospectiveKind::ForwardReport,
        RetrospectiveKind::ForwardReport,
    ];
    let recovered_rate = identity_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = identity_recovery_rate(&truth, &collapsed).expect("collapsed");
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
        identity_recovery_rate(&[], &[]),
        Err(RetrospectiveEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        identity_recovery_rate(&[RetrospectiveKind::RetrospectiveReport], &[]),
        Err(RetrospectiveEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[
                RetrospectiveKind::RetrospectiveReport,
                RetrospectiveKind::ForwardReport
            ],
            &[RetrospectiveKind::RetrospectiveReport]
        ),
        Err(RetrospectiveEdgeError::InvalidEdgePayload)
    );
}
