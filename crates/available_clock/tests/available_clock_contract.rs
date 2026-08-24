//! Event and system time cannot stand in for availability.

use available_clock::{
    AvailableClockError, ClockFamily, eligibility_recovery_rate, refuse_event_time_as_available,
    refuse_system_time_as_available, stamp_is_available,
};

#[test]
fn event_and_system_time_cannot_stand_in_for_availability() {
    assert_eq!(
        refuse_event_time_as_available(),
        Err(AvailableClockError::EventTimeIsNotAvailableTime)
    );
    assert_eq!(
        refuse_system_time_as_available(),
        Err(AvailableClockError::SystemTimeIsNotAvailableTime)
    );
    assert!(stamp_is_available(ClockFamily::AvailableTime).expect("available"));
    assert!(!stamp_is_available(ClockFamily::EventTime).expect("event"));
    assert!(!stamp_is_available(ClockFamily::SystemTime).expect("system"));
}

#[test]
fn recovered_availability_stamps_match_known_truth_better_than_system_stand_in() {
    let truth = [
        ClockFamily::AvailableTime,
        ClockFamily::AvailableTime,
        ClockFamily::AvailableTime,
    ];
    let recovered = truth;
    let collapsed = [
        ClockFamily::SystemTime,
        ClockFamily::SystemTime,
        ClockFamily::SystemTime,
    ];
    let recovered_flags = [
        stamp_is_available(recovered[0]).expect("r0"),
        stamp_is_available(recovered[1]).expect("r1"),
        stamp_is_available(recovered[2]).expect("r2"),
    ];
    let collapsed_flags = [
        stamp_is_available(collapsed[0]).expect("c0"),
        stamp_is_available(collapsed[1]).expect("c1"),
        stamp_is_available(collapsed[2]).expect("c2"),
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
        Err(AvailableClockError::InvalidAvailabilityPayload)
    );
    assert_eq!(
        eligibility_recovery_rate(&[true], &[]),
        Err(AvailableClockError::InvalidAvailabilityPayload)
    );
    assert_eq!(
        eligibility_recovery_rate(&[true, false], &[true]),
        Err(AvailableClockError::InvalidAvailabilityPayload)
    );
}
