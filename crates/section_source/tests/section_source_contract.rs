//! Report section boilerplate is not unique content and not stopword deletion.

use section_source::{
    SectionKind, SectionSourceError, identity_recovery_rate, refuse_section_as_stopword_deletion,
    refuse_section_as_unique_content,
};

#[test]
fn a_section_cannot_become_unique_content_or_stopword_deletion() {
    assert_eq!(
        refuse_section_as_unique_content(SectionKind::SectionBoilerplate),
        Err(SectionSourceError::SectionIsNotUniqueContent)
    );
    assert_eq!(
        refuse_section_as_stopword_deletion(SectionKind::SectionBoilerplate),
        Err(SectionSourceError::SectionIsNotStopwordDeletion)
    );
    refuse_section_as_unique_content(SectionKind::UniqueContent).expect("unique");
    refuse_section_as_stopword_deletion(SectionKind::UniqueContent).expect("unique");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_unique_collapse() {
    let truth = [
        SectionKind::SectionBoilerplate,
        SectionKind::UniqueContent,
        SectionKind::SectionBoilerplate,
    ];
    let recovered = truth;
    let collapsed = [
        SectionKind::UniqueContent,
        SectionKind::UniqueContent,
        SectionKind::UniqueContent,
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
        Err(SectionSourceError::InvalidSectionPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[SectionKind::SectionBoilerplate], &[]),
        Err(SectionSourceError::InvalidSectionPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[SectionKind::SectionBoilerplate, SectionKind::UniqueContent],
            &[SectionKind::SectionBoilerplate]
        ),
        Err(SectionSourceError::InvalidSectionPayload)
    );
}
