//! Opaque analytical identifiers for members and groups.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An opaque analytical identifier for a document, person, or other member.
///
/// Identity mapping to natural persons remains outside this crate (purpose-bound
/// privacy authority). Psychometric estimators consume only these opaque IDs.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct MemberId(Uuid);

impl MemberId {
    /// Mint a new time-ordered opaque member identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Reconstruct a member identifier from a validated UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Borrow the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for MemberId {
    fn default() -> Self {
        Self::new()
    }
}

/// An opaque analytical identifier for a cross-classified group context.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct GroupId(Uuid);

impl GroupId {
    /// Mint a new time-ordered opaque group identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Reconstruct a group identifier from a validated UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Borrow the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for GroupId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{GroupId, MemberId};
    use uuid::Uuid;

    #[test]
    fn identifiers_preserve_uuid_round_trip_and_default_minting() {
        let member = MemberId::default();
        let group = GroupId::default();
        assert_eq!(
            MemberId::from_uuid(member.as_uuid()).as_uuid(),
            member.as_uuid()
        );
        assert_eq!(
            GroupId::from_uuid(group.as_uuid()).as_uuid(),
            group.as_uuid()
        );
        let fixed = Uuid::nil();
        assert_eq!(MemberId::from_uuid(fixed).as_uuid(), fixed);
        assert_ne!(MemberId::new().as_uuid(), Uuid::nil());
        assert_ne!(GroupId::new().as_uuid(), Uuid::nil());
    }
}
