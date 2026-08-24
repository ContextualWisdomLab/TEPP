//! Opaque identifier for one evidence-bounded interpretation.

use uuid::Uuid;

/// Opaque identifier for an untrusted interpretation proposal.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InterpretationId(Uuid);

impl InterpretationId {
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
    use super::InterpretationId;
    use uuid::Uuid;

    #[test]
    fn identity_round_trips() {
        let id = InterpretationId::from_uuid(Uuid::from_u128(9));
        assert_eq!(id.as_uuid(), Uuid::from_u128(9));
    }
}
