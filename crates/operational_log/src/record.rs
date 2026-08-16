//! Operational-log records bound to opaque analytical subjects.

use crate::OperationalLogError;

/// One operational-log line without source text or source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationalLogRecord {
    action_code: u16,
    analytical_subject: u128,
    system_time_seconds: i64,
}

impl OperationalLogRecord {
    /// Record an action against an opaque analytical subject.
    #[must_use]
    pub const fn new(action_code: u16, analytical_subject: u128, system_time_seconds: i64) -> Self {
        Self {
            action_code,
            analytical_subject,
            system_time_seconds,
        }
    }

    /// Closed action code, never source text.
    #[must_use]
    pub const fn action_code(self) -> u16 {
        self.action_code
    }

    /// Opaque analytical subject, never a source identity.
    #[must_use]
    pub const fn analytical_subject(self) -> u128 {
        self.analytical_subject
    }

    /// System/record time of the log line in seconds.
    #[must_use]
    pub const fn system_time_seconds(self) -> i64 {
        self.system_time_seconds
    }
}

/// Replay an operational log without rewriting lines.
///
/// # Errors
///
/// Returns [`OperationalLogError::InvalidLogPayload`] when no records are
/// supplied.
pub fn replay_operational_log(
    records: &[OperationalLogRecord],
) -> Result<Vec<OperationalLogRecord>, OperationalLogError> {
    if records.is_empty() {
        return Err(OperationalLogError::InvalidLogPayload);
    }
    Ok(records.to_vec())
}

/// Refuse to place raw source text in the operational log.
///
/// # Errors
///
/// Always returns [`OperationalLogError::SourceTextNotLoggable`].
pub fn refuse_source_text_in_log() -> Result<(), OperationalLogError> {
    Err(OperationalLogError::SourceTextNotLoggable)
}

/// Refuse to place source identity in the operational log.
///
/// # Errors
///
/// Always returns [`OperationalLogError::SourceIdentityNotLoggable`].
pub fn refuse_source_identity_in_log() -> Result<(), OperationalLogError> {
    Err(OperationalLogError::SourceIdentityNotLoggable)
}

/// Refuse to treat a blanket PII mask as operational-log authorization.
///
/// # Errors
///
/// Always returns [`OperationalLogError::BlanketMaskIsNotAuthorization`].
pub fn refuse_blanket_mask_as_log_grant() -> Result<(), OperationalLogError> {
    Err(OperationalLogError::BlanketMaskIsNotAuthorization)
}

/// Fraction of replayed log records that match known truth.
///
/// # Errors
///
/// Returns [`OperationalLogError::InvalidLogPayload`] when either slice is
/// empty or the lengths differ.
pub fn log_recovery_rate(
    truth: &[OperationalLogRecord],
    decided: &[OperationalLogRecord],
) -> Result<f64, OperationalLogError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(OperationalLogError::InvalidLogPayload);
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
        OperationalLogRecord, log_recovery_rate, refuse_blanket_mask_as_log_grant,
        refuse_source_identity_in_log, refuse_source_text_in_log, replay_operational_log,
    };
    use crate::OperationalLogError;

    #[test]
    fn local_branches_cover_replay_and_fail_closed_paths() {
        let truth = [
            OperationalLogRecord::new(1, 11, 10),
            OperationalLogRecord::new(2, 22, 11),
        ];
        assert_eq!(truth[0].action_code(), 1);
        assert_eq!(truth[0].analytical_subject(), 11);
        assert_eq!(truth[0].system_time_seconds(), 10);
        let replayed = replay_operational_log(&truth).expect("replay");
        let matched = log_recovery_rate(&truth, &replayed).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
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
        assert_eq!(
            replay_operational_log(&[]),
            Err(OperationalLogError::InvalidLogPayload)
        );
        assert_eq!(
            log_recovery_rate(&[], &[]),
            Err(OperationalLogError::InvalidLogPayload)
        );
        assert_eq!(
            log_recovery_rate(&truth, &[]),
            Err(OperationalLogError::InvalidLogPayload)
        );
    }
}
