//! Association, precedence, and document links are not causal language.

use causal_language::{
    CausalLanguageError, ClaimKind, claim_kind_recovery_rate, refuse_unidentified_as_causal,
};

#[test]
fn unidentified_claims_cannot_become_causal_language() {
    assert_eq!(
        refuse_unidentified_as_causal(ClaimKind::Association),
        Err(CausalLanguageError::UnidentifiedIsNotCausal)
    );
    assert_eq!(
        refuse_unidentified_as_causal(ClaimKind::TemporalPrecedence),
        Err(CausalLanguageError::UnidentifiedIsNotCausal)
    );
    assert_eq!(
        refuse_unidentified_as_causal(ClaimKind::DocumentLink),
        Err(CausalLanguageError::UnidentifiedIsNotCausal)
    );
    refuse_unidentified_as_causal(ClaimKind::IdentifiedExperimental)
        .expect("experimental identification is causal-eligible");
    refuse_unidentified_as_causal(ClaimKind::IdentifiedQuasiExperimental)
        .expect("quasi-experimental identification is causal-eligible");
    refuse_unidentified_as_causal(ClaimKind::IdentifiedObservational)
        .expect("defensible observational identification is causal-eligible");
}

#[test]
fn recovered_claim_kinds_match_known_truth_better_than_a_causal_collapse() {
    let truth = [
        ClaimKind::Association,
        ClaimKind::TemporalPrecedence,
        ClaimKind::DocumentLink,
        ClaimKind::IdentifiedExperimental,
    ];
    let recovered = truth;
    let collapsed = [
        ClaimKind::IdentifiedExperimental,
        ClaimKind::IdentifiedExperimental,
        ClaimKind::IdentifiedExperimental,
        ClaimKind::IdentifiedExperimental,
    ];
    let recovered_rate = claim_kind_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = claim_kind_recovery_rate(&truth, &collapsed).expect("collapsed");
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
fn empty_or_mismatched_claim_payloads_fail_closed() {
    assert_eq!(
        claim_kind_recovery_rate(&[], &[]),
        Err(CausalLanguageError::InvalidClaimPayload)
    );
    assert_eq!(
        claim_kind_recovery_rate(&[ClaimKind::Association], &[]),
        Err(CausalLanguageError::InvalidClaimPayload)
    );
    assert_eq!(
        claim_kind_recovery_rate(
            &[ClaimKind::Association, ClaimKind::DocumentLink],
            &[ClaimKind::Association]
        ),
        Err(CausalLanguageError::InvalidClaimPayload)
    );
}
