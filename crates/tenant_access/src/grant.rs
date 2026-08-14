//! Tenant/role grants with a system-time lifetime window.

use crate::{AccessRole, PrincipalId, TenantAccessError, TenantId};
use temporal_core::SystemTime;

/// One request to exercise a tenant/role grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessRequest {
    tenant: TenantId,
    principal: PrincipalId,
    role: AccessRole,
    evaluation_time: SystemTime,
}

impl AccessRequest {
    /// Bind the requested tenant, principal, role, and system-time evaluation.
    #[must_use]
    pub const fn new(
        tenant: TenantId,
        principal: PrincipalId,
        role: AccessRole,
        evaluation_time: SystemTime,
    ) -> Self {
        Self {
            tenant,
            principal,
            role,
            evaluation_time,
        }
    }

    /// Return the requested tenant.
    #[must_use]
    pub const fn tenant(self) -> TenantId {
        self.tenant
    }

    /// Return the requested principal.
    #[must_use]
    pub const fn principal(self) -> PrincipalId {
        self.principal
    }

    /// Return the requested role.
    #[must_use]
    pub const fn role(self) -> AccessRole {
        self.role
    }

    /// Return the system-time evaluation instant.
    #[must_use]
    pub const fn evaluation_time(self) -> SystemTime {
        self.evaluation_time
    }
}

/// One tenant/role grant with an inclusive-start, exclusive-end lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessGrant {
    tenant: TenantId,
    principal: PrincipalId,
    role: AccessRole,
    valid_from: SystemTime,
    valid_to: Option<SystemTime>,
}

impl AccessGrant {
    /// Bind a principal to one tenant role for a system-time window.
    ///
    /// `valid_to = None` is an open-ended active grant. A present end bound
    /// must be strictly after `valid_from`.
    ///
    /// # Errors
    ///
    /// Returns [`TenantAccessError::InvertedLifetime`] when the end is not
    /// after the start.
    pub fn new(
        tenant: TenantId,
        principal: PrincipalId,
        role: AccessRole,
        valid_from: SystemTime,
        valid_to: Option<SystemTime>,
    ) -> Result<Self, TenantAccessError> {
        if let Some(end) = valid_to
            && end.instant() <= valid_from.instant()
        {
            return Err(TenantAccessError::InvertedLifetime);
        }
        Ok(Self {
            tenant,
            principal,
            role,
            valid_from,
            valid_to,
        })
    }

    /// Return the granted tenant.
    #[must_use]
    pub const fn tenant(self) -> TenantId {
        self.tenant
    }

    /// Return the holding principal.
    #[must_use]
    pub const fn principal(self) -> PrincipalId {
        self.principal
    }

    /// Return the granted role.
    #[must_use]
    pub const fn role(self) -> AccessRole {
        self.role
    }

    /// Return the inclusive system-time start.
    #[must_use]
    pub const fn valid_from(self) -> SystemTime {
        self.valid_from
    }

    /// Return the exclusive system-time end, if any.
    #[must_use]
    pub const fn valid_to(self) -> Option<SystemTime> {
        self.valid_to
    }

    /// Authorize a request against this grant.
    ///
    /// # Errors
    ///
    /// Returns a tenant, principal, role, or lifetime error when the request
    /// is outside this grant.
    pub fn authorize(self, request: &AccessRequest) -> Result<(), TenantAccessError> {
        if request.tenant != self.tenant {
            return Err(TenantAccessError::TenantMismatch);
        }
        if request.principal != self.principal {
            return Err(TenantAccessError::PrincipalMismatch);
        }
        if request.role != self.role {
            return Err(TenantAccessError::RoleNotGranted);
        }
        if request.evaluation_time.instant() < self.valid_from.instant() {
            return Err(TenantAccessError::NotYetValid);
        }
        if let Some(end) = self.valid_to
            && request.evaluation_time.instant() >= end.instant()
        {
            return Err(TenantAccessError::Expired);
        }
        Ok(())
    }
}

/// Authorize a request when any stored grant matches.
///
/// A principal may hold several roles for one tenant. One matching grant is
/// sufficient. Empty grant sets fail closed.
///
/// # Errors
///
/// Returns [`TenantAccessError::InvalidAccessPayload`] when `grants` is empty
/// and [`TenantAccessError::NoMatchingGrant`] when none authorize.
pub fn authorize_with_grants(
    grants: &[AccessGrant],
    request: &AccessRequest,
) -> Result<(), TenantAccessError> {
    if grants.is_empty() {
        return Err(TenantAccessError::InvalidAccessPayload);
    }
    if grants.iter().any(|grant| grant.authorize(request).is_ok()) {
        Ok(())
    } else {
        Err(TenantAccessError::NoMatchingGrant)
    }
}

#[cfg(test)]
mod tests {
    use super::{AccessGrant, AccessRequest};
    use crate::{AccessRole, PrincipalId, TenantId};
    use temporal_core::SystemTime;
    use uuid::Uuid;

    fn system_time(stamp: &str) -> SystemTime {
        SystemTime::parse_rfc3339(stamp).expect("rfc3339")
    }

    #[test]
    fn grant_and_request_accessors_round_trip() {
        let tenant = TenantId::from_uuid(Uuid::from_u128(7));
        let principal = PrincipalId::from_uuid(Uuid::from_u128(8));
        let start = system_time("2026-02-01T00:00:00Z");
        let end = system_time("2026-03-01T00:00:00Z");
        let grant = AccessGrant::new(
            tenant,
            principal,
            AccessRole::ExportOfficer,
            start,
            Some(end),
        )
        .expect("window");
        let request = AccessRequest::new(tenant, principal, AccessRole::ExportOfficer, start);
        assert_eq!(grant.tenant().as_uuid(), Uuid::from_u128(7));
        assert_eq!(grant.principal().as_uuid(), Uuid::from_u128(8));
        assert_eq!(grant.role(), AccessRole::ExportOfficer);
        assert_eq!(grant.valid_from(), start);
        assert_eq!(grant.valid_to(), Some(end));
        assert_eq!(request.tenant(), tenant);
        assert_eq!(request.principal(), principal);
        assert_eq!(request.role(), AccessRole::ExportOfficer);
        assert_eq!(request.evaluation_time(), start);
        grant.authorize(&request).expect("start is valid");
    }
}
