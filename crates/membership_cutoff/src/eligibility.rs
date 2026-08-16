//! Cutoff filters for multiple-membership observations.

use crate::MembershipCutoffError;

/// One membership observation stamped with availability time in seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MembershipObservation {
    unit_id: u128,
    available_seconds: i64,
}

impl MembershipObservation {
    /// Bind a membership unit to its availability time.
    #[must_use]
    pub const fn new(unit_id: u128, available_seconds: i64) -> Self {
        Self {
            unit_id,
            available_seconds,
        }
    }

    /// Opaque unit that holds the membership.
    #[must_use]
    pub const fn unit_id(self) -> u128 {
        self.unit_id
    }

    /// Availability time in seconds.
    #[must_use]
    pub const fn available_seconds(self) -> i64 {
        self.available_seconds
    }
}

/// Refuse a single membership whose availability exceeds the cutoff.
///
/// # Errors
///
/// Returns [`MembershipCutoffError::AvailabilityExceedsCutoff`] when
/// `available_seconds` is greater than `cutoff_seconds`.
pub fn refuse_membership_after_cutoff(
    available_seconds: i64,
    cutoff_seconds: i64,
) -> Result<(), MembershipCutoffError> {
    if available_seconds > cutoff_seconds {
        return Err(MembershipCutoffError::AvailabilityExceedsCutoff);
    }
    Ok(())
}

/// Keep memberships whose availability does not exceed the cutoff.
///
/// # Errors
///
/// Returns [`MembershipCutoffError::InvalidEligibilityPayload`] when no
/// observations are supplied.
pub fn eligible_memberships(
    observations: &[MembershipObservation],
    cutoff_seconds: i64,
) -> Result<Vec<MembershipObservation>, MembershipCutoffError> {
    if observations.is_empty() {
        return Err(MembershipCutoffError::InvalidEligibilityPayload);
    }
    Ok(observations
        .iter()
        .copied()
        .filter(|observation| observation.available_seconds <= cutoff_seconds)
        .collect())
}

/// Fraction of recovered eligibility flags that match known truth.
///
/// # Errors
///
/// Returns [`MembershipCutoffError::InvalidEligibilityPayload`] when either
/// slice is empty or the lengths differ.
pub fn eligibility_recovery_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, MembershipCutoffError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(MembershipCutoffError::InvalidEligibilityPayload);
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
        MembershipObservation, eligibility_recovery_rate, eligible_memberships,
        refuse_membership_after_cutoff,
    };
    use crate::MembershipCutoffError;

    #[test]
    fn local_branches_cover_filter_and_fail_closed_paths() {
        let on_cutoff = MembershipObservation::new(1, 10);
        assert_eq!(on_cutoff.unit_id(), 1);
        assert_eq!(on_cutoff.available_seconds(), 10);
        refuse_membership_after_cutoff(10, 10).expect("on cutoff");
        assert_eq!(
            refuse_membership_after_cutoff(11, 10),
            Err(MembershipCutoffError::AvailabilityExceedsCutoff)
        );
        let observations = [
            MembershipObservation::new(1, 9),
            MembershipObservation::new(2, 11),
        ];
        let eligible = eligible_memberships(&observations, 10).expect("filter");
        assert_eq!(eligible, vec![MembershipObservation::new(1, 9)]);
        let matched = eligibility_recovery_rate(&[true, false], &[true, false]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            eligible_memberships(&[], 10),
            Err(MembershipCutoffError::InvalidEligibilityPayload)
        );
        assert_eq!(
            eligibility_recovery_rate(&[], &[]),
            Err(MembershipCutoffError::InvalidEligibilityPayload)
        );
        assert_eq!(
            eligibility_recovery_rate(&[true], &[]),
            Err(MembershipCutoffError::InvalidEligibilityPayload)
        );
    }
}
