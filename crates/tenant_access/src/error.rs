//! Fail-closed tenant, role, and lifetime errors.

use std::fmt;

/// A fail-closed tenant-access error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TenantAccessError {
    /// The request tenant is not the granted tenant.
    TenantMismatch,
    /// The request principal is not the grant holder.
    PrincipalMismatch,
    /// The requested access role is not on the grant.
    RoleNotGranted,
    /// The evaluation instant is before the grant start.
    NotYetValid,
    /// The evaluation instant is at or after the grant end.
    Expired,
    /// The lifetime window is inverted or zero-width.
    InvertedLifetime,
    /// Event, document, availability, or cutoff time was offered as access time.
    EventTimeCannotAuthorize,
    /// An unknown access-clock wire name was supplied.
    UnknownAccessClock,
    /// An unknown access-role wire name was supplied.
    UnknownAccessRole,
    /// Blanket PII masking was offered as a substitute for authorization.
    BlanketMaskIsNotAuthorization,
    /// No stored grant authorized the request.
    NoMatchingGrant,
    /// Grant or recovery slices were empty or length-mismatched.
    InvalidAccessPayload,
}

impl fmt::Display for TenantAccessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TenantMismatch => "access grant used for a different tenant",
            Self::PrincipalMismatch => "access grant used for a different principal",
            Self::RoleNotGranted => "access role is not granted",
            Self::NotYetValid => "access grant is not yet valid",
            Self::Expired => "access grant is expired",
            Self::InvertedLifetime => "access grant lifetime is inverted",
            Self::EventTimeCannotAuthorize => "event time cannot authorize access",
            Self::UnknownAccessClock => "unknown access clock",
            Self::UnknownAccessRole => "unknown access role",
            Self::BlanketMaskIsNotAuthorization => "blanket mask is not authorization",
            Self::NoMatchingGrant => "no matching access grant",
            Self::InvalidAccessPayload => "invalid access payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TenantAccessError {}

#[cfg(test)]
mod tests {
    use super::TenantAccessError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                TenantAccessError::TenantMismatch,
                "access grant used for a different tenant",
            ),
            (
                TenantAccessError::PrincipalMismatch,
                "access grant used for a different principal",
            ),
            (
                TenantAccessError::RoleNotGranted,
                "access role is not granted",
            ),
            (
                TenantAccessError::NotYetValid,
                "access grant is not yet valid",
            ),
            (TenantAccessError::Expired, "access grant is expired"),
            (
                TenantAccessError::InvertedLifetime,
                "access grant lifetime is inverted",
            ),
            (
                TenantAccessError::EventTimeCannotAuthorize,
                "event time cannot authorize access",
            ),
            (
                TenantAccessError::UnknownAccessClock,
                "unknown access clock",
            ),
            (TenantAccessError::UnknownAccessRole, "unknown access role"),
            (
                TenantAccessError::BlanketMaskIsNotAuthorization,
                "blanket mask is not authorization",
            ),
            (
                TenantAccessError::NoMatchingGrant,
                "no matching access grant",
            ),
            (
                TenantAccessError::InvalidAccessPayload,
                "invalid access payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
