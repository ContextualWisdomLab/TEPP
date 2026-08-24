//! Corpus-background wording is not unique content and not stopword deletion.

use corpus_background::{
    CorpusBackgroundError, CorpusBackgroundKind, identity_recovery_rate,
    refuse_corpus_background_as_stopword_deletion, refuse_corpus_background_as_unique_content,
};

#[test]
fn corpus_background_cannot_become_unique_content_or_stopword_deletion() {
    assert_eq!(
        refuse_corpus_background_as_unique_content(CorpusBackgroundKind::CorpusBackground),
        Err(CorpusBackgroundError::CorpusBackgroundIsNotUniqueContent)
    );
    assert_eq!(
        refuse_corpus_background_as_stopword_deletion(CorpusBackgroundKind::CorpusBackground),
        Err(CorpusBackgroundError::CorpusBackgroundIsNotStopwordDeletion)
    );
    refuse_corpus_background_as_unique_content(CorpusBackgroundKind::UniqueContent)
        .expect("unique");
    refuse_corpus_background_as_stopword_deletion(CorpusBackgroundKind::UniqueContent)
        .expect("unique");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_unique_content_collapse() {
    let truth = [
        CorpusBackgroundKind::CorpusBackground,
        CorpusBackgroundKind::UniqueContent,
        CorpusBackgroundKind::CorpusBackground,
    ];
    let recovered = truth;
    let collapsed = [
        CorpusBackgroundKind::UniqueContent,
        CorpusBackgroundKind::UniqueContent,
        CorpusBackgroundKind::UniqueContent,
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
        Err(CorpusBackgroundError::InvalidCorpusBackgroundPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[CorpusBackgroundKind::CorpusBackground], &[]),
        Err(CorpusBackgroundError::InvalidCorpusBackgroundPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[
                CorpusBackgroundKind::CorpusBackground,
                CorpusBackgroundKind::UniqueContent
            ],
            &[CorpusBackgroundKind::CorpusBackground]
        ),
        Err(CorpusBackgroundError::InvalidCorpusBackgroundPayload)
    );
}
