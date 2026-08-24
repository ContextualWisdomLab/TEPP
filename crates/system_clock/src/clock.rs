//! Clock-family identity for system stamps.

use crate::SystemClockError;

/// Closed vocabulary of clocks that must not be confused with system time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockFamily {
    /// Event/valid time.
    EventTime,
    /// Assertion time.
    AssertionTime,
    /// Document creation or revision time.
    DocumentTime,
    /// Availability time.
    AvailableTime,
    /// Knowledge-cutoff time.
    CutoffTime,
    /// System/record time.
    SystemTime,
}

/// Return whether a stamp is on the system clock.
///
/// # Errors
///
/// This function is infallible for the closed vocabulary and exists to keep
/// the public comparison surface explicit.
#[allow(clippy::unnecessary_wraps)]
pub fn stamp_is_system(family: ClockFamily) -> Result<bool, SystemClockError> {
    Ok(matches!(family, ClockFamily::SystemTime))
}

/// Refuse to treat event time as system time.
///
/// # Errors
///
/// Always returns [`SystemClockError::EventTimeIsNotSystemTime`].
pub fn refuse_event_time_as_system() -> Result<(), SystemClockError> {
    Err(SystemClockError::EventTimeIsNotSystemTime)
}

/// Refuse to treat assertion time as system time.
///
/// # Errors
///
/// Always returns [`SystemClockError::AssertionTimeIsNotSystemTime`].
pub fn refuse_assertion_time_as_system() -> Result<(), SystemClockError> {
    Err(SystemClockError::AssertionTimeIsNotSystemTime)
}

/// Refuse to treat document time as system time.
///
/// # Errors
///
/// Always returns [`SystemClockError::DocumentTimeIsNotSystemTime`].
pub fn refuse_document_time_as_system() -> Result<(), SystemClockError> {
    Err(SystemClockError::DocumentTimeIsNotSystemTime)
}

/// Refuse to treat availability time as system time.
///
/// # Errors
///
/// Always returns [`SystemClockError::AvailableTimeIsNotSystemTime`].
pub fn refuse_available_time_as_system() -> Result<(), SystemClockError> {
    Err(SystemClockError::AvailableTimeIsNotSystemTime)
}

/// Refuse to treat knowledge-cutoff time as system time.
///
/// # Errors
///
/// Always returns [`SystemClockError::CutoffTimeIsNotSystemTime`].
pub fn refuse_cutoff_time_as_system() -> Result<(), SystemClockError> {
    Err(SystemClockError::CutoffTimeIsNotSystemTime)
}

/// Fraction of recovered system-clock flags that match known truth.
///
/// # Errors
///
/// Returns [`SystemClockError::InvalidSystemPayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(truth: &[bool], decided: &[bool]) -> Result<f64, SystemClockError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(SystemClockError::InvalidSystemPayload);
    }
    let mut matches = 0_usize;
    for (truth_flag, decided_flag) in truth.iter().zip(decided) {
        if truth_flag == decided_flag {
            matches += 1;
        }
    }
    Ok(matches as f64 / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        ClockFamily, identity_recovery_rate, refuse_assertion_time_as_system,
        refuse_available_time_as_system, refuse_cutoff_time_as_system,
        refuse_document_time_as_system, refuse_event_time_as_system, stamp_is_system,
    };
    use crate::SystemClockError;

    #[test]
    fn local_branches_cover_families_and_payloads() {
        assert!(stamp_is_system(ClockFamily::SystemTime).expect("system"));
        assert!(!stamp_is_system(ClockFamily::EventTime).expect("event"));
        assert!(!stamp_is_system(ClockFamily::AssertionTime).expect("assertion"));
        assert!(!stamp_is_system(ClockFamily::DocumentTime).expect("document"));
        assert!(!stamp_is_system(ClockFamily::AvailableTime).expect("available"));
        assert!(!stamp_is_system(ClockFamily::CutoffTime).expect("cutoff"));
        assert_eq!(
            refuse_event_time_as_system(),
            Err(SystemClockError::EventTimeIsNotSystemTime)
        );
        assert_eq!(
            refuse_assertion_time_as_system(),
            Err(SystemClockError::AssertionTimeIsNotSystemTime)
        );
        assert_eq!(
            refuse_document_time_as_system(),
            Err(SystemClockError::DocumentTimeIsNotSystemTime)
        );
        assert_eq!(
            refuse_available_time_as_system(),
            Err(SystemClockError::AvailableTimeIsNotSystemTime)
        );
        assert_eq!(
            refuse_cutoff_time_as_system(),
            Err(SystemClockError::CutoffTimeIsNotSystemTime)
        );
        let matched = identity_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(SystemClockError::InvalidSystemPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[true], &[]),
            Err(SystemClockError::InvalidSystemPayload)
        );
    }
}
