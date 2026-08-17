//! Translation, same-language copy, and revision are not state transitions.

use translation_edge::{
    TranslationEdgeError, TranslationKind, edge_kind_recovery_rate,
    refuse_same_language_as_translation, refuse_translation_as_transition,
};

#[test]
fn provenance_kinds_cannot_become_state_transitions() {
    assert_eq!(
        refuse_translation_as_transition(TranslationKind::Translation),
        Err(TranslationEdgeError::TranslationIsNotTransition)
    );
    assert_eq!(
        refuse_translation_as_transition(TranslationKind::SameLanguageCopy),
        Err(TranslationEdgeError::TranslationIsNotTransition)
    );
    assert_eq!(
        refuse_translation_as_transition(TranslationKind::Revision),
        Err(TranslationEdgeError::TranslationIsNotTransition)
    );
}

#[test]
fn same_primary_language_cannot_be_classified_as_translation() {
    assert_eq!(
        refuse_same_language_as_translation("en", "en-US"),
        Err(TranslationEdgeError::SameLanguageIsNotTranslation)
    );
    assert_eq!(
        refuse_same_language_as_translation("KO", "ko-KR"),
        Err(TranslationEdgeError::SameLanguageIsNotTranslation)
    );
    refuse_same_language_as_translation("en", "ko").expect("cross-language");
    refuse_same_language_as_translation("zh-Hans", "en-GB").expect("cross-language");
}

#[test]
fn empty_or_malformed_language_tags_fail_closed() {
    assert_eq!(
        refuse_same_language_as_translation("", "en"),
        Err(TranslationEdgeError::InvalidLanguageTag)
    );
    assert_eq!(
        refuse_same_language_as_translation("en", " "),
        Err(TranslationEdgeError::InvalidLanguageTag)
    );
    assert_eq!(
        refuse_same_language_as_translation("-US", "en"),
        Err(TranslationEdgeError::InvalidLanguageTag)
    );
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_translation_collapse() {
    let truth = [
        TranslationKind::Translation,
        TranslationKind::SameLanguageCopy,
        TranslationKind::Revision,
    ];
    let recovered = truth;
    let collapsed = [
        TranslationKind::Translation,
        TranslationKind::Translation,
        TranslationKind::Translation,
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
        Err(TranslationEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        edge_kind_recovery_rate(&[TranslationKind::Translation], &[]),
        Err(TranslationEdgeError::InvalidEdgePayload)
    );
    assert_eq!(
        edge_kind_recovery_rate(
            &[TranslationKind::Translation, TranslationKind::Revision],
            &[TranslationKind::Translation]
        ),
        Err(TranslationEdgeError::InvalidEdgePayload)
    );
}
