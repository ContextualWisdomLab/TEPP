//! Grant presence required at untrusted intake.

use crate::IntakeAuthorizationError;

/// Closed vocabulary of untrusted inbound kinds that require a grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntakeKind {
    /// External document bytes.
    Document,
    /// Serialized domain or wire record.
    SerializedRecord,
    /// Model checkpoint or artifact bytes.
    ModelCheckpoint,
    /// LLM or agent output.
    LlmOutput,
}

impl IntakeKind {
    /// Return the stable wire intake-kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::SerializedRecord => "serialized_record",
            Self::ModelCheckpoint => "model_checkpoint",
            Self::LlmOutput => "llm_output",
        }
    }

    /// Parse a stable wire intake-kind name.
    ///
    /// # Errors
    ///
    /// Returns [`IntakeAuthorizationError::InvalidIntakePayload`] for
    /// unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, IntakeAuthorizationError> {
        match name {
            "document" => Ok(Self::Document),
            "serialized_record" => Ok(Self::SerializedRecord),
            "model_checkpoint" => Ok(Self::ModelCheckpoint),
            "llm_output" => Ok(Self::LlmOutput),
            _ => Err(IntakeAuthorizationError::InvalidIntakePayload),
        }
    }
}

/// Whether a purpose-bound grant is present at intake.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrantPresence {
    /// A grant exists for this intake.
    Present,
    /// No grant exists for this intake.
    Absent,
}

/// Refuse untrusted intake that has no purpose-bound grant.
///
/// Cross-purpose reuse of a present grant is owned by `purpose_authorization`.
/// Identity, provenance, size, and depth are owned by `payload_bound`.
///
/// # Errors
///
/// Returns [`IntakeAuthorizationError::MissingGrant`] when `grant` is
/// [`GrantPresence::Absent`].
pub fn refuse_intake_without_grant(
    kind: IntakeKind,
    grant: GrantPresence,
) -> Result<(), IntakeAuthorizationError> {
    let _ = kind.wire_name();
    match grant {
        GrantPresence::Absent => Err(IntakeAuthorizationError::MissingGrant),
        GrantPresence::Present => Ok(()),
    }
}

/// Refuse to treat size, identity, or provenance bounds as authorization.
///
/// # Errors
///
/// Always returns [`IntakeAuthorizationError::BoundsAreNotAuthorization`].
pub fn refuse_bounds_as_authorization() -> Result<(), IntakeAuthorizationError> {
    Err(IntakeAuthorizationError::BoundsAreNotAuthorization)
}

/// Fraction of recovered grant-presence flags that match known truth.
///
/// # Errors
///
/// Returns [`IntakeAuthorizationError::InvalidIntakePayload`] when either
/// slice is empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, IntakeAuthorizationError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(IntakeAuthorizationError::InvalidIntakePayload);
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
        identity_recovery_rate, refuse_bounds_as_authorization, refuse_intake_without_grant,
        GrantPresence, IntakeKind,
    };
    use crate::IntakeAuthorizationError;

    #[test]
    fn local_branches_cover_kinds_grants_and_payloads() {
        for kind in [
            IntakeKind::Document,
            IntakeKind::SerializedRecord,
            IntakeKind::ModelCheckpoint,
            IntakeKind::LlmOutput,
        ] {
            assert_eq!(
                refuse_intake_without_grant(kind, GrantPresence::Absent),
                Err(IntakeAuthorizationError::MissingGrant)
            );
            refuse_intake_without_grant(kind, GrantPresence::Present).expect("present");
            assert_eq!(
                IntakeKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            refuse_bounds_as_authorization(),
            Err(IntakeAuthorizationError::BoundsAreNotAuthorization)
        );
        assert_eq!(
            IntakeKind::from_wire_name("trusted"),
            Err(IntakeAuthorizationError::InvalidIntakePayload)
        );
        let matched = identity_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(IntakeAuthorizationError::InvalidIntakePayload)
        );
        assert_eq!(
            identity_recovery_rate(&[true], &[]),
            Err(IntakeAuthorizationError::InvalidIntakePayload)
        );
    }
}
