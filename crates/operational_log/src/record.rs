//! Operational-log records bound to opaque analytical subjects.

use crate::OperationalLogError;

/// Privileged export of an authorized analysis artifact.
pub const ACTION_PRIVILEGED_EXPORT: u16 = 1_001;
/// Authorized read of a separately protected identity mapping.
pub const ACTION_IDENTITY_MAPPING_READ: u16 = 1_002;
/// Ordinary diagnosis that must not copy source text or source identity.
pub const ACTION_ORDINARY_DIAGNOSIS: u16 = 1_003;

/// Opaque analytical subject that is not a source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnalyticalSubject(u128);

impl AnalyticalSubject {
    /// Bind an already-separated opaque analytical identifier.
    #[must_use]
    pub const fn from_opaque(value: u128) -> Self {
        Self(value)
    }

    /// Return the opaque identifier without exposing a source identity type.
    #[must_use]
    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

/// One operational-log line without source text or source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationalLogRecord {
    action_code: u16,
    analytical_subject: AnalyticalSubject,
    system_time_seconds: i64,
}

impl OperationalLogRecord {
    /// Bind fields after `try_record` has refused forbidden payloads.
    #[must_use]
    pub(crate) const fn new(
        action_code: u16,
        analytical_subject: AnalyticalSubject,
        system_time_seconds: i64,
    ) -> Self {
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
    pub const fn analytical_subject(self) -> AnalyticalSubject {
        self.analytical_subject
    }

    /// System/record time of the log line in seconds.
    #[must_use]
    pub const fn system_time_seconds(self) -> i64 {
        self.system_time_seconds
    }
}

/// Record an operation only when source text, source identity, and blanket
/// masking are absent.
///
/// # Errors
///
/// Returns [`OperationalLogError::SourceTextNotLoggable`] when source text is
/// supplied, [`OperationalLogError::SourceIdentityNotLoggable`] when a source
/// identity is supplied, or
/// [`OperationalLogError::BlanketMaskIsNotAuthorization`] when a blanket mask
/// is treated as a log grant.
pub fn try_record(
    action_code: u16,
    analytical_subject: AnalyticalSubject,
    system_time_seconds: i64,
    source_text: Option<&str>,
    source_identity: Option<&str>,
    blanket_mask: bool,
) -> Result<OperationalLogRecord, OperationalLogError> {
    if let Some(source_text) = source_text {
        refuse_source_text_in_log(source_text)?;
    }
    if let Some(source_identity) = source_identity {
        refuse_source_identity_in_log(source_identity)?;
    }
    refuse_blanket_mask_as_log_grant(blanket_mask)?;
    Ok(OperationalLogRecord::new(
        action_code,
        analytical_subject,
        system_time_seconds,
    ))
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
pub fn refuse_source_text_in_log(_source_text: &str) -> Result<(), OperationalLogError> {
    Err(OperationalLogError::SourceTextNotLoggable)
}

/// Refuse to place source identity in the operational log.
///
/// # Errors
///
/// Always returns [`OperationalLogError::SourceIdentityNotLoggable`].
pub fn refuse_source_identity_in_log(_source_identity: &str) -> Result<(), OperationalLogError> {
    Err(OperationalLogError::SourceIdentityNotLoggable)
}

/// Refuse to treat a source identity as an analytical subject.
///
/// # Errors
///
/// Always returns [`OperationalLogError::SourceIdentityNotLoggable`].
pub fn refuse_source_identity_as_subject(
    _source_identity: &str,
) -> Result<AnalyticalSubject, OperationalLogError> {
    Err(OperationalLogError::SourceIdentityNotLoggable)
}

/// Refuse to treat a blanket PII mask as operational-log authorization.
///
/// # Errors
///
/// Returns [`OperationalLogError::BlanketMaskIsNotAuthorization`] when
/// `blanket_mask` is true.
pub fn refuse_blanket_mask_as_log_grant(blanket_mask: bool) -> Result<(), OperationalLogError> {
    if blanket_mask {
        return Err(OperationalLogError::BlanketMaskIsNotAuthorization);
    }
    Ok(())
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
    #[allow(clippy::cast_precision_loss)]
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        ACTION_ORDINARY_DIAGNOSIS, ACTION_PRIVILEGED_EXPORT, AnalyticalSubject,
        OperationalLogRecord, log_recovery_rate, refuse_blanket_mask_as_log_grant,
        refuse_source_identity_as_subject, refuse_source_identity_in_log,
        refuse_source_text_in_log, replay_operational_log, try_record,
    };
    use crate::OperationalLogError;

    #[test]
    fn local_branches_cover_replay_and_fail_closed_paths() {
        let author = AnalyticalSubject::from_opaque(11);
        let truth = [
            OperationalLogRecord::new(ACTION_PRIVILEGED_EXPORT, author, 10),
            OperationalLogRecord::new(
                ACTION_ORDINARY_DIAGNOSIS,
                AnalyticalSubject::from_opaque(22),
                11,
            ),
        ];
        assert_eq!(truth[0].action_code(), ACTION_PRIVILEGED_EXPORT);
        assert_eq!(truth[0].analytical_subject(), author);
        assert_eq!(truth[0].system_time_seconds(), 10);
        assert_eq!(author.as_u128(), 11);
        let replayed = replay_operational_log(&truth).expect("replay");
        let matched = log_recovery_rate(&truth, &replayed).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            try_record(
                ACTION_PRIVILEGED_EXPORT,
                author,
                10,
                Some("source"),
                None,
                false,
            ),
            Err(OperationalLogError::SourceTextNotLoggable)
        );
        assert_eq!(
            try_record(
                ACTION_PRIVILEGED_EXPORT,
                author,
                10,
                None,
                Some("identity"),
                false,
            ),
            Err(OperationalLogError::SourceIdentityNotLoggable)
        );
        assert_eq!(
            try_record(ACTION_PRIVILEGED_EXPORT, author, 10, None, None, true),
            Err(OperationalLogError::BlanketMaskIsNotAuthorization)
        );
        try_record(ACTION_PRIVILEGED_EXPORT, author, 10, None, None, false).expect("opaque line");
        assert_eq!(
            refuse_source_text_in_log("source"),
            Err(OperationalLogError::SourceTextNotLoggable)
        );
        assert_eq!(
            refuse_source_identity_in_log("identity"),
            Err(OperationalLogError::SourceIdentityNotLoggable)
        );
        assert_eq!(
            refuse_source_identity_as_subject("identity"),
            Err(OperationalLogError::SourceIdentityNotLoggable)
        );
        assert_eq!(
            refuse_blanket_mask_as_log_grant(true),
            Err(OperationalLogError::BlanketMaskIsNotAuthorization)
        );
        refuse_blanket_mask_as_log_grant(false).expect("clear");
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
        let mismatched = [truth[0]];
        let partial = log_recovery_rate(&truth, &[truth[0], mismatched[0]]).expect("partial");
        assert!(partial < 1.0);
    }
}
