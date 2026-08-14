//! Opaque tenant and principal identities.

use uuid::Uuid;

/// Opaque tenant workspace identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TenantId(Uuid);

impl TenantId {
    /// Reconstruct from a UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Borrow the UUID value.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

/// Opaque principal that holds an access grant.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PrincipalId(Uuid);

impl PrincipalId {
    /// Reconstruct from a UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Borrow the UUID value.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{PrincipalId, TenantId};
    use uuid::Uuid;

    #[test]
    fn identity_accessors_round_trip() {
        let tenant = TenantId::from_uuid(Uuid::from_u128(4));
        let principal = PrincipalId::from_uuid(Uuid::from_u128(5));
        assert_eq!(tenant.as_uuid(), Uuid::from_u128(4));
        assert_eq!(principal.as_uuid(), Uuid::from_u128(5));
    }
}
