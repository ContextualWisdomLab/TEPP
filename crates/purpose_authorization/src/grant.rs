//! Purpose-bound grants held by an opaque principal.

use crate::{PurposeAuthorizationError, PurposeCode, refuse_cross_purpose_use};
use uuid::Uuid;

/// Opaque principal that holds a purpose grant.
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

/// One purpose-bound authorization grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationGrant {
    purpose: PurposeCode,
    principal: PrincipalId,
}

impl AuthorizationGrant {
    /// Bind a principal to one processing purpose.
    #[must_use]
    pub const fn new(purpose: PurposeCode, principal: PrincipalId) -> Self {
        Self { purpose, principal }
    }

    /// Return the granted purpose.
    #[must_use]
    pub const fn purpose(self) -> PurposeCode {
        self.purpose
    }

    /// Return the holding principal.
    #[must_use]
    pub const fn principal(self) -> PrincipalId {
        self.principal
    }

    /// Authorize a requested purpose against this grant.
    ///
    /// # Errors
    ///
    /// Returns [`PurposeAuthorizationError::CrossPurposeUse`] when the
    /// requested purpose differs.
    pub fn authorize(self, requested: PurposeCode) -> Result<(), PurposeAuthorizationError> {
        refuse_cross_purpose_use(self.purpose, requested)
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthorizationGrant, PrincipalId};
    use crate::PurposeCode;
    use uuid::Uuid;

    #[test]
    fn grant_accessors_round_trip() {
        let principal = PrincipalId::from_uuid(Uuid::from_u128(8));
        let grant = AuthorizationGrant::new(PurposeCode::ExportFulfillment, principal);
        assert_eq!(grant.purpose(), PurposeCode::ExportFulfillment);
        assert_eq!(grant.principal().as_uuid(), Uuid::from_u128(8));
    }
}
