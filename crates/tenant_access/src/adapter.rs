//! In-memory tenant/role adapter with no application-table coupling.

use crate::{AccessGrant, AccessRequest, TenantAccessError, authorize_with_grants};

/// Evaluate an access request without reading TEPP application tables.
pub trait TenantAccessAdapter {
    /// Authorize one request against stored grants.
    ///
    /// # Errors
    ///
    /// Returns a tenant-access error when no grant authorizes the request.
    fn evaluate(&self, request: &AccessRequest) -> Result<(), TenantAccessError>;
}

/// In-memory grant set used by contract tests and standalone adapters.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryTenantAccessAdapter {
    grants: Vec<AccessGrant>,
}

impl InMemoryTenantAccessAdapter {
    /// Create an adapter from an owned grant list.
    #[must_use]
    pub fn from_grants(grants: Vec<AccessGrant>) -> Self {
        Self { grants }
    }

    /// Borrow the stored grants.
    #[must_use]
    pub fn grants(&self) -> &[AccessGrant] {
        &self.grants
    }
}

impl TenantAccessAdapter for InMemoryTenantAccessAdapter {
    fn evaluate(&self, request: &AccessRequest) -> Result<(), TenantAccessError> {
        authorize_with_grants(&self.grants, request)
    }
}

#[cfg(test)]
mod tests {
    use super::InMemoryTenantAccessAdapter;

    #[test]
    fn default_adapter_exposes_empty_grants() {
        let adapter = InMemoryTenantAccessAdapter::default();
        assert!(adapter.grants().is_empty());
    }
}
