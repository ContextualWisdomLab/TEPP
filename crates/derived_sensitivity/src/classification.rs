//! Sensitivity inheritance for derived topic, factor, and relation artifacts.

use crate::DerivedSensitivityError;

/// Closed derived-artifact kind: topic proportion or topic identity.
pub const KIND_TOPIC: u16 = 1;
/// Closed derived-artifact kind: factor score or loading.
pub const KIND_FACTOR: u16 = 2;
/// Closed derived-artifact kind: relation or graph edge.
pub const KIND_RELATION: u16 = 3;

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
    /// Bind a closed derived-artifact kind to the source sensitivity class.
    ///
    /// # Errors
    ///
    /// Returns [`DerivedSensitivityError::InvalidSensitivityPayload`] when
    /// `kind_code` is not topic, factor, or relation.
    pub const fn try_new(
        kind_code: u16,
        source_class: SensitivityClass,
    ) -> Result<Self, DerivedSensitivityError> {
        if !matches!(kind_code, KIND_TOPIC | KIND_FACTOR | KIND_RELATION) {
            return Err(DerivedSensitivityError::InvalidSensitivityPayload);
        }
        Ok(Self {
            kind_code,
            source_class,
        })
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
/// Returns [`DerivedSensitivityError::InvalidSensitivityPayload`] when
/// `kind_code` is not topic, factor, or relation. This function never
/// upgrades Restricted or Internal to Public.
pub fn inherit_sensitivity(
    source_class: SensitivityClass,
    kind_code: u16,
) -> Result<DerivedArtifact, DerivedSensitivityError> {
    DerivedArtifact::try_new(kind_code, source_class)
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

/// Fraction of paired records whose inherited sensitivity class matches known
/// truth. Kind identity is validated at construction but is not part of this
/// class-recovery estimand.
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
        if truth_record.source_class() == decided_record.source_class() {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        DerivedArtifact, KIND_FACTOR, KIND_RELATION, KIND_TOPIC, SensitivityClass,
        inherit_sensitivity, refuse_blanket_mask_as_declassification, refuse_derivation_as_public,
        sensitivity_recovery_rate,
    };
    use crate::DerivedSensitivityError;

    #[test]
    fn local_branches_cover_inherit_and_fail_closed_paths() {
        let restricted =
            inherit_sensitivity(SensitivityClass::Restricted, KIND_TOPIC).expect("restricted");
        assert_eq!(restricted.kind_code(), KIND_TOPIC);
        assert_eq!(restricted.source_class(), SensitivityClass::Restricted);
        let public = inherit_sensitivity(SensitivityClass::Public, KIND_FACTOR).expect("public");
        assert_eq!(public.source_class(), SensitivityClass::Public);
        let internal =
            inherit_sensitivity(SensitivityClass::Internal, KIND_RELATION).expect("internal");
        assert_eq!(internal.source_class(), SensitivityClass::Internal);
        assert_eq!(
            inherit_sensitivity(SensitivityClass::Restricted, 99),
            Err(DerivedSensitivityError::InvalidSensitivityPayload)
        );
        refuse_derivation_as_public(SensitivityClass::Public).expect("already public");
        assert_eq!(
            refuse_derivation_as_public(SensitivityClass::Internal),
            Err(DerivedSensitivityError::DerivationIsNotDeclassification)
        );
        assert_eq!(
            refuse_blanket_mask_as_declassification(),
            Err(DerivedSensitivityError::BlanketMaskIsNotAuthorization)
        );
        let truth = [DerivedArtifact::try_new(1, SensitivityClass::Restricted).expect("topic")];
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
