//! Privileged-access replay cannot carry source identity or a blanket mask.

use privileged_access::{
    PrivilegedAccessError, PrivilegedAction, PrivilegedAuditRecord, audit_recovery_rate,
    refuse_blanket_mask_as_audit_grant, refuse_source_identity_in_audit, replay_privileged_audit,
};

fn record(action: PrivilegedAction, subject: u128, allowed: bool) -> PrivilegedAuditRecord {
    PrivilegedAuditRecord::new(action, 7, subject, 10, allowed)
}

#[test]
fn source_identity_and_blanket_mask_cannot_authorize_audit() {
    assert_eq!(
        refuse_source_identity_in_audit(),
        Err(PrivilegedAccessError::SourceIdentityNotAuditable)
    );
    assert_eq!(
        refuse_blanket_mask_as_audit_grant(),
        Err(PrivilegedAccessError::BlanketMaskIsNotAuthorization)
    );
}

#[test]
fn replayed_decisions_match_known_truth_better_than_a_collapsed_allow_log() {
    let truth = [
        record(PrivilegedAction::ReidentificationExport, 1, true),
        record(PrivilegedAction::GrantUse, 2, false),
        record(PrivilegedAction::ReidentificationExport, 3, false),
    ];
    let replayed = replay_privileged_audit(&truth).expect("replay");
    let collapsed = [
        record(PrivilegedAction::ReidentificationExport, 1, true),
        record(PrivilegedAction::GrantUse, 2, true),
        record(PrivilegedAction::ReidentificationExport, 3, true),
    ];
    let recovered_rate = audit_recovery_rate(&truth, &replayed).expect("recovered");
    let collapsed_rate = audit_recovery_rate(&truth, &collapsed).expect("collapsed");
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
fn empty_or_mismatched_audit_payloads_fail_closed() {
    assert_eq!(
        replay_privileged_audit(&[]),
        Err(PrivilegedAccessError::InvalidAuditPayload)
    );
    assert_eq!(
        audit_recovery_rate(&[], &[]),
        Err(PrivilegedAccessError::InvalidAuditPayload)
    );
    assert_eq!(
        audit_recovery_rate(&[record(PrivilegedAction::GrantUse, 1, true)], &[]),
        Err(PrivilegedAccessError::InvalidAuditPayload)
    );
    assert_eq!(
        audit_recovery_rate(
            &[
                record(PrivilegedAction::GrantUse, 1, true),
                record(PrivilegedAction::GrantUse, 2, false)
            ],
            &[record(PrivilegedAction::GrantUse, 1, true)]
        ),
        Err(PrivilegedAccessError::InvalidAuditPayload)
    );
}
