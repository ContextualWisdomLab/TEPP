//! Operational logs cannot carry source text, source identity, or a blanket mask.

use operational_log::{
    ActionCode, AnalyticalSubject, OperationalLogError, OperationalLogRecord, SourceIdentity,
    log_recovery_rate, refuse_blanket_mask_as_log_grant, refuse_source_identity_as_subject,
    refuse_source_identity_in_log, refuse_source_text_in_log, replay_operational_log,
};

fn record(action: ActionCode, system_time_seconds: i64) -> OperationalLogRecord {
    OperationalLogRecord::try_record(
        action,
        AnalyticalSubject::new(),
        system_time_seconds,
        None,
        None,
        false,
    )
    .expect("record")
}

#[test]
fn source_text_identity_and_blanket_mask_cannot_enter_the_log() {
    assert_eq!(
        refuse_source_text_in_log(Some("raw source")),
        Err(OperationalLogError::SourceTextNotLoggable)
    );
    assert_eq!(
        refuse_source_identity_in_log(Some(SourceIdentity::new())),
        Err(OperationalLogError::SourceIdentityNotLoggable)
    );
    assert_eq!(
        refuse_blanket_mask_as_log_grant(true),
        Err(OperationalLogError::BlanketMaskIsNotAuthorization)
    );
    assert_eq!(
        refuse_source_identity_as_subject(SourceIdentity::new()),
        Err(OperationalLogError::SourceIdentityNotLoggable)
    );
}

#[test]
fn replayed_log_lines_match_known_truth_better_than_a_collapsed_action() {
    let truth = [
        record(ActionCode::AnalyticalRead, 10),
        record(ActionCode::AnalyticalWrite, 11),
        record(ActionCode::AnalyticalReject, 12),
    ];
    let replayed = replay_operational_log(&truth).expect("replay");
    let collapsed = [
        record(ActionCode::AnalyticalRead, 10),
        record(ActionCode::AnalyticalRead, 11),
        record(ActionCode::AnalyticalRead, 12),
    ];
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
        log_recovery_rate(&[record(ActionCode::AnalyticalRead, 10)], &[]),
        Err(OperationalLogError::InvalidLogPayload)
    );
    assert_eq!(
        log_recovery_rate(
            &[
                record(ActionCode::AnalyticalRead, 10),
                record(ActionCode::AnalyticalWrite, 11),
            ],
            &[record(ActionCode::AnalyticalRead, 10)],
        ),
        Err(OperationalLogError::InvalidLogPayload)
    );
}

#[test]
fn construction_rejects_source_payloads_and_blanket_mask_intent() {
    let subject = AnalyticalSubject::new();
    assert_eq!(
        OperationalLogRecord::try_record(
            ActionCode::AnalyticalRead,
            subject,
            10,
            Some("raw source"),
            None,
            false,
        ),
        Err(OperationalLogError::SourceTextNotLoggable)
    );
    assert_eq!(
        OperationalLogRecord::try_record(
            ActionCode::AnalyticalRead,
            subject,
            10,
            None,
            Some(SourceIdentity::new()),
            false,
        ),
        Err(OperationalLogError::SourceIdentityNotLoggable)
    );
    assert_eq!(
        OperationalLogRecord::try_record(ActionCode::AnalyticalRead, subject, 10, None, None, true),
        Err(OperationalLogError::BlanketMaskIsNotAuthorization)
    );
}

#[test]
fn replay_rejects_decreasing_system_time() {
    let ordered = [
        record(ActionCode::AnalyticalRead, 10),
        record(ActionCode::AnalyticalWrite, 11),
    ];
    assert!(replay_operational_log(&ordered).is_ok());
    let reverse = [
        record(ActionCode::AnalyticalRead, 11),
        record(ActionCode::AnalyticalWrite, 10),
    ];
    assert_eq!(
        replay_operational_log(&reverse),
        Err(OperationalLogError::InvalidLogPayload)
    );
}
