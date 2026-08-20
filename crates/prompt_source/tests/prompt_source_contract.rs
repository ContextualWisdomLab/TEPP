//! Prompt boilerplate is not unique content and not stopword deletion.

use prompt_source::{
    PromptKind, PromptSourceError, identity_recovery_rate, refuse_prompt_as_stopword_deletion,
    refuse_prompt_as_unique_content,
};

#[test]
fn prompt_boilerplate_cannot_become_unique_content_or_stopword_deletion() {
    assert_eq!(
        refuse_prompt_as_unique_content(PromptKind::PromptBoilerplate),
        Err(PromptSourceError::PromptIsNotUniqueContent)
    );
    assert_eq!(
        refuse_prompt_as_stopword_deletion(PromptKind::PromptBoilerplate),
        Err(PromptSourceError::PromptIsNotStopwordDeletion)
    );
    refuse_prompt_as_unique_content(PromptKind::UniqueContent).expect("unique");
    refuse_prompt_as_stopword_deletion(PromptKind::UniqueContent).expect("unique");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_unique_content_collapse() {
    let truth = [
        PromptKind::PromptBoilerplate,
        PromptKind::UniqueContent,
        PromptKind::PromptBoilerplate,
    ];
    let recovered = truth;
    let collapsed = [
        PromptKind::UniqueContent,
        PromptKind::UniqueContent,
        PromptKind::UniqueContent,
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
        Err(PromptSourceError::InvalidPromptPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[PromptKind::PromptBoilerplate], &[]),
        Err(PromptSourceError::InvalidPromptPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[PromptKind::PromptBoilerplate, PromptKind::UniqueContent],
            &[PromptKind::PromptBoilerplate]
        ),
        Err(PromptSourceError::InvalidPromptPayload)
    );
}
