//! Opaque identifiers for event mentions and instances.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Opaque identifier for a fallible textual event mention.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct EventMentionId(Uuid);

impl EventMentionId {
    /// Mint a new time-ordered mention identifier.
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

impl Default for EventMentionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Opaque identifier for a versioned event instance (not a mention).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct EventInstanceId(Uuid);

impl EventInstanceId {
    /// Mint a new time-ordered instance identifier.
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

impl Default for EventInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{EventInstanceId, EventMentionId};

    #[test]
    fn mention_and_instance_ids_are_distinct_types() {
        let mention = EventMentionId::default();
        let _fresh = EventMentionId::new();
        let instance = EventInstanceId::default();
        assert_ne!(mention.as_uuid(), instance.as_uuid());
        assert_eq!(
            EventMentionId::from_uuid(mention.as_uuid()).as_uuid(),
            mention.as_uuid()
        );
        assert_eq!(
            EventInstanceId::from_uuid(instance.as_uuid()).as_uuid(),
            instance.as_uuid()
        );
    }
}
