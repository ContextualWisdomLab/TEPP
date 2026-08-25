//! Event, system, document, and available time cannot stand in for assertion.

use assertion_clock::{
    AssertionClockError, ClockFamily, identity_recovery_rate, refuse_available_time_as_assertion,
    refuse_document_time_as_assertion, refuse_event_time_as_assertion,
    refuse_system_time_as_assertion, stamp_is_assertion,
};

#[test]
fn other_clocks_cannot_stand_in_for_assertion_time() {
    assert_eq!(
        refuse_event_time_as_assertion(),
        Err(AssertionClockError::EventTimeIsNotAssertionTime)
    );
    assert_eq!(
        refuse_system_time_as_assertion(),
        Err(AssertionClockError::SystemTimeIsNotAssertionTime)
    );
    assert_eq!(
        refuse_document_time_as_assertion(),
        Err(AssertionClockError::DocumentTimeIsNotAssertionTime)
    );
    assert_eq!(
        refuse_available_time_as_assertion(),
        Err(AssertionClockError::AvailableTimeIsNotAssertionTime)
    );
    assert!(stamp_is_assertion(ClockFamily::AssertionTime).expect("assertion"));
    assert!(!stamp_is_assertion(ClockFamily::EventTime).expect("event"));
    assert!(!stamp_is_assertion(ClockFamily::SystemTime).expect("system"));
    assert!(!stamp_is_assertion(ClockFamily::DocumentTime).expect("document"));
    assert!(!stamp_is_assertion(ClockFamily::AvailableTime).expect("available"));
}

#[test]
fn recovered_assertion_stamps_match_known_truth_better_than_event_stand_in() {
    let recovered = [
        ClockFamily::AssertionTime,
        ClockFamily::AssertionTime,
        ClockFamily::AssertionTime,
    ];
    let collapsed = [
        ClockFamily::EventTime,
        ClockFamily::EventTime,
        ClockFamily::EventTime,
    ];
    let recovered_flags = [
        stamp_is_assertion(recovered[0]).expect("r0"),
        stamp_is_assertion(recovered[1]).expect("r1"),
        stamp_is_assertion(recovered[2]).expect("r2"),
    ];
    let collapsed_flags = [
        stamp_is_assertion(collapsed[0]).expect("c0"),
        stamp_is_assertion(collapsed[1]).expect("c1"),
        stamp_is_assertion(collapsed[2]).expect("c2"),
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
        Err(AssertionClockError::InvalidAssertionPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true], &[]),
        Err(AssertionClockError::InvalidAssertionPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true, false], &[true]),
        Err(AssertionClockError::InvalidAssertionPayload)
    );
}
