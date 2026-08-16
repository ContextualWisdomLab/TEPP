//! Operational logs cannot carry source text, source identity, or a blanket mask.

use operational_log::{
    OperationalLogError, OperationalLogRecord, log_recovery_rate, refuse_blanket_mask_as_log_grant,
    refuse_source_identity_in_log, refuse_source_text_in_log, replay_operational_log,
};

fn record(action: u16, subject: u128) -> OperationalLogRecord {
    OperationalLogRecord::new(action, subject, 10)
}

#[test]
fn source_text_identity_and_blanket_mask_cannot_enter_the_log() {
    assert_eq!(
        refuse_source_text_in_log(),
        Err(OperationalLogError::SourceTextNotLoggable)
    );
    assert_eq!(
        refuse_source_identity_in_log(),
        Err(OperationalLogError::SourceIdentityNotLoggable)
    );
    assert_eq!(
        refuse_blanket_mask_as_log_grant(),
        Err(OperationalLogError::BlanketMaskIsNotAuthorization)
    );
}

#[test]
fn replayed_log_lines_match_known_truth_better_than_a_collapsed_action() {
    let truth = [record(1, 11), record(2, 22), record(3, 33)];
    let replayed = replay_operational_log(&truth).expect("replay");
    let collapsed = [record(1, 11), record(1, 22), record(1, 33)];
    let recovered_rate = log_recovery_rate(&truth, &replayed).expect("recovered");
    let collapsed_rate = log_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_record, decided_record) in truth.iter().zip(replayed.iter()) {
            if truth_record == decided_record {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_log_payloads_fail_closed() {
    assert_eq!(
        replay_operational_log(&[]),
        Err(OperationalLogError::InvalidLogPayload)
    );
    assert_eq!(
        log_recovery_rate(&[], &[]),
        Err(OperationalLogError::InvalidLogPayload)
    );
    assert_eq!(
        log_recovery_rate(&[record(1, 11)], &[]),
        Err(OperationalLogError::InvalidLogPayload)
    );
    assert_eq!(
        log_recovery_rate(&[record(1, 11), record(2, 22)], &[record(1, 11)]),
        Err(OperationalLogError::InvalidLogPayload)
    );
}
