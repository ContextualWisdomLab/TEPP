//! A template copy is not the source document and not a state transition.

use copy_identity::{
    CopyIdentityError, CopyKind, identity_recovery_rate, refuse_copy_as_source_identity,
    refuse_copy_as_transition,
};

#[test]
fn a_copy_cannot_become_the_source_identity_or_a_transition() {
    assert_eq!(
        refuse_copy_as_source_identity(CopyKind::TemplateCopy),
        Err(CopyIdentityError::CopyIsNotSourceIdentity)
    );
    assert_eq!(
        refuse_copy_as_transition(CopyKind::TemplateCopy),
        Err(CopyIdentityError::CopyIsNotTransition)
    );
    refuse_copy_as_source_identity(CopyKind::SourceDocument).expect("source");
    refuse_copy_as_transition(CopyKind::SourceDocument).expect("source");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_a_source_collapse() {
    let truth = [
        CopyKind::TemplateCopy,
        CopyKind::SourceDocument,
        CopyKind::TemplateCopy,
    ];
    let recovered = truth;
    let collapsed = [
        CopyKind::SourceDocument,
        CopyKind::SourceDocument,
        CopyKind::SourceDocument,
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
        Err(CopyIdentityError::InvalidCopyPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[CopyKind::TemplateCopy], &[]),
        Err(CopyIdentityError::InvalidCopyPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[CopyKind::TemplateCopy, CopyKind::SourceDocument],
            &[CopyKind::TemplateCopy]
        ),
        Err(CopyIdentityError::InvalidCopyPayload)
    );
}
