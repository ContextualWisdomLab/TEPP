//! Observed versus inferred relation evidence status.

use crate::InferredStatusError;

/// Closed vocabulary of presence evidence that is not yet a transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceStatus {
    /// Directly observed in source documents or authoritative systems.
    Observed,
    /// Derived by a model, reasoner, or heuristic and not yet promoted.
    Inferred,
}

impl EvidenceStatus {
    /// Return the stable wire status name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::Inferred => "inferred",
        }
    }

    /// Parse a stable wire status name.
    ///
    /// # Errors
    ///
    /// Returns [`InferredStatusError::InvalidStatusPayload`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, InferredStatusError> {
        match name {
            "observed" => Ok(Self::Observed),
            "inferred" => Ok(Self::Inferred),
            _ => Err(InferredStatusError::InvalidStatusPayload),
        }
    }
}

/// Return whether a status is observed evidence.
///
/// # Errors
///
/// This function is infallible for the closed vocabulary and exists to keep
/// the public comparison surface explicit.
#[allow(clippy::unnecessary_wraps)]
pub fn status_is_observed(status: EvidenceStatus) -> Result<bool, InferredStatusError> {
    Ok(matches!(status, EvidenceStatus::Observed))
}

/// Refuse to treat an inferred relation as observed evidence.
///
/// # Errors
///
/// Returns [`InferredStatusError::InferredIsNotObserved`] when `status` is
/// [`EvidenceStatus::Inferred`].
pub fn refuse_inferred_as_observed(status: EvidenceStatus) -> Result<(), InferredStatusError> {
    match status {
        EvidenceStatus::Inferred => Err(InferredStatusError::InferredIsNotObserved),
        EvidenceStatus::Observed => Ok(()),
    }
}

/// Refuse to treat an inferred relation as a state transition.
///
/// # Errors
///
/// Returns [`InferredStatusError::InferredIsNotTransition`] when `status` is
/// [`EvidenceStatus::Inferred`].
pub fn refuse_inferred_as_transition(status: EvidenceStatus) -> Result<(), InferredStatusError> {
    match status {
        EvidenceStatus::Inferred => Err(InferredStatusError::InferredIsNotTransition),
        EvidenceStatus::Observed => Ok(()),
    }
}

/// Fraction of recovered evidence statuses that match known truth.
///
/// # Errors
///
/// Returns [`InferredStatusError::InvalidStatusPayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[EvidenceStatus],
    decided: &[EvidenceStatus],
) -> Result<f64, InferredStatusError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(InferredStatusError::InvalidStatusPayload);
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
    use super::{
        identity_recovery_rate, refuse_inferred_as_observed, refuse_inferred_as_transition,
        status_is_observed, EvidenceStatus,
    };
    use crate::InferredStatusError;

    #[test]
    fn local_branches_cover_statuses_payloads_and_wire_names() {
        assert_eq!(
            refuse_inferred_as_observed(EvidenceStatus::Inferred),
            Err(InferredStatusError::InferredIsNotObserved)
        );
        assert_eq!(
            refuse_inferred_as_transition(EvidenceStatus::Inferred),
            Err(InferredStatusError::InferredIsNotTransition)
        );
        refuse_inferred_as_observed(EvidenceStatus::Observed).expect("observed");
        refuse_inferred_as_transition(EvidenceStatus::Observed).expect("observed");
        assert!(status_is_observed(EvidenceStatus::Observed).expect("observed"));
        assert!(!status_is_observed(EvidenceStatus::Inferred).expect("inferred"));
        for status in [EvidenceStatus::Observed, EvidenceStatus::Inferred] {
            assert_eq!(
                EvidenceStatus::from_wire_name(status.wire_name()).expect("round-trip"),
                status
            );
        }
        assert_eq!(
            EvidenceStatus::from_wire_name("promoted"),
            Err(InferredStatusError::InvalidStatusPayload)
        );
        let matched =
            identity_recovery_rate(&[EvidenceStatus::Inferred], &[EvidenceStatus::Inferred])
                .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(InferredStatusError::InvalidStatusPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[EvidenceStatus::Inferred], &[]),
            Err(InferredStatusError::InvalidStatusPayload)
        );
    }
}
