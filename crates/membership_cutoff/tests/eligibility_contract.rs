//! Delayed memberships cannot enter a historical estimation row.

use membership_cutoff::{
    MembershipCutoffError, MembershipObservation, eligibility_recovery_rate, eligible_memberships,
    refuse_membership_after_cutoff,
};

fn observation(unit: u128, available: i64) -> MembershipObservation {
    MembershipObservation::new(unit, available)
}

#[test]
fn delayed_memberships_cannot_enter_a_historical_row() {
    let cutoff = 100;
    assert_eq!(
        refuse_membership_after_cutoff(101, cutoff),
        Err(MembershipCutoffError::AvailabilityExceedsCutoff)
    );
    refuse_membership_after_cutoff(100, cutoff).expect("on cutoff");
    refuse_membership_after_cutoff(99, cutoff).expect("before cutoff");
    let observations = [observation(1, 90), observation(2, 101), observation(3, 100)];
    let eligible = eligible_memberships(&observations, cutoff).expect("filter");
    assert_eq!(eligible, vec![observation(1, 90), observation(3, 100)]);
}

#[test]
fn recovered_eligibility_matches_known_truth_better_than_keeping_all() {
    let cutoff = 50;
    let observations = [observation(1, 10), observation(2, 80), observation(3, 50)];
    let eligible = eligible_memberships(&observations, cutoff).expect("filter");
    let truth = [true, false, true];
    let recovered = [
        observations[0].available_seconds() <= cutoff,
        observations[1].available_seconds() <= cutoff,
        observations[2].available_seconds() <= cutoff,
    ];
    let collapsed = [true, true, true];
    let recovered_rate = eligibility_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = eligibility_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_flag, decided_flag) in truth.iter().zip(recovered.iter()) {
            if truth_flag == decided_flag {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
    assert_eq!(eligible.len(), 2);
}

#[test]
fn empty_or_mismatched_eligibility_payloads_fail_closed() {
    assert_eq!(
        eligible_memberships(&[], 10),
        Err(MembershipCutoffError::InvalidEligibilityPayload)
    );
    assert_eq!(
        eligibility_recovery_rate(&[], &[]),
        Err(MembershipCutoffError::InvalidEligibilityPayload)
    );
    assert_eq!(
        eligibility_recovery_rate(&[true], &[]),
        Err(MembershipCutoffError::InvalidEligibilityPayload)
    );
    assert_eq!(
        eligibility_recovery_rate(&[true, false], &[true]),
        Err(MembershipCutoffError::InvalidEligibilityPayload)
    );
}
