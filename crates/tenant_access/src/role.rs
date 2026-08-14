//! Closed access-role vocabulary, distinct from scientific membership roles.

use crate::TenantAccessError;

/// Closed access-control role vocabulary for protected operations.
///
/// These are authorization roles, not psychometric membership roles such as
/// author or department.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AccessRole {
    /// May submit or inspect analysis runs.
    AnalysisOperator,
    /// May request purpose-bound exports.
    ExportOfficer,
    /// May administer retention, deletion, and legal-hold requests.
    PrivacyOfficer,
    /// May read audit evidence without source-text export.
    Auditor,
}

impl AccessRole {
    /// Stable wire name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::AnalysisOperator => "analysis_operator",
            Self::ExportOfficer => "export_officer",
            Self::PrivacyOfficer => "privacy_officer",
            Self::Auditor => "auditor",
        }
    }

    /// Parse a stable wire role name.
    ///
    /// # Errors
    ///
    /// Returns [`TenantAccessError::UnknownAccessRole`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, TenantAccessError> {
        match name {
            "analysis_operator" => Ok(Self::AnalysisOperator),
            "export_officer" => Ok(Self::ExportOfficer),
            "privacy_officer" => Ok(Self::PrivacyOfficer),
            "auditor" => Ok(Self::Auditor),
            _ => Err(TenantAccessError::UnknownAccessRole),
        }
    }
}

/// Clock families that may evaluate an access lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessClock {
    /// TEPP system time at which the platform records the decision.
    System,
    /// Assertion time at which the grant itself was claimed.
    Assertion,
}

/// Parse the clock that may evaluate an access grant.
///
/// Event, document, availability, and knowledge-cutoff clocks cannot authorize
/// access. Those clocks measure scientific evidence, not grant lifetime
/// (ADR 0002/0009).
///
/// # Errors
///
/// Returns [`TenantAccessError::EventTimeCannotAuthorize`] for ineligible
/// scientific clocks and [`TenantAccessError::UnknownAccessClock`] for
/// unrecognized names.
pub fn access_clock_from_wire(name: &str) -> Result<AccessClock, TenantAccessError> {
    match name {
        "system_time" => Ok(AccessClock::System),
        "assertion_time" => Ok(AccessClock::Assertion),
        "event_time" | "document_time" | "available_time" | "knowledge_cutoff" => {
            Err(TenantAccessError::EventTimeCannotAuthorize)
        }
        _ => Err(TenantAccessError::UnknownAccessClock),
    }
}

/// Explicit refusal to treat blanket PII masking as tenant/role authorization.
///
/// # Errors
///
/// Always returns [`TenantAccessError::BlanketMaskIsNotAuthorization`].
pub fn refuse_blanket_mask_as_access() -> Result<(), TenantAccessError> {
    Err(TenantAccessError::BlanketMaskIsNotAuthorization)
}

/// Fraction of recovered tenant/role pairs that match known truth.
///
/// # Errors
///
/// Returns [`TenantAccessError::InvalidAccessPayload`] when either slice is
/// empty or the lengths differ.
pub fn tenant_role_recovery_rate(
    truth: &[(crate::TenantId, AccessRole)],
    decided: &[(crate::TenantId, AccessRole)],
) -> Result<f64, TenantAccessError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(TenantAccessError::InvalidAccessPayload);
    }
    let mut matches = 0_u32;
    for (truth_row, decided_row) in truth.iter().zip(decided) {
        if truth_row == decided_row {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{AccessClock, AccessRole, access_clock_from_wire, tenant_role_recovery_rate};
    use crate::TenantAccessError;

    #[test]
    fn role_and_clock_wire_names_round_trip() {
        for role in [
            AccessRole::AnalysisOperator,
            AccessRole::ExportOfficer,
            AccessRole::PrivacyOfficer,
            AccessRole::Auditor,
        ] {
            assert_eq!(
                AccessRole::from_wire_name(role.wire_name()).expect("round trip"),
                role
            );
        }
        assert_eq!(
            AccessRole::from_wire_name("author"),
            Err(TenantAccessError::UnknownAccessRole)
        );
        assert_eq!(
            access_clock_from_wire("system_time"),
            Ok(AccessClock::System)
        );
        assert_eq!(
            access_clock_from_wire("assertion_time"),
            Ok(AccessClock::Assertion)
        );
        assert_eq!(
            tenant_role_recovery_rate(&[], &[]),
            Err(TenantAccessError::InvalidAccessPayload)
        );
    }
}
