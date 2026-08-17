//! A summary is not a state transition and not the source document.

use summarizes_edge::{
    identity_recovery_rate, refuse_summary_as_source_identity, refuse_summary_as_transition,
    SummarizesEdgeError, SummarizesKind,
};

#[test]
fn a_summary_cannot_become_a_transition_or_the_source_identity() {
    assert_eq!(
        refuse_summary_as_transition(SummarizesKind::Summary),
        Err(SummarizesEdgeError::SummaryIsNotTransition)
    );
    assert_eq!(
        refuse_summary_as_source_identity(SummarizesKind::Summary),
        Err(SummarizesEdgeError::SummaryIsNotSourceIdentity)
    );
    refuse_summary_as_transition(SummarizesKind::SourceDocument).expect("source");
    refuse_summary_as_source_identity(SummarizesKind::SourceDocument).expect("source");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_source_collapse() {
    let truth = [
        SummarizesKind::Summary,
        SummarizesKind::SourceDocument,
        SummarizesKind::Summary,
    ];
    let recovered = truth;
    let collapsed = [
        SummarizesKind::SourceDocument,
        SummarizesKind::SourceDocument,
        SummarizesKind::SourceDocument,
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
        Err(SummarizesEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        identity_recovery_rate(&[SummarizesKind::Summary], &[]),
        Err(SummarizesEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[SummarizesKind::Summary, SummarizesKind::SourceDocument],
            &[SummarizesKind::Summary]
        ),
        Err(SummarizesEdgeError::InvalidEdgePayload)
    );
}
