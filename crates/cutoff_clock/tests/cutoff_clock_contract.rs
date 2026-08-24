//! Event, system, and availability time cannot stand in for knowledge cutoff.

use cutoff_clock::{
    ClockFamily, CutoffClockError, eligibility_recovery_rate, refuse_available_time_as_cutoff,
    refuse_event_time_as_cutoff, refuse_system_time_as_cutoff, stamp_is_cutoff,
};

#[test]
fn event_system_and_available_time_cannot_stand_in_for_cutoff() {
    assert_eq!(
        refuse_event_time_as_cutoff(),
        Err(CutoffClockError::EventTimeIsNotKnowledgeCutoff)
    );
    assert_eq!(
        refuse_system_time_as_cutoff(),
        Err(CutoffClockError::SystemTimeIsNotKnowledgeCutoff)
    );
    assert_eq!(
        refuse_available_time_as_cutoff(),
        Err(CutoffClockError::AvailableTimeIsNotKnowledgeCutoff)
    );
    assert!(stamp_is_cutoff(ClockFamily::KnowledgeCutoff).expect("cutoff"));
    assert!(!stamp_is_cutoff(ClockFamily::EventTime).expect("event"));
    assert!(!stamp_is_cutoff(ClockFamily::SystemTime).expect("system"));
    assert!(!stamp_is_cutoff(ClockFamily::AvailableTime).expect("available"));
}

#[test]
fn recovered_cutoff_stamps_match_known_truth_better_than_available_stand_in() {
    let truth = [
        ClockFamily::KnowledgeCutoff,
        ClockFamily::KnowledgeCutoff,
        ClockFamily::KnowledgeCutoff,
    ];
    let recovered = truth;
    let collapsed = [
        ClockFamily::AvailableTime,
        ClockFamily::AvailableTime,
        ClockFamily::AvailableTime,
    ];
    let recovered_flags = [
        stamp_is_cutoff(recovered[0]).expect("r0"),
        stamp_is_cutoff(recovered[1]).expect("r1"),
        stamp_is_cutoff(recovered[2]).expect("r2"),
    ];
    let collapsed_flags = [
        stamp_is_cutoff(collapsed[0]).expect("c0"),
        stamp_is_cutoff(collapsed[1]).expect("c1"),
        stamp_is_cutoff(collapsed[2]).expect("c2"),
    ];
    let truth_flags = [true, true, true];
    let recovered_rate = eligibility_recovery_rate(&truth_flags, &recovered_flags).expect("ok");
    let collapsed_rate = eligibility_recovery_rate(&truth_flags, &collapsed_flags).expect("bad");
    let expected = {
        let mut matches = 0_u32;
        for (truth_flag, decided_flag) in truth_flags.iter().zip(recovered_flags.iter()) {
            if truth_flag == decided_flag {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth_flags.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_eligibility_payloads_fail_closed() {
    assert_eq!(
        eligibility_recovery_rate(&[], &[]),
        Err(CutoffClockError::InvalidCutoffPayload)
    );
    assert_eq!(
        eligibility_recovery_rate(&[true], &[]),
        Err(CutoffClockError::InvalidCutoffPayload)
    );
    assert_eq!(
        eligibility_recovery_rate(&[true, false], &[true]),
        Err(CutoffClockError::InvalidCutoffPayload)
    );
}
