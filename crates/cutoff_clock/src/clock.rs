//! Clock-family identity for knowledge-cutoff stamps.

use crate::CutoffClockError;

/// Closed vocabulary of clocks that must not be confused with knowledge cutoff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockFamily {
    /// Event/valid time.
    EventTime,
    /// System/record time.
    SystemTime,
    /// Availability time.
    AvailableTime,
    /// Analysis knowledge cutoff.
    KnowledgeCutoff,
}

/// Return whether a stamp is on the knowledge-cutoff clock.
///
/// # Errors
///
/// This function is infallible for the closed vocabulary and exists to keep
/// the public comparison surface explicit.
#[allow(clippy::unnecessary_wraps)]
pub fn stamp_is_cutoff(family: ClockFamily) -> Result<bool, CutoffClockError> {
    Ok(matches!(family, ClockFamily::KnowledgeCutoff))
}

/// Refuse to treat event time as knowledge cutoff.
///
/// # Errors
///
/// Always returns [`CutoffClockError::EventTimeIsNotKnowledgeCutoff`].
pub fn refuse_event_time_as_cutoff() -> Result<(), CutoffClockError> {
    Err(CutoffClockError::EventTimeIsNotKnowledgeCutoff)
}

/// Refuse to treat system time as knowledge cutoff.
///
/// # Errors
///
/// Always returns [`CutoffClockError::SystemTimeIsNotKnowledgeCutoff`].
pub fn refuse_system_time_as_cutoff() -> Result<(), CutoffClockError> {
    Err(CutoffClockError::SystemTimeIsNotKnowledgeCutoff)
}

/// Refuse to treat availability time as knowledge cutoff.
///
/// # Errors
///
/// Always returns [`CutoffClockError::AvailableTimeIsNotKnowledgeCutoff`].
pub fn refuse_available_time_as_cutoff() -> Result<(), CutoffClockError> {
    Err(CutoffClockError::AvailableTimeIsNotKnowledgeCutoff)
}

/// Fraction of recovered cutoff flags that match known truth.
///
/// # Errors
///
/// Returns [`CutoffClockError::InvalidCutoffPayload`] when either slice is
/// empty or the lengths differ.
pub fn eligibility_recovery_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, CutoffClockError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(CutoffClockError::InvalidCutoffPayload);
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
        ClockFamily, eligibility_recovery_rate, refuse_available_time_as_cutoff,
        refuse_event_time_as_cutoff, refuse_system_time_as_cutoff, stamp_is_cutoff,
    };
    use crate::CutoffClockError;

    #[test]
    fn local_branches_cover_families_and_payloads() {
        assert!(stamp_is_cutoff(ClockFamily::KnowledgeCutoff).expect("cutoff"));
        assert!(!stamp_is_cutoff(ClockFamily::EventTime).expect("event"));
        assert!(!stamp_is_cutoff(ClockFamily::SystemTime).expect("system"));
        assert!(!stamp_is_cutoff(ClockFamily::AvailableTime).expect("available"));
        assert_eq!(
            refuse_event_time_as_cutoff(),
            Err(CutoffClockError::EventTimeIsNotKnowledgeCutoff)
        );
        assert_eq!(
            refuse_system_time_as_cutoff(),
            Err(CutoffClockError::SystemTimeIsNotKnowledgeCutoff)
        );
        assert_eq!(
            refuse_available_time_as_cutoff(),
            Err(CutoffClockError::AvailableTimeIsNotKnowledgeCutoff)
        );
        let matched = eligibility_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            eligibility_recovery_rate(&[], &[]),
            Err(CutoffClockError::InvalidCutoffPayload)
        );
        assert_eq!(
            eligibility_recovery_rate(&[true], &[]),
            Err(CutoffClockError::InvalidCutoffPayload)
        );
    }
}
