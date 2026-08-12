//! Opaque identifiers for relation endpoints and edges.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Opaque analytical identifier for a relation endpoint (event, document, entity).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RelationEndpointId(Uuid);

impl RelationEndpointId {
    /// Mint a new time-ordered endpoint identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

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

impl Default for RelationEndpointId {
    fn default() -> Self {
        Self::new()
    }
}

/// Opaque identifier for one typed relation edge.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RelationEdgeId(Uuid);

impl RelationEdgeId {
    /// Mint a new time-ordered edge identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

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

impl Default for RelationEdgeId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{RelationEdgeId, RelationEndpointId};

    #[test]
    fn endpoint_and_edge_ids_are_distinct_types() {
        let endpoint = RelationEndpointId::default();
        let _fresh = RelationEndpointId::new();
        let edge = RelationEdgeId::default();
        assert_ne!(endpoint.as_uuid(), edge.as_uuid());
        assert_eq!(
            RelationEndpointId::from_uuid(endpoint.as_uuid()).as_uuid(),
            endpoint.as_uuid()
        );
        assert_eq!(
            RelationEdgeId::from_uuid(edge.as_uuid()).as_uuid(),
            edge.as_uuid()
        );
    }
}
