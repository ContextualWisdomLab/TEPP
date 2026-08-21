//! Default stopword deletion cannot erase repeated report language.

use stopword_deletion::{
    DeletionKind, StopwordDeletionError, identity_recovery_rate, refuse_default_stopword_deletion,
};

#[test]
fn a_default_stopword_list_cannot_delete_repeated_report_language() {
    assert_eq!(
        refuse_default_stopword_deletion(DeletionKind::DefaultStopwordList),
        Err(StopwordDeletionError::DefaultStopwordDeletion)
    );
    refuse_default_stopword_deletion(DeletionKind::ExplicitMethodSource).expect("source");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_stopword_collapse() {
    let truth = [
        DeletionKind::ExplicitMethodSource,
        DeletionKind::ExplicitMethodSource,
        DeletionKind::DefaultStopwordList,
    ];
    let recovered = truth;
    let collapsed = [
        DeletionKind::DefaultStopwordList,
        DeletionKind::DefaultStopwordList,
        DeletionKind::DefaultStopwordList,
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
        Err(StopwordDeletionError::InvalidDeletionPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[DeletionKind::DefaultStopwordList], &[]),
        Err(StopwordDeletionError::InvalidDeletionPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[
                DeletionKind::DefaultStopwordList,
                DeletionKind::ExplicitMethodSource
            ],
            &[DeletionKind::DefaultStopwordList]
        ),
        Err(StopwordDeletionError::InvalidDeletionPayload)
    );
}
