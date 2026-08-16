//! Sensitivity inheritance for derived topic, factor, and relation artifacts.

use crate::DerivedSensitivityError;

/// Closed sensitivity vocabulary for source and derived artifacts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SensitivityClass {
    /// Ordinary compute may use the artifact under an authorized purpose.
    Internal,
    /// Re-identification or privileged access is required.
    Restricted,
    /// Explicitly public after an independent declassification decision.
    Public,
}

/// One derived topic, factor, or relation artifact with inherited sensitivity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedArtifact {
    kind_code: u16,
    source_class: SensitivityClass,
}

impl DerivedArtifact {
    /// Bind a derived-artifact kind to the source sensitivity class.
    #[must_use]
    pub const fn new(kind_code: u16, source_class: SensitivityClass) -> Self {
        Self {
            kind_code,
            source_class,
        }
    }

    /// Closed kind code (topic, factor, or relation), never a public default.
    #[must_use]
    pub const fn kind_code(self) -> u16 {
        self.kind_code
    }

    /// Inherited source sensitivity class.
    #[must_use]
    pub const fn source_class(self) -> SensitivityClass {
        self.source_class
    }
}

/// Inherit the source sensitivity class onto a derived artifact.
///
/// # Errors
///
/// Returns [`DerivedSensitivityError::DerivationIsNotDeclassification`] when
/// the source class is not public and a caller would treat derivation as a
/// public default. This function never upgrades Restricted or Internal to
/// Public.
pub fn inherit_sensitivity(
    source_class: SensitivityClass,
    kind_code: u16,
) -> Result<DerivedArtifact, DerivedSensitivityError> {
    if matches!(source_class, SensitivityClass::Public) {
        return Ok(DerivedArtifact::new(kind_code, SensitivityClass::Public));
    }
    Ok(DerivedArtifact::new(kind_code, source_class))
}

/// Refuse to treat derivation as declassification to public.
///
/// # Errors
///
/// Returns [`DerivedSensitivityError::DerivationIsNotDeclassification`] when
/// the source class is Restricted or Internal.
pub fn refuse_derivation_as_public(
    source_class: SensitivityClass,
) -> Result<(), DerivedSensitivityError> {
    if matches!(source_class, SensitivityClass::Public) {
        return Ok(());
    }
    Err(DerivedSensitivityError::DerivationIsNotDeclassification)
}

/// Refuse to treat a blanket PII mask as declassification authorization.
///
/// # Errors
///
/// Always returns [`DerivedSensitivityError::BlanketMaskIsNotAuthorization`].
pub fn refuse_blanket_mask_as_declassification() -> Result<(), DerivedSensitivityError> {
    Err(DerivedSensitivityError::BlanketMaskIsNotAuthorization)
}

/// Fraction of inherited classes that match known truth.
///
/// # Errors
///
/// Returns [`DerivedSensitivityError::InvalidSensitivityPayload`] when either
/// slice is empty or the lengths differ.
pub fn sensitivity_recovery_rate(
    truth: &[DerivedArtifact],
    decided: &[DerivedArtifact],
) -> Result<f64, DerivedSensitivityError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(DerivedSensitivityError::InvalidSensitivityPayload);
    }
    let mut matches = 0_u32;
    for (truth_record, decided_record) in truth.iter().zip(decided) {
        if truth_record == decided_record {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        DerivedArtifact, SensitivityClass, inherit_sensitivity,
        refuse_blanket_mask_as_declassification, refuse_derivation_as_public,
        sensitivity_recovery_rate,
    };
    use crate::DerivedSensitivityError;

    #[test]
    fn local_branches_cover_inherit_and_fail_closed_paths() {
        let restricted = inherit_sensitivity(SensitivityClass::Restricted, 1).expect("restricted");
        assert_eq!(restricted.kind_code(), 1);
        assert_eq!(restricted.source_class(), SensitivityClass::Restricted);
        let public = inherit_sensitivity(SensitivityClass::Public, 2).expect("public");
        assert_eq!(public.source_class(), SensitivityClass::Public);
        refuse_derivation_as_public(SensitivityClass::Public).expect("already public");
        assert_eq!(
            refuse_derivation_as_public(SensitivityClass::Internal),
            Err(DerivedSensitivityError::DerivationIsNotDeclassification)
        );
        assert_eq!(
            refuse_blanket_mask_as_declassification(),
            Err(DerivedSensitivityError::BlanketMaskIsNotAuthorization)
        );
        let truth = [DerivedArtifact::new(1, SensitivityClass::Restricted)];
        let matched = sensitivity_recovery_rate(&truth, &truth).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            sensitivity_recovery_rate(&[], &[]),
            Err(DerivedSensitivityError::InvalidSensitivityPayload)
        );
        assert_eq!(
            sensitivity_recovery_rate(&truth, &[]),
            Err(DerivedSensitivityError::InvalidSensitivityPayload)
        );
    }
}
