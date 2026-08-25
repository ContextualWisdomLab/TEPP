//! Observed, inferred, and unobserved statuses stay distinct.

use relation_absence::{
    ObservationStatus, RelationAbsenceError, refuse_absence_as_negative, status_recovery_rate,
};

#[test]
fn unobserved_pairs_cannot_become_negative_evidence() {
    assert_eq!(
        refuse_absence_as_negative(ObservationStatus::Unobserved),
        Err(RelationAbsenceError::AbsenceIsNotNegative)
    );
    refuse_absence_as_negative(ObservationStatus::Observed).expect("observed is not absence");
    refuse_absence_as_negative(ObservationStatus::Inferred).expect("inferred is not absence");
}

#[test]
fn recovered_statuses_match_known_truth_better_than_an_absence_collapse() {
    let truth = [
        ObservationStatus::Observed,
        ObservationStatus::Inferred,
        ObservationStatus::Unobserved,
    ];
    let recovered = truth;
    let collapsed = [
        ObservationStatus::Observed,
        ObservationStatus::Observed,
        ObservationStatus::Observed,
    ];
    let recovered_rate = status_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = status_recovery_rate(&truth, &collapsed).expect("collapsed");
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
        status_recovery_rate(&[], &[]),
        Err(RelationAbsenceError::InvalidObservationPayload)
    );
    assert_eq!(
        status_recovery_rate(&[ObservationStatus::Observed], &[]),
        Err(RelationAbsenceError::InvalidObservationPayload)
    );
    assert_eq!(
        status_recovery_rate(
            &[ObservationStatus::Observed, ObservationStatus::Unobserved],
            &[ObservationStatus::Observed]
        ),
        Err(RelationAbsenceError::InvalidObservationPayload)
    );
}
