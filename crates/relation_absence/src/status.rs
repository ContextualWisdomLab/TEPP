//! Observed, inferred, and unobserved relation statuses stay distinct.

use crate::RelationAbsenceError;

/// Closed vocabulary of relation observation statuses.
///
/// Unobserved is a missing-status, not a negative edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationStatus {
    /// Directly observed in source documents or authoritative systems.
    Observed,
    /// Derived by a model, reasoner, or heuristic and not yet promoted.
    Inferred,
    /// No observed or inferred evidence exists for this pair.
    Unobserved,
}

impl ObservationStatus {
    /// Return the stable wire status name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::Inferred => "inferred",
            Self::Unobserved => "unobserved",
        }
    }

    /// Parse a stable wire status name.
    ///
    /// # Errors
    ///
    /// Returns [`RelationAbsenceError::InvalidObservationPayload`] for
    /// unrecognized names, including `no_relationship`.
    pub fn from_wire_name(name: &str) -> Result<Self, RelationAbsenceError> {
        match name {
            "observed" => Ok(Self::Observed),
            "inferred" => Ok(Self::Inferred),
            "unobserved" => Ok(Self::Unobserved),
            _ => Err(RelationAbsenceError::InvalidObservationPayload),
        }
    }
}

/// Refuse to treat an unobserved pair as evidence of no relationship.
///
/// Observed and inferred statuses are presence evidence. They are not
/// absence, so this gate lets them through.
///
/// # Errors
///
/// Returns [`RelationAbsenceError::AbsenceIsNotNegative`] when `status` is
/// [`ObservationStatus::Unobserved`].
pub fn refuse_absence_as_negative(status: ObservationStatus) -> Result<(), RelationAbsenceError> {
    match status {
        ObservationStatus::Unobserved => Err(RelationAbsenceError::AbsenceIsNotNegative),
        ObservationStatus::Observed | ObservationStatus::Inferred => Ok(()),
    }
}

/// Fraction of recovered observation statuses that match known truth.
///
/// # Errors
///
/// Returns [`RelationAbsenceError::InvalidObservationPayload`] when either
/// slice is empty or the lengths differ.
pub fn status_recovery_rate(
    truth: &[ObservationStatus],
    decided: &[ObservationStatus],
) -> Result<f64, RelationAbsenceError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(RelationAbsenceError::InvalidObservationPayload);
    }
    let mut matches = 0_u32;
    for (truth_status, decided_status) in truth.iter().zip(decided) {
        if truth_status == decided_status {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{ObservationStatus, refuse_absence_as_negative, status_recovery_rate};
    use crate::RelationAbsenceError;

    #[test]
    fn local_branches_cover_statuses_payloads_and_wire_names() {
        assert_eq!(
            refuse_absence_as_negative(ObservationStatus::Unobserved),
            Err(RelationAbsenceError::AbsenceIsNotNegative)
        );
        refuse_absence_as_negative(ObservationStatus::Observed).expect("observed");
        refuse_absence_as_negative(ObservationStatus::Inferred).expect("inferred");
        for status in [
            ObservationStatus::Observed,
            ObservationStatus::Inferred,
            ObservationStatus::Unobserved,
        ] {
            assert_eq!(
                ObservationStatus::from_wire_name(status.wire_name()).expect("round-trip"),
                status
            );
        }
        assert_eq!(
            ObservationStatus::from_wire_name("no_relationship"),
            Err(RelationAbsenceError::InvalidObservationPayload)
        );
        let truth = [ObservationStatus::Observed, ObservationStatus::Unobserved];
        let matched = status_recovery_rate(&truth, &truth).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        let partial = status_recovery_rate(
            &truth,
            &[ObservationStatus::Observed, ObservationStatus::Observed],
        )
        .expect("partial");
        assert!((partial - 0.5).abs() < f64::EPSILON);
        assert_eq!(
            status_recovery_rate(&[], &[]),
            Err(RelationAbsenceError::InvalidObservationPayload)
        );
        assert_eq!(
            status_recovery_rate(&truth, &[]),
            Err(RelationAbsenceError::InvalidObservationPayload)
        );
    }
}
