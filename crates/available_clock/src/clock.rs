//! Clock-family identity for availability stamps.

use crate::AvailableClockError;

/// Closed vocabulary of clocks that must not be confused with availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockFamily {
    /// Event/valid time.
    EventTime,
    /// System/record time.
    SystemTime,
    /// Availability time.
    AvailableTime,
}

/// Return whether a stamp is on the availability clock.
///
/// # Errors
///
/// This function is infallible for the closed vocabulary and exists to keep
/// the public comparison surface explicit.
#[allow(clippy::unnecessary_wraps)]
pub fn stamp_is_available(family: ClockFamily) -> Result<bool, AvailableClockError> {
    Ok(matches!(family, ClockFamily::AvailableTime))
}

/// Refuse to treat event time as availability time.
///
/// # Errors
///
/// Always returns [`AvailableClockError::EventTimeIsNotAvailableTime`].
pub fn refuse_event_time_as_available() -> Result<(), AvailableClockError> {
    Err(AvailableClockError::EventTimeIsNotAvailableTime)
}

/// Refuse to treat system time as availability time.
///
/// # Errors
///
/// Always returns [`AvailableClockError::SystemTimeIsNotAvailableTime`].
pub fn refuse_system_time_as_available() -> Result<(), AvailableClockError> {
    Err(AvailableClockError::SystemTimeIsNotAvailableTime)
}

/// Fraction of recovered availability flags that match known truth.
///
/// # Errors
///
/// Returns [`AvailableClockError::InvalidAvailabilityPayload`] when either
/// slice is empty or the lengths differ.
pub fn eligibility_recovery_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, AvailableClockError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(AvailableClockError::InvalidAvailabilityPayload);
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
        ClockFamily, eligibility_recovery_rate, refuse_event_time_as_available,
        refuse_system_time_as_available, stamp_is_available,
    };
    use crate::AvailableClockError;

    #[test]
    fn local_branches_cover_families_and_payloads() {
        assert!(stamp_is_available(ClockFamily::AvailableTime).expect("available"));
        assert!(!stamp_is_available(ClockFamily::EventTime).expect("event"));
        assert!(!stamp_is_available(ClockFamily::SystemTime).expect("system"));
        assert_eq!(
            refuse_event_time_as_available(),
            Err(AvailableClockError::EventTimeIsNotAvailableTime)
        );
        assert_eq!(
            refuse_system_time_as_available(),
            Err(AvailableClockError::SystemTimeIsNotAvailableTime)
        );
        let matched = eligibility_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            eligibility_recovery_rate(&[], &[]),
            Err(AvailableClockError::InvalidAvailabilityPayload)
        );
        assert_eq!(
            eligibility_recovery_rate(&[true], &[]),
            Err(AvailableClockError::InvalidAvailabilityPayload)
        );
    }
}
