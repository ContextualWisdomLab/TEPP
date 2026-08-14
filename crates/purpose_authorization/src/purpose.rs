//! Closed processing-purpose vocabulary and recovery.

use crate::PurposeAuthorizationError;

/// Closed processing-purpose vocabulary bound to TEPP retention purposes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PurposeCode {
    /// Psychometric and statistical analysis.
    PsychometricAnalysis,
    /// Legal or contractual preservation.
    LegalPreservation,
    /// Operations and audit review.
    OperationsAudit,
    /// Authorized export fulfillment.
    ExportFulfillment,
}

impl PurposeCode {
    /// Stable wire name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::PsychometricAnalysis => "psychometric_analysis",
            Self::LegalPreservation => "legal_preservation",
            Self::OperationsAudit => "operations_audit",
            Self::ExportFulfillment => "export_fulfillment",
        }
    }

    /// Parse a stable wire purpose name.
    ///
    /// # Errors
    ///
    /// Returns [`PurposeAuthorizationError::UnknownPurpose`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, PurposeAuthorizationError> {
        match name {
            "psychometric_analysis" => Ok(Self::PsychometricAnalysis),
            "legal_preservation" => Ok(Self::LegalPreservation),
            "operations_audit" => Ok(Self::OperationsAudit),
            "export_fulfillment" => Ok(Self::ExportFulfillment),
            _ => Err(PurposeAuthorizationError::UnknownPurpose),
        }
    }
}

/// Fraction of recovered purposes that match known truth.
///
/// # Errors
///
/// Returns [`PurposeAuthorizationError::InvalidPurposePayload`] when either
/// slice is empty or the lengths differ.
pub fn purpose_recovery_rate(
    truth: &[PurposeCode],
    decided: &[PurposeCode],
) -> Result<f64, PurposeAuthorizationError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(PurposeAuthorizationError::InvalidPurposePayload);
    }
    let mut matches = 0_u32;
    for (truth_purpose, decided_purpose) in truth.iter().zip(decided) {
        if truth_purpose == decided_purpose {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

/// Explicit refusal to use a grant for a different purpose.
///
/// # Errors
///
/// Returns [`PurposeAuthorizationError::CrossPurposeUse`] when the purposes
/// differ.
pub fn refuse_cross_purpose_use(
    granted: PurposeCode,
    requested: PurposeCode,
) -> Result<(), PurposeAuthorizationError> {
    if granted == requested {
        Ok(())
    } else {
        Err(PurposeAuthorizationError::CrossPurposeUse)
    }
}

/// Explicit refusal to treat blanket PII masking as authorization.
///
/// # Errors
///
/// Always returns [`PurposeAuthorizationError::BlanketMaskIsNotAuthorization`].
pub fn refuse_blanket_mask_as_authorization() -> Result<(), PurposeAuthorizationError> {
    Err(PurposeAuthorizationError::BlanketMaskIsNotAuthorization)
}

#[cfg(test)]
mod tests {
    use super::{PurposeCode, purpose_recovery_rate};
    use crate::PurposeAuthorizationError;

    #[test]
    fn wire_names_round_trip() {
        for purpose in [
            PurposeCode::PsychometricAnalysis,
            PurposeCode::LegalPreservation,
            PurposeCode::OperationsAudit,
            PurposeCode::ExportFulfillment,
        ] {
            assert_eq!(
                PurposeCode::from_wire_name(purpose.wire_name()).expect("round trip"),
                purpose
            );
        }
        assert_eq!(
            PurposeCode::from_wire_name("marketing"),
            Err(PurposeAuthorizationError::UnknownPurpose)
        );
        assert_eq!(
            purpose_recovery_rate(&[PurposeCode::OperationsAudit], &[]),
            Err(PurposeAuthorizationError::InvalidPurposePayload)
        );
    }
}
