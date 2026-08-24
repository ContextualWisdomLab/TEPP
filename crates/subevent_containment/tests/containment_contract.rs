//! A subevent cannot escape its parent event-time interval.

use subevent_containment::{
    EventInterval, SubeventContainmentError, containment_recovery_rate, interval_contains,
    refuse_escaped_subevent,
};

fn interval(start: i64, end: i64) -> EventInterval {
    EventInterval::new(start, end).expect("interval")
}

#[test]
fn escaped_subevents_cannot_attach_to_the_parent() {
    let parent = interval(10, 40);
    let inside = interval(15, 30);
    let early = interval(0, 20);
    let late = interval(30, 50);
    assert!(interval_contains(parent, inside).expect("inside"));
    refuse_escaped_subevent(parent, inside).expect("contained");
    assert!(!interval_contains(parent, early).expect("early"));
    assert_eq!(
        refuse_escaped_subevent(parent, early),
        Err(SubeventContainmentError::SubeventEscapesParent)
    );
    assert_eq!(
        refuse_escaped_subevent(parent, late),
        Err(SubeventContainmentError::SubeventEscapesParent)
    );
}

#[test]
fn recovered_containment_matches_known_truth_better_than_accepting_all() {
    let parent = interval(10, 40);
    let children = [interval(15, 30), interval(0, 20), interval(12, 18)];
    let truth = [true, false, true];
    let recovered = [
        interval_contains(parent, children[0]).expect("c0"),
        interval_contains(parent, children[1]).expect("c1"),
        interval_contains(parent, children[2]).expect("c2"),
    ];
    let collapsed = [true, true, true];
    let recovered_rate = containment_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = containment_recovery_rate(&truth, &collapsed).expect("collapsed");
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
}

#[test]
fn empty_or_invalid_interval_payloads_fail_closed() {
    assert_eq!(
        EventInterval::new(10, 10),
        Err(SubeventContainmentError::InvalidIntervalPayload)
    );
    assert_eq!(
        EventInterval::new(10, 9),
        Err(SubeventContainmentError::InvalidIntervalPayload)
    );
    assert_eq!(
        containment_recovery_rate(&[], &[]),
        Err(SubeventContainmentError::InvalidIntervalPayload)
    );
    assert_eq!(
        containment_recovery_rate(&[true], &[]),
        Err(SubeventContainmentError::InvalidIntervalPayload)
    );
    assert_eq!(
        containment_recovery_rate(&[true, false], &[true]),
        Err(SubeventContainmentError::InvalidIntervalPayload)
    );
}
