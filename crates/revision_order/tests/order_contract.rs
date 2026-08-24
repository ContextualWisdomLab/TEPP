//! Later revisions cannot carry earlier or equal system time.

use revision_order::{
    DocumentRevision, RevisionOrderError, order_recovery_rate, refuse_nonincreasing_system_time,
    revisions_are_increasing,
};

fn revision(number: u32, system: i64) -> DocumentRevision {
    DocumentRevision::new(number, system).expect("revision")
}

#[test]
fn later_revisions_cannot_move_backward_in_system_time() {
    let first = revision(1, 10);
    let second = revision(2, 20);
    let backward = revision(3, 15);
    assert!(revisions_are_increasing(first, second).expect("increasing"));
    refuse_nonincreasing_system_time(first, second).expect("ok");
    assert!(!revisions_are_increasing(second, backward).expect("backward"));
    assert_eq!(
        refuse_nonincreasing_system_time(second, backward),
        Err(RevisionOrderError::SystemTimeDidNotIncrease)
    );
    assert_eq!(
        refuse_nonincreasing_system_time(second, revision(4, 20)),
        Err(RevisionOrderError::SystemTimeDidNotIncrease)
    );
}

#[test]
fn recovered_order_flags_match_known_truth_better_than_accepting_all() {
    let pairs = [
        (revision(1, 10), revision(2, 20)),
        (revision(2, 20), revision(3, 15)),
        (revision(3, 30), revision(4, 40)),
    ];
    let truth = [true, false, true];
    let recovered = [
        revisions_are_increasing(pairs[0].0, pairs[0].1).expect("p0"),
        revisions_are_increasing(pairs[1].0, pairs[1].1).expect("p1"),
        revisions_are_increasing(pairs[2].0, pairs[2].1).expect("p2"),
    ];
    let collapsed = [true, true, true];
    let recovered_rate = order_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = order_recovery_rate(&truth, &collapsed).expect("collapsed");
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
fn empty_or_invalid_revision_payloads_fail_closed() {
    assert_eq!(
        DocumentRevision::new(0, 10),
        Err(RevisionOrderError::InvalidRevisionPayload)
    );
    assert_eq!(
        order_recovery_rate(&[], &[]),
        Err(RevisionOrderError::InvalidRevisionPayload)
    );
    assert_eq!(
        order_recovery_rate(&[true], &[]),
        Err(RevisionOrderError::InvalidRevisionPayload)
    );
    assert_eq!(
        order_recovery_rate(&[true, false], &[true]),
        Err(RevisionOrderError::InvalidRevisionPayload)
    );
}
