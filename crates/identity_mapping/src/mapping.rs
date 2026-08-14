//! Opaque analytical identifiers and separately authorized export.

use crate::IdentityMappingError;

/// One opaque analytical identifier paired with a separately stored source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentityMapRecord {
    analytical_id: u128,
    source_identity: u128,
}

impl IdentityMapRecord {
    /// Bind an opaque analytical identifier to a protected source identity.
    #[must_use]
    pub const fn new(analytical_id: u128, source_identity: u128) -> Self {
        Self {
            analytical_id,
            source_identity,
        }
    }

    /// Opaque identifier used in ordinary compute artifacts.
    #[must_use]
    pub const fn analytical_id(self) -> u128 {
        self.analytical_id
    }

    /// Source identity that may be exported only under re-identification purpose.
    #[must_use]
    pub const fn source_identity(self) -> u128 {
        self.source_identity
    }
}

/// Closed purpose vocabulary for identity-mapping operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingPurpose {
    /// Ordinary psychometric or longitudinal computation.
    AnalyticalComputation,
    /// Explicit re-identification export of the protected mapping.
    ReidentificationExport,
}

/// Export source identities only when the purpose is re-identification.
///
/// # Errors
///
/// Returns [`IdentityMappingError::UnauthorizedReidentification`] when the
/// purpose is analytical, or
/// [`IdentityMappingError::InvalidMappingPayload`] when no records are
/// supplied.
pub fn export_reidentification(
    records: &[IdentityMapRecord],
    purpose: MappingPurpose,
) -> Result<Vec<IdentityMapRecord>, IdentityMappingError> {
    if matches!(purpose, MappingPurpose::AnalyticalComputation) {
        return Err(IdentityMappingError::UnauthorizedReidentification);
    }
    if records.is_empty() {
        return Err(IdentityMappingError::InvalidMappingPayload);
    }
    Ok(records.to_vec())
}

/// Refuse to treat a blanket PII mask as re-identification authorization.
///
/// # Errors
///
/// Always returns [`IdentityMappingError::BlanketMaskIsNotAuthorization`].
pub fn refuse_blanket_mask_as_reidentification() -> Result<(), IdentityMappingError> {
    Err(IdentityMappingError::BlanketMaskIsNotAuthorization)
}

/// Fraction of recovered mapping pairs that match known truth.
///
/// # Errors
///
/// Returns [`IdentityMappingError::InvalidMappingPayload`] when either slice
/// is empty or the lengths differ.
pub fn mapping_recovery_rate(
    truth: &[IdentityMapRecord],
    decided: &[IdentityMapRecord],
) -> Result<f64, IdentityMappingError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(IdentityMappingError::InvalidMappingPayload);
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
        IdentityMapRecord, MappingPurpose, export_reidentification, mapping_recovery_rate,
        refuse_blanket_mask_as_reidentification,
    };
    use crate::IdentityMappingError;

    #[test]
    fn local_branches_cover_authorized_and_fail_closed_paths() {
        let truth = [IdentityMapRecord::new(1, 11), IdentityMapRecord::new(2, 22)];
        assert_eq!(truth[0].analytical_id(), 1);
        assert_eq!(truth[0].source_identity(), 11);
        let exported = export_reidentification(&truth, MappingPurpose::ReidentificationExport)
            .expect("authorized");
        let matched = mapping_recovery_rate(&truth, &exported).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            export_reidentification(&truth, MappingPurpose::AnalyticalComputation),
            Err(IdentityMappingError::UnauthorizedReidentification)
        );
        assert_eq!(
            refuse_blanket_mask_as_reidentification(),
            Err(IdentityMappingError::BlanketMaskIsNotAuthorization)
        );
        assert_eq!(
            mapping_recovery_rate(&[], &[]),
            Err(IdentityMappingError::InvalidMappingPayload)
        );
        assert_eq!(
            mapping_recovery_rate(&truth, &[]),
            Err(IdentityMappingError::InvalidMappingPayload)
        );
        assert_eq!(
            export_reidentification(&[], MappingPurpose::ReidentificationExport),
            Err(IdentityMappingError::InvalidMappingPayload)
        );
    }
}
