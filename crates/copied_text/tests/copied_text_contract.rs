//! Copied-text residue is not unique content and not stopword deletion.

use copied_text::{
    CopiedKind, CopiedTextError, identity_recovery_rate, refuse_copied_text_as_stopword_deletion,
    refuse_copied_text_as_unique_content,
};

#[test]
fn copied_text_cannot_become_unique_content_or_stopword_deletion() {
    assert_eq!(
        refuse_copied_text_as_unique_content(CopiedKind::CopiedText),
        Err(CopiedTextError::CopiedTextIsNotUniqueContent)
    );
    assert_eq!(
        refuse_copied_text_as_stopword_deletion(CopiedKind::CopiedText),
        Err(CopiedTextError::CopiedTextIsNotStopwordDeletion)
    );
    refuse_copied_text_as_unique_content(CopiedKind::UniqueContent).expect("unique");
    refuse_copied_text_as_stopword_deletion(CopiedKind::UniqueContent).expect("unique");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_unique_content_collapse() {
    let truth = [
        CopiedKind::CopiedText,
        CopiedKind::UniqueContent,
        CopiedKind::CopiedText,
    ];
    let recovered = truth;
    let collapsed = [
        CopiedKind::UniqueContent,
        CopiedKind::UniqueContent,
        CopiedKind::UniqueContent,
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
        Err(CopiedTextError::InvalidCopiedPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[CopiedKind::CopiedText], &[]),
        Err(CopiedTextError::InvalidCopiedPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[CopiedKind::CopiedText, CopiedKind::UniqueContent],
            &[CopiedKind::CopiedText]
        ),
        Err(CopiedTextError::InvalidCopiedPayload)
    );
}
