//! Input→process→outcome edges never move backward in event time.

use outcome_order::{
    OutcomeKind, OutcomeOrderError, kind_recovery_rate, refuse_outcome_of_as_transition,
    refuse_reverse_ipo_order,
};

#[test]
fn input_and_process_edges_cannot_move_backward_in_event_time() {
    refuse_reverse_ipo_order(OutcomeKind::InputTo, 1, 2).expect("forward input");
    refuse_reverse_ipo_order(OutcomeKind::ProcessTo, 2, 3).expect("forward process");
    assert_eq!(
        refuse_reverse_ipo_order(OutcomeKind::InputTo, 3, 1),
        Err(OutcomeOrderError::ReverseIpoOrder)
    );
    assert_eq!(
        refuse_reverse_ipo_order(OutcomeKind::ProcessTo, 5, 4),
        Err(OutcomeOrderError::ReverseIpoOrder)
    );
    assert_eq!(
        refuse_reverse_ipo_order(OutcomeKind::InputTo, 7, 7),
        Err(OutcomeOrderError::UncertainIpoOrder)
    );
}

#[test]
fn outcome_of_may_point_backward_and_is_not_a_transition() {
    refuse_reverse_ipo_order(OutcomeKind::OutcomeOf, 9, 2).expect("provenance may look back");
    refuse_reverse_ipo_order(OutcomeKind::OutcomeOf, 2, 2).expect("same-rank provenance");
    assert_eq!(
        refuse_outcome_of_as_transition(OutcomeKind::OutcomeOf),
        Err(OutcomeOrderError::OutcomeOfIsNotTransition)
    );
    refuse_outcome_of_as_transition(OutcomeKind::InputTo).expect("input_to is a transition");
    refuse_outcome_of_as_transition(OutcomeKind::ProcessTo).expect("process_to is a transition");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_an_input_collapse() {
    let truth = [
        OutcomeKind::InputTo,
        OutcomeKind::ProcessTo,
        OutcomeKind::OutcomeOf,
    ];
    let recovered = truth;
    let collapsed = [
        OutcomeKind::InputTo,
        OutcomeKind::InputTo,
        OutcomeKind::InputTo,
    ];
    let recovered_rate = kind_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = kind_recovery_rate(&truth, &collapsed).expect("collapsed");
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
        kind_recovery_rate(&[], &[]),
        Err(OutcomeOrderError::InvalidEdgePayload)
    );
    assert_eq!(
        kind_recovery_rate(&[OutcomeKind::InputTo], &[]),
        Err(OutcomeOrderError::InvalidEdgePayload)
    );
    assert_eq!(
        kind_recovery_rate(
            &[OutcomeKind::InputTo, OutcomeKind::ProcessTo],
            &[OutcomeKind::InputTo]
        ),
        Err(OutcomeOrderError::InvalidEdgePayload)
    );
}
