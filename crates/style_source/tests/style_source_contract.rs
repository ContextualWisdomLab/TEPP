//! House-voice style residue is not unique content and not stopword deletion.

use style_source::{
    identity_recovery_rate, refuse_style_as_stopword_deletion, refuse_style_as_unique_content,
    StyleKind, StyleSourceError,
};

#[test]
fn style_residue_cannot_become_unique_content_or_stopword_deletion() {
    assert_eq!(
        refuse_style_as_unique_content(StyleKind::StyleResidue),
        Err(StyleSourceError::StyleIsNotUniqueContent)
    );
    assert_eq!(
        refuse_style_as_stopword_deletion(StyleKind::StyleResidue),
        Err(StyleSourceError::StyleIsNotStopwordDeletion)
    );
    refuse_style_as_unique_content(StyleKind::UniqueContent).expect("unique");
    refuse_style_as_stopword_deletion(StyleKind::UniqueContent).expect("unique");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_unique_content_collapse() {
    let truth = [
        StyleKind::StyleResidue,
        StyleKind::UniqueContent,
        StyleKind::StyleResidue,
    ];
    let recovered = truth;
    let collapsed = [
        StyleKind::UniqueContent,
        StyleKind::UniqueContent,
        StyleKind::UniqueContent,
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
        Err(StyleSourceError::InvalidStylePayload)
    );
    assert_eq!(
        identity_recovery_rate(&[StyleKind::StyleResidue], &[]),
        Err(StyleSourceError::InvalidStylePayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[StyleKind::StyleResidue, StyleKind::UniqueContent],
            &[StyleKind::StyleResidue]
        ),
        Err(StyleSourceError::InvalidStylePayload)
    );
}
