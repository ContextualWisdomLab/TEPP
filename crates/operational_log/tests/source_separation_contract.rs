//! Operational logs cannot carry source text, source identity, or a blanket mask.

use operational_log::{
    ACTION_IDENTITY_MAPPING_READ, ACTION_ORDINARY_DIAGNOSIS, ACTION_PRIVILEGED_EXPORT,
    AnalyticalSubject, OperationalLogError, OperationalLogRecord, log_recovery_rate,
    refuse_blanket_mask_as_log_grant, refuse_source_identity_as_subject,
    refuse_source_identity_in_log, refuse_source_text_in_log, replay_operational_log, try_record,
};

/// Author-owned customer renewal text that also names a partner review.
const RENEWAL_SOURCE_TEXT: &str = "Kim Park (kim.park@acme.example) escalated the ACME Corp Q2 renewal after the partner legal review stalled.";
const AUTHOR_SOURCE_IDENTITY: &str = "kim.park@acme.example";
const AUTHOR_SUBJECT: u128 = 0xA11_CE01;
const CUSTOMER_SUBJECT: u128 = 0xA11_CE02;
const PROJECT_SUBJECT: u128 = 0xA11_CE03;
const SYSTEM_TIME: i64 = 1_713_196_800;

fn record(action: u16, subject: u128) -> OperationalLogRecord {
    try_record(
        action,
        AnalyticalSubject::from_opaque(subject),
        SYSTEM_TIME,
        None,
        None,
        false,
    )
    .expect("opaque membership line")
}

#[test]
fn privileged_export_of_author_customer_project_text_cannot_enter_the_log() {
    let author = AnalyticalSubject::from_opaque(AUTHOR_SUBJECT);
    assert_eq!(
        try_record(
            ACTION_PRIVILEGED_EXPORT,
            author,
            SYSTEM_TIME,
            Some(RENEWAL_SOURCE_TEXT),
            None,
            false,
        ),
        Err(OperationalLogError::SourceTextNotLoggable)
    );
    assert_eq!(
        try_record(
            ACTION_PRIVILEGED_EXPORT,
            author,
            SYSTEM_TIME,
            Some(""),
            None,
            false,
        ),
        Err(OperationalLogError::SourceTextNotLoggable)
    );
    assert_eq!(
        refuse_source_text_in_log(RENEWAL_SOURCE_TEXT),
        Err(OperationalLogError::SourceTextNotLoggable)
    );
}

#[test]
fn source_identity_cannot_become_a_log_subject_or_log_field() {
    let author = AnalyticalSubject::from_opaque(AUTHOR_SUBJECT);
    assert_eq!(
        try_record(
            ACTION_IDENTITY_MAPPING_READ,
            author,
            SYSTEM_TIME,
            None,
            Some(AUTHOR_SOURCE_IDENTITY),
            false,
        ),
        Err(OperationalLogError::SourceIdentityNotLoggable)
    );
    assert_eq!(
        refuse_source_identity_in_log(AUTHOR_SOURCE_IDENTITY),
        Err(OperationalLogError::SourceIdentityNotLoggable)
    );
    assert_eq!(
        refuse_source_identity_as_subject(AUTHOR_SOURCE_IDENTITY),
        Err(OperationalLogError::SourceIdentityNotLoggable)
    );
    assert_eq!(
        try_record(
            ACTION_IDENTITY_MAPPING_READ,
            author,
            SYSTEM_TIME,
            None,
            Some(""),
            false,
        ),
        Err(OperationalLogError::SourceIdentityNotLoggable)
    );
}

#[test]
fn blanket_mask_is_not_a_log_grant_and_clear_intent_records_opaque_fields_only() {
    let author = AnalyticalSubject::from_opaque(AUTHOR_SUBJECT);
    assert_eq!(
        try_record(
            ACTION_PRIVILEGED_EXPORT,
            author,
            SYSTEM_TIME,
            None,
            None,
            true,
        ),
        Err(OperationalLogError::BlanketMaskIsNotAuthorization)
    );
    assert_eq!(
        refuse_blanket_mask_as_log_grant(true),
        Err(OperationalLogError::BlanketMaskIsNotAuthorization)
    );
    refuse_blanket_mask_as_log_grant(false).expect("clear intent is not a mask grant");
    let recorded = try_record(
        ACTION_ORDINARY_DIAGNOSIS,
        author,
        SYSTEM_TIME,
        None,
        None,
        false,
    )
    .expect("opaque diagnosis line");
    assert_eq!(recorded.action_code(), ACTION_ORDINARY_DIAGNOSIS);
    assert_eq!(recorded.analytical_subject(), author);
    assert_eq!(recorded.system_time_seconds(), SYSTEM_TIME);
    assert_eq!(author.as_u128(), AUTHOR_SUBJECT);
    let mapping = try_record(
        ACTION_IDENTITY_MAPPING_READ,
        author,
        SYSTEM_TIME,
        None,
        None,
        false,
    )
    .expect("opaque mapping line");
    assert_eq!(mapping.action_code(), ACTION_IDENTITY_MAPPING_READ);
    assert_eq!(
        try_record(
            ACTION_PRIVILEGED_EXPORT,
            author,
            SYSTEM_TIME,
            Some(RENEWAL_SOURCE_TEXT),
            Some(AUTHOR_SOURCE_IDENTITY),
            true,
        ),
        Err(OperationalLogError::SourceTextNotLoggable)
    );
}

#[test]
fn replayed_membership_actions_match_known_truth_better_than_a_collapsed_action() {
    let truth = [
        record(ACTION_PRIVILEGED_EXPORT, AUTHOR_SUBJECT),
        record(ACTION_IDENTITY_MAPPING_READ, CUSTOMER_SUBJECT),
        record(ACTION_ORDINARY_DIAGNOSIS, PROJECT_SUBJECT),
    ];
    let replayed = replay_operational_log(&truth).expect("replay");
    let collapsed = [
        record(ACTION_PRIVILEGED_EXPORT, AUTHOR_SUBJECT),
        record(ACTION_PRIVILEGED_EXPORT, CUSTOMER_SUBJECT),
        record(ACTION_PRIVILEGED_EXPORT, PROJECT_SUBJECT),
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
fn replayed_membership_subjects_match_known_truth_better_than_a_collapsed_subject() {
    let truth = [
        record(ACTION_PRIVILEGED_EXPORT, AUTHOR_SUBJECT),
        record(ACTION_IDENTITY_MAPPING_READ, CUSTOMER_SUBJECT),
        record(ACTION_ORDINARY_DIAGNOSIS, PROJECT_SUBJECT),
    ];
    let replayed = replay_operational_log(&truth).expect("replay");
    let collapsed_subject = [
        record(ACTION_PRIVILEGED_EXPORT, AUTHOR_SUBJECT),
        record(ACTION_IDENTITY_MAPPING_READ, AUTHOR_SUBJECT),
        record(ACTION_ORDINARY_DIAGNOSIS, AUTHOR_SUBJECT),
    ];
    let recovered_rate = log_recovery_rate(&truth, &replayed).expect("recovered");
    let collapsed_rate = log_recovery_rate(&truth, &collapsed_subject).expect("collapsed");
    assert!((recovered_rate - 1.0).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
    assert!(collapsed_rate < 1.0);
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
        log_recovery_rate(&[record(ACTION_PRIVILEGED_EXPORT, AUTHOR_SUBJECT)], &[]),
        Err(OperationalLogError::InvalidLogPayload)
    );
    assert_eq!(
        log_recovery_rate(
            &[
                record(ACTION_PRIVILEGED_EXPORT, AUTHOR_SUBJECT),
                record(ACTION_IDENTITY_MAPPING_READ, CUSTOMER_SUBJECT),
            ],
            &[record(ACTION_PRIVILEGED_EXPORT, AUTHOR_SUBJECT)],
        ),
        Err(OperationalLogError::InvalidLogPayload)
    );
}
