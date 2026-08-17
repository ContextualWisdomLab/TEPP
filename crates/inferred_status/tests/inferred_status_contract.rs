//! Inferred relations cannot be promoted to observed evidence or transitions.

use inferred_status::{
    identity_recovery_rate, refuse_inferred_as_observed, refuse_inferred_as_transition,
    status_is_observed, EvidenceStatus, InferredStatusError,
};

#[test]
fn inferred_status_cannot_become_observed_or_a_transition() {
    assert_eq!(
        refuse_inferred_as_observed(EvidenceStatus::Inferred),
        Err(InferredStatusError::InferredIsNotObserved)
    );
    assert_eq!(
        refuse_inferred_as_transition(EvidenceStatus::Inferred),
        Err(InferredStatusError::InferredIsNotTransition)
    );
    refuse_inferred_as_observed(EvidenceStatus::Observed).expect("observed stays observed");
    refuse_inferred_as_transition(EvidenceStatus::Observed)
        .expect("observed may be considered for promotion elsewhere");
    assert!(status_is_observed(EvidenceStatus::Observed).expect("observed"));
    assert!(!status_is_observed(EvidenceStatus::Inferred).expect("inferred"));
}

#[test]
fn recovered_statuses_match_known_truth_better_than_an_observed_collapse() {
    let truth = [
        EvidenceStatus::Observed,
        EvidenceStatus::Inferred,
        EvidenceStatus::Inferred,
    ];
    let recovered = truth;
    let collapsed = [
        EvidenceStatus::Observed,
        EvidenceStatus::Observed,
        EvidenceStatus::Observed,
    ];
    let recovered_rate = identity_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = identity_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_status, decided_status) in truth.iter().zip(recovered.iter()) {
            if truth_status == decided_status {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_status_payloads_fail_closed() {
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(InferredStatusError::InvalidStatusPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[EvidenceStatus::Inferred], &[]),
        Err(InferredStatusError::InvalidStatusPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[EvidenceStatus::Observed, EvidenceStatus::Inferred],
            &[EvidenceStatus::Observed]
        ),
        Err(InferredStatusError::InvalidStatusPayload)
    );
}
