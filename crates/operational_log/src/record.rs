//! Operational-log records bound to opaque analytical subjects.

use crate::OperationalLogError;
use uuid::Uuid;

/// Closed action vocabulary for operational-log records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum ActionCode {
    /// Read an analytical artifact through an authorized path.
    AnalyticalRead = 1,
    /// Write an analytical artifact through an authorized path.
    AnalyticalWrite = 2,
    /// Refuse an unauthorized analytical operation.
    AnalyticalReject = 3,
}

/// Opaque analytical subject generated independently from source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnalyticalSubject(Uuid);

impl AnalyticalSubject {
    /// Generate a fresh opaque subject identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for AnalyticalSubject {
    fn default() -> Self {
        Self::new()
    }
}

/// Source identity that must never become an operational-log subject.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceIdentity(Uuid);

impl SourceIdentity {
    /// Construct a source-identity token for a refusal-path test or boundary.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for SourceIdentity {
    fn default() -> Self {
        Self::new()
    }
}

/// One operational-log line without source text or source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationalLogRecord {
    action_code: ActionCode,
    analytical_subject: AnalyticalSubject,
    system_time_seconds: i64,
}

impl OperationalLogRecord {
    /// Construct a record only after source-separation authorization checks.
    ///
    /// # Errors
    ///
    /// Returns a source-separation or blanket-mask error when prohibited
    /// payloads or authorization intent are supplied.
    pub fn try_record(
        action_code: ActionCode,
        analytical_subject: AnalyticalSubject,
        system_time_seconds: i64,
        source_text: Option<&str>,
        source_identity: Option<SourceIdentity>,
        blanket_mask_requested: bool,
    ) -> Result<Self, OperationalLogError> {
        refuse_source_text_in_log(source_text)?;
        refuse_source_identity_in_log(source_identity)?;
        refuse_blanket_mask_as_log_grant(blanket_mask_requested)?;
        Ok(Self {
            action_code,
            analytical_subject,
            system_time_seconds,
        })
    }

    /// Closed action code, never source text.
    #[must_use]
    pub const fn action_code(self) -> ActionCode {
        self.action_code
    }

    /// Opaque analytical subject, never a source identity.
    #[must_use]
    pub const fn analytical_subject(self) -> AnalyticalSubject {
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
/// supplied or system time decreases between adjacent records.
pub fn replay_operational_log(
    records: &[OperationalLogRecord],
) -> Result<Vec<OperationalLogRecord>, OperationalLogError> {
    if records.is_empty() {
        return Err(OperationalLogError::InvalidLogPayload);
    }
    if records
        .windows(2)
        .any(|pair| pair[1].system_time_seconds < pair[0].system_time_seconds)
    {
        return Err(OperationalLogError::InvalidLogPayload);
    }
    Ok(records.to_vec())
}

/// Refuse to place raw source text in the operational log.
///
/// # Errors
///
/// Always returns [`OperationalLogError::SourceTextNotLoggable`].
pub fn refuse_source_text_in_log(source_text: Option<&str>) -> Result<(), OperationalLogError> {
    if source_text.is_some() {
        Err(OperationalLogError::SourceTextNotLoggable)
    } else {
        Ok(())
    }
}

/// Refuse to place source identity in the operational log.
///
/// # Errors
///
/// Always returns [`OperationalLogError::SourceIdentityNotLoggable`].
pub fn refuse_source_identity_in_log(
    source_identity: Option<SourceIdentity>,
) -> Result<(), OperationalLogError> {
    if source_identity.is_some() {
        Err(OperationalLogError::SourceIdentityNotLoggable)
    } else {
        Ok(())
    }
}

/// Refuse to reinterpret a source identity as an analytical subject.
///
/// # Errors
///
/// Always returns [`OperationalLogError::SourceIdentityNotLoggable`].
pub fn refuse_source_identity_as_subject(
    _source_identity: SourceIdentity,
) -> Result<(), OperationalLogError> {
    Err(OperationalLogError::SourceIdentityNotLoggable)
}

/// Refuse to treat a blanket PII mask as operational-log authorization.
///
/// # Errors
///
/// Always returns [`OperationalLogError::BlanketMaskIsNotAuthorization`].
pub fn refuse_blanket_mask_as_log_grant(
    blanket_mask_requested: bool,
) -> Result<(), OperationalLogError> {
    if blanket_mask_requested {
        Err(OperationalLogError::BlanketMaskIsNotAuthorization)
    } else {
        Ok(())
    }
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
        ActionCode, AnalyticalSubject, OperationalLogRecord, SourceIdentity, log_recovery_rate,
        refuse_blanket_mask_as_log_grant, refuse_source_identity_as_subject,
        refuse_source_identity_in_log, refuse_source_text_in_log, replay_operational_log,
    };
    use crate::OperationalLogError;

    #[test]
    fn local_branches_cover_replay_and_fail_closed_paths() {
        let default_subject = AnalyticalSubject::default();
        let default_source_identity = SourceIdentity::default();
        let truth = [
            OperationalLogRecord::try_record(
                ActionCode::AnalyticalRead,
                default_subject,
                10,
                None,
                None,
                false,
            )
            .expect("first record"),
            OperationalLogRecord::try_record(
                ActionCode::AnalyticalWrite,
                AnalyticalSubject::new(),
                11,
                None,
                None,
                false,
            )
            .expect("second record"),
        ];
        assert_eq!(truth[0].action_code(), ActionCode::AnalyticalRead);
        assert_eq!(truth[0].analytical_subject(), default_subject);
        assert_eq!(truth[0].system_time_seconds(), 10);
        let replayed = replay_operational_log(&truth).expect("replay");
        let matched = log_recovery_rate(&truth, &replayed).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            refuse_source_text_in_log(Some("source")),
            Err(OperationalLogError::SourceTextNotLoggable)
        );
        assert_eq!(
            refuse_source_identity_in_log(Some(default_source_identity)),
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
        assert!(refuse_source_text_in_log(None).is_ok());
        assert!(refuse_source_identity_in_log(None).is_ok());
        assert!(refuse_blanket_mask_as_log_grant(false).is_ok());
        assert_eq!(
            replay_operational_log(&[]),
            Err(OperationalLogError::InvalidLogPayload)
        );
        let reverse = [
            truth[1],
            OperationalLogRecord::try_record(
                ActionCode::AnalyticalRead,
                AnalyticalSubject::new(),
                9,
                None,
                None,
                false,
            )
            .expect("reverse record"),
        ];
        assert_eq!(
            replay_operational_log(&reverse),
            Err(OperationalLogError::InvalidLogPayload)
        );
        assert_eq!(
            OperationalLogRecord::try_record(
                ActionCode::AnalyticalRead,
                AnalyticalSubject::new(),
                10,
                Some("source"),
                None,
                false,
            ),
            Err(OperationalLogError::SourceTextNotLoggable)
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
