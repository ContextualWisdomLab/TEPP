//! Privileged-access audit records and replay.

use crate::PrivilegedAccessError;

/// Privileged action recorded without source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivilegedAction {
    /// Export of a separately protected identity map.
    ReidentificationExport,
    /// Use of a purpose-bound or tenant/role grant.
    GrantUse,
}

/// One privileged-access decision bound to an opaque analytical subject.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrivilegedAuditRecord {
    action: PrivilegedAction,
    purpose_code: u16,
    analytical_subject: u128,
    system_time_seconds: i64,
    allowed: bool,
}

impl PrivilegedAuditRecord {
    /// Record a privileged decision against an opaque analytical subject.
    #[must_use]
    pub const fn new(
        action: PrivilegedAction,
        purpose_code: u16,
        analytical_subject: u128,
        system_time_seconds: i64,
        allowed: bool,
    ) -> Self {
        Self {
            action,
            purpose_code,
            analytical_subject,
            system_time_seconds,
            allowed,
        }
    }

    /// Privileged action that was requested.
    #[must_use]
    pub const fn action(self) -> PrivilegedAction {
        self.action
    }

    /// Purpose code bound to the decision.
    #[must_use]
    pub const fn purpose_code(self) -> u16 {
        self.purpose_code
    }

    /// Opaque analytical subject, never a source identity.
    #[must_use]
    pub const fn analytical_subject(self) -> u128 {
        self.analytical_subject
    }

    /// System/record time of the decision in seconds.
    #[must_use]
    pub const fn system_time_seconds(self) -> i64 {
        self.system_time_seconds
    }

    /// Whether the privileged action was allowed.
    #[must_use]
    pub const fn allowed(self) -> bool {
        self.allowed
    }
}

/// Replay a privileged-access log without rewriting decisions.
///
/// # Errors
///
/// Returns [`PrivilegedAccessError::InvalidAuditPayload`] when no records are
/// supplied.
pub fn replay_privileged_audit(
    records: &[PrivilegedAuditRecord],
) -> Result<Vec<PrivilegedAuditRecord>, PrivilegedAccessError> {
    if records.is_empty() {
        return Err(PrivilegedAccessError::InvalidAuditPayload);
    }
    Ok(records.to_vec())
}

/// Refuse to place source identity in the privileged-access audit log.
///
/// # Errors
///
/// Always returns [`PrivilegedAccessError::SourceIdentityNotAuditable`].
pub fn refuse_source_identity_in_audit() -> Result<(), PrivilegedAccessError> {
    Err(PrivilegedAccessError::SourceIdentityNotAuditable)
}

/// Refuse to treat a blanket PII mask as privileged-access authorization.
///
/// # Errors
///
/// Always returns [`PrivilegedAccessError::BlanketMaskIsNotAuthorization`].
pub fn refuse_blanket_mask_as_audit_grant() -> Result<(), PrivilegedAccessError> {
    Err(PrivilegedAccessError::BlanketMaskIsNotAuthorization)
}

/// Fraction of replayed audit records that match known truth.
///
/// # Errors
///
/// Returns [`PrivilegedAccessError::InvalidAuditPayload`] when either slice
/// is empty or the lengths differ.
pub fn audit_recovery_rate(
    truth: &[PrivilegedAuditRecord],
    decided: &[PrivilegedAuditRecord],
) -> Result<f64, PrivilegedAccessError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(PrivilegedAccessError::InvalidAuditPayload);
    }
    let mut matches = 0_u32;
    for (truth_record, decided_record) in truth.iter().zip(decided) {
        if truth_record == decided_record {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        PrivilegedAction, PrivilegedAuditRecord, audit_recovery_rate,
        refuse_blanket_mask_as_audit_grant, refuse_source_identity_in_audit,
        replay_privileged_audit,
    };
    use crate::PrivilegedAccessError;

    #[test]
    fn local_branches_cover_replay_and_fail_closed_paths() {
        let truth = [
            PrivilegedAuditRecord::new(PrivilegedAction::GrantUse, 7, 1, 10, true),
            PrivilegedAuditRecord::new(PrivilegedAction::ReidentificationExport, 7, 2, 11, false),
        ];
        assert_eq!(truth[0].action(), PrivilegedAction::GrantUse);
        assert_eq!(truth[0].purpose_code(), 7);
        assert_eq!(truth[0].analytical_subject(), 1);
        assert_eq!(truth[0].system_time_seconds(), 10);
        assert!(truth[0].allowed());
        let replayed = replay_privileged_audit(&truth).expect("replay");
        let matched = audit_recovery_rate(&truth, &replayed).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            refuse_source_identity_in_audit(),
            Err(PrivilegedAccessError::SourceIdentityNotAuditable)
        );
        assert_eq!(
            refuse_blanket_mask_as_audit_grant(),
            Err(PrivilegedAccessError::BlanketMaskIsNotAuthorization)
        );
        assert_eq!(
            replay_privileged_audit(&[]),
            Err(PrivilegedAccessError::InvalidAuditPayload)
        );
        assert_eq!(
            audit_recovery_rate(&[], &[]),
            Err(PrivilegedAccessError::InvalidAuditPayload)
        );
        assert_eq!(
            audit_recovery_rate(&truth, &[]),
            Err(PrivilegedAccessError::InvalidAuditPayload)
        );
    }
}
