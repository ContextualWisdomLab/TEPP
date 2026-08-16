//! Other TEPP clocks cannot stand in for system time.

use system_clock::{
    ClockFamily, SystemClockError, identity_recovery_rate, refuse_assertion_time_as_system,
    refuse_available_time_as_system, refuse_cutoff_time_as_system, refuse_document_time_as_system,
    refuse_event_time_as_system, stamp_is_system,
};

#[test]
fn other_clocks_cannot_stand_in_for_system_time() {
    assert_eq!(
        refuse_event_time_as_system(),
        Err(SystemClockError::EventTimeIsNotSystemTime)
    );
    assert_eq!(
        refuse_assertion_time_as_system(),
        Err(SystemClockError::AssertionTimeIsNotSystemTime)
    );
    assert_eq!(
        refuse_document_time_as_system(),
        Err(SystemClockError::DocumentTimeIsNotSystemTime)
    );
    assert_eq!(
        refuse_available_time_as_system(),
        Err(SystemClockError::AvailableTimeIsNotSystemTime)
    );
    assert_eq!(
        refuse_cutoff_time_as_system(),
        Err(SystemClockError::CutoffTimeIsNotSystemTime)
    );
    assert!(stamp_is_system(ClockFamily::SystemTime).expect("system"));
    assert!(!stamp_is_system(ClockFamily::EventTime).expect("event"));
    assert!(!stamp_is_system(ClockFamily::AssertionTime).expect("assertion"));
    assert!(!stamp_is_system(ClockFamily::DocumentTime).expect("document"));
    assert!(!stamp_is_system(ClockFamily::AvailableTime).expect("available"));
    assert!(!stamp_is_system(ClockFamily::CutoffTime).expect("cutoff"));
}

#[test]
fn recovered_system_stamps_match_known_truth_better_than_event_stand_in() {
    let recovered = [
        ClockFamily::SystemTime,
        ClockFamily::SystemTime,
        ClockFamily::SystemTime,
    ];
    let collapsed = [
        ClockFamily::EventTime,
        ClockFamily::EventTime,
        ClockFamily::EventTime,
    ];
    let recovered_flags = [
        stamp_is_system(recovered[0]).expect("r0"),
        stamp_is_system(recovered[1]).expect("r1"),
        stamp_is_system(recovered[2]).expect("r2"),
    ];
    let collapsed_flags = [
        stamp_is_system(collapsed[0]).expect("c0"),
        stamp_is_system(collapsed[1]).expect("c1"),
        stamp_is_system(collapsed[2]).expect("c2"),
    ];
    let truth_flags = [true, true, true];
    let recovered_rate = identity_recovery_rate(&truth_flags, &recovered_flags).expect("ok");
    let collapsed_rate = identity_recovery_rate(&truth_flags, &collapsed_flags).expect("bad");
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
fn empty_or_mismatched_identity_payloads_fail_closed() {
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(SystemClockError::InvalidSystemPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true], &[]),
        Err(SystemClockError::InvalidSystemPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true, false], &[true]),
        Err(SystemClockError::InvalidSystemPayload)
    );
}
