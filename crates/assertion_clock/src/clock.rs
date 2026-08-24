//! Clock-family identity for assertion stamps.

use crate::AssertionClockError;

/// Closed vocabulary of clocks that must not be confused with assertion time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockFamily {
    /// Event/valid time.
    EventTime,
    /// System/record time.
    SystemTime,
    /// Document creation or revision time.
    DocumentTime,
    /// Availability time.
    AvailableTime,
    /// Assertion time.
    AssertionTime,
}

/// Return whether a stamp is on the assertion clock.
///
/// # Errors
///
/// This function is infallible for the closed vocabulary and exists to keep
/// the public comparison surface explicit.
#[allow(clippy::unnecessary_wraps)]
pub fn stamp_is_assertion(family: ClockFamily) -> Result<bool, AssertionClockError> {
    Ok(matches!(family, ClockFamily::AssertionTime))
}

/// Refuse to treat event time as assertion time.
///
/// # Errors
///
/// Always returns [`AssertionClockError::EventTimeIsNotAssertionTime`].
pub fn refuse_event_time_as_assertion() -> Result<(), AssertionClockError> {
    Err(AssertionClockError::EventTimeIsNotAssertionTime)
}

/// Refuse to treat system time as assertion time.
///
/// # Errors
///
/// Always returns [`AssertionClockError::SystemTimeIsNotAssertionTime`].
pub fn refuse_system_time_as_assertion() -> Result<(), AssertionClockError> {
    Err(AssertionClockError::SystemTimeIsNotAssertionTime)
}

/// Refuse to treat document time as assertion time.
///
/// # Errors
///
/// Always returns [`AssertionClockError::DocumentTimeIsNotAssertionTime`].
pub fn refuse_document_time_as_assertion() -> Result<(), AssertionClockError> {
    Err(AssertionClockError::DocumentTimeIsNotAssertionTime)
}

/// Refuse to treat availability time as assertion time.
///
/// # Errors
///
/// Always returns [`AssertionClockError::AvailableTimeIsNotAssertionTime`].
pub fn refuse_available_time_as_assertion() -> Result<(), AssertionClockError> {
    Err(AssertionClockError::AvailableTimeIsNotAssertionTime)
}

/// Fraction of recovered assertion flags that match known truth.
///
/// # Errors
///
/// Returns [`AssertionClockError::InvalidAssertionPayload`] when either slice
/// is empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, AssertionClockError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(AssertionClockError::InvalidAssertionPayload);
    }
    let mut matches = 0_u32;
    for (truth_flag, decided_flag) in truth.iter().zip(decided) {
        if truth_flag == decided_flag {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        ClockFamily, identity_recovery_rate, refuse_available_time_as_assertion,
        refuse_document_time_as_assertion, refuse_event_time_as_assertion,
        refuse_system_time_as_assertion, stamp_is_assertion,
    };
    use crate::AssertionClockError;

    #[test]
    fn local_branches_cover_families_and_payloads() {
        assert!(stamp_is_assertion(ClockFamily::AssertionTime).expect("assertion"));
        assert!(!stamp_is_assertion(ClockFamily::EventTime).expect("event"));
        assert!(!stamp_is_assertion(ClockFamily::SystemTime).expect("system"));
        assert!(!stamp_is_assertion(ClockFamily::DocumentTime).expect("document"));
        assert!(!stamp_is_assertion(ClockFamily::AvailableTime).expect("available"));
        assert_eq!(
            refuse_event_time_as_assertion(),
            Err(AssertionClockError::EventTimeIsNotAssertionTime)
        );
        assert_eq!(
            refuse_system_time_as_assertion(),
            Err(AssertionClockError::SystemTimeIsNotAssertionTime)
        );
        assert_eq!(
            refuse_document_time_as_assertion(),
            Err(AssertionClockError::DocumentTimeIsNotAssertionTime)
        );
        assert_eq!(
            refuse_available_time_as_assertion(),
            Err(AssertionClockError::AvailableTimeIsNotAssertionTime)
        );
        let matched = identity_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(AssertionClockError::InvalidAssertionPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[true], &[]),
            Err(AssertionClockError::InvalidAssertionPayload)
        );
    }
}
