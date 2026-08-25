//! Assertion, system, document, and available time cannot stand in for event time.

use event_clock::{
    ClockFamily, EventClockError, identity_recovery_rate, refuse_assertion_time_as_event,
    refuse_available_time_as_event, refuse_document_time_as_event, refuse_system_time_as_event,
    stamp_is_event,
};

#[test]
fn other_clocks_cannot_stand_in_for_event_time() {
    assert_eq!(
        refuse_assertion_time_as_event(),
        Err(EventClockError::AssertionTimeIsNotEventTime)
    );
    assert_eq!(
        refuse_system_time_as_event(),
        Err(EventClockError::SystemTimeIsNotEventTime)
    );
    assert_eq!(
        refuse_document_time_as_event(),
        Err(EventClockError::DocumentTimeIsNotEventTime)
    );
    assert_eq!(
        refuse_available_time_as_event(),
        Err(EventClockError::AvailableTimeIsNotEventTime)
    );
    assert!(stamp_is_event(ClockFamily::EventTime).expect("event"));
    assert!(!stamp_is_event(ClockFamily::AssertionTime).expect("assertion"));
    assert!(!stamp_is_event(ClockFamily::SystemTime).expect("system"));
    assert!(!stamp_is_event(ClockFamily::DocumentTime).expect("document"));
    assert!(!stamp_is_event(ClockFamily::AvailableTime).expect("available"));
}

#[test]
fn recovered_event_stamps_match_known_truth_better_than_assertion_stand_in() {
    let recovered = [
        ClockFamily::EventTime,
        ClockFamily::EventTime,
        ClockFamily::EventTime,
    ];
    let collapsed = [
        ClockFamily::AssertionTime,
        ClockFamily::AssertionTime,
        ClockFamily::AssertionTime,
    ];
    let recovered_flags = [
        stamp_is_event(recovered[0]).expect("r0"),
        stamp_is_event(recovered[1]).expect("r1"),
        stamp_is_event(recovered[2]).expect("r2"),
    ];
    let collapsed_flags = [
        stamp_is_event(collapsed[0]).expect("c0"),
        stamp_is_event(collapsed[1]).expect("c1"),
        stamp_is_event(collapsed[2]).expect("c2"),
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
        Err(EventClockError::InvalidEventPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true], &[]),
        Err(EventClockError::InvalidEventPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true, false], &[true]),
        Err(EventClockError::InvalidEventPayload)
    );
}
