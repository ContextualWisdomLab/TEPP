//! Clock-family identity for event/valid-time stamps.

use crate::EventClockError;

/// Closed vocabulary of clocks that must not be confused with event time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockFamily {
    /// Event/valid time.
    EventTime,
    /// Assertion time.
    AssertionTime,
    /// System/record time.
    SystemTime,
    /// Document creation or revision time.
    DocumentTime,
    /// Availability time.
    AvailableTime,
}

/// Return whether a stamp is on the event clock.
///
/// # Errors
///
/// This function is infallible for the closed vocabulary and exists to keep
/// the public comparison surface explicit.
#[allow(clippy::unnecessary_wraps)]
pub fn stamp_is_event(family: ClockFamily) -> Result<bool, EventClockError> {
    Ok(matches!(family, ClockFamily::EventTime))
}

/// Refuse to treat assertion time as event time.
///
/// # Errors
///
/// Always returns [`EventClockError::AssertionTimeIsNotEventTime`].
pub fn refuse_assertion_time_as_event() -> Result<(), EventClockError> {
    Err(EventClockError::AssertionTimeIsNotEventTime)
}

/// Refuse to treat system time as event time.
///
/// # Errors
///
/// Always returns [`EventClockError::SystemTimeIsNotEventTime`].
pub fn refuse_system_time_as_event() -> Result<(), EventClockError> {
    Err(EventClockError::SystemTimeIsNotEventTime)
}

/// Refuse to treat document time as event time.
///
/// # Errors
///
/// Always returns [`EventClockError::DocumentTimeIsNotEventTime`].
pub fn refuse_document_time_as_event() -> Result<(), EventClockError> {
    Err(EventClockError::DocumentTimeIsNotEventTime)
}

/// Refuse to treat availability time as event time.
///
/// # Errors
///
/// Always returns [`EventClockError::AvailableTimeIsNotEventTime`].
pub fn refuse_available_time_as_event() -> Result<(), EventClockError> {
    Err(EventClockError::AvailableTimeIsNotEventTime)
}

/// Fraction of recovered event flags that match known truth.
///
/// # Errors
///
/// Returns [`EventClockError::InvalidEventPayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(truth: &[bool], decided: &[bool]) -> Result<f64, EventClockError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(EventClockError::InvalidEventPayload);
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
        ClockFamily, identity_recovery_rate, refuse_assertion_time_as_event,
        refuse_available_time_as_event, refuse_document_time_as_event, refuse_system_time_as_event,
        stamp_is_event,
    };
    use crate::EventClockError;

    #[test]
    fn local_branches_cover_families_and_payloads() {
        assert!(stamp_is_event(ClockFamily::EventTime).expect("event"));
        assert!(!stamp_is_event(ClockFamily::AssertionTime).expect("assertion"));
        assert!(!stamp_is_event(ClockFamily::SystemTime).expect("system"));
        assert!(!stamp_is_event(ClockFamily::DocumentTime).expect("document"));
        assert!(!stamp_is_event(ClockFamily::AvailableTime).expect("available"));
        assert_eq!(
            refuse_assertion_time_as_event(),
            Err(EventClockError::AssertionTimeIsNotEventTime)
        );
        assert_eq!(
            refuse_system_time_as_event(),
            Err(EventClockError::SystemTimeIsNotEventTime)
        );
        assert_eq!(
            refuse_document_time_as_event(),
            Err(EventClockError::DocumentTimeIsNotEventTime)
        );
        assert_eq!(
            refuse_available_time_as_event(),
            Err(EventClockError::AvailableTimeIsNotEventTime)
        );
        let matched = identity_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(EventClockError::InvalidEventPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[true], &[]),
            Err(EventClockError::InvalidEventPayload)
        );
    }
}
