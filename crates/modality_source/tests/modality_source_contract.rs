//! Non-lexical modality is not unique content and not stopword deletion.

use modality_source::{
    identity_recovery_rate, refuse_modality_as_stopword_deletion,
    refuse_modality_as_unique_content, ModalityKind, ModalitySourceError,
};

#[test]
fn non_lexical_modality_cannot_become_unique_content_or_stopword_deletion() {
    assert_eq!(
        refuse_modality_as_unique_content(ModalityKind::NonLexicalModality),
        Err(ModalitySourceError::ModalityIsNotUniqueContent)
    );
    assert_eq!(
        refuse_modality_as_stopword_deletion(ModalityKind::NonLexicalModality),
        Err(ModalitySourceError::ModalityIsNotStopwordDeletion)
    );
    refuse_modality_as_unique_content(ModalityKind::UniqueContent).expect("unique");
    refuse_modality_as_stopword_deletion(ModalityKind::UniqueContent).expect("unique");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_unique_content_collapse() {
    let truth = [
        ModalityKind::NonLexicalModality,
        ModalityKind::UniqueContent,
        ModalityKind::NonLexicalModality,
    ];
    let recovered = truth;
    let collapsed = [
        ModalityKind::UniqueContent,
        ModalityKind::UniqueContent,
        ModalityKind::UniqueContent,
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
        Err(ModalitySourceError::InvalidModalityPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[ModalityKind::NonLexicalModality], &[]),
        Err(ModalitySourceError::InvalidModalityPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[
                ModalityKind::NonLexicalModality,
                ModalityKind::UniqueContent
            ],
            &[ModalityKind::NonLexicalModality]
        ),
        Err(ModalitySourceError::InvalidModalityPayload)
    );
}
