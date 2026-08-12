//! Fail-closed relation-graph validation errors.

use std::fmt;

/// A fail-closed relation-graph domain error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RelationError {
    /// A forward transition would move backward in event time.
    ReverseTemporalOrder,
    /// Event-time intervals are not proper enough to establish forward order.
    UncertainTemporalOrder,
    /// A transition edge formed a cycle in the forward-transition subgraph.
    TransitionCycle,
    /// A transition edge used the same endpoint as source and target.
    SelfTransition,
    /// An unknown relation kind name was supplied.
    UnknownRelationKind,
    /// A duplicate edge identity was rejected by the graph.
    DuplicateRelationEdge,
    /// A wire payload was malformed, incomplete, or used an unsupported version.
    InvalidWirePayload,
    /// A wire payload used a schema version this crate does not support.
    UnsupportedWireVersion,
}

impl fmt::Display for RelationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ReverseTemporalOrder => "reverse temporal order on transition edge",
            Self::UncertainTemporalOrder => "uncertain temporal order on transition edge",
            Self::TransitionCycle => "forward transition cycle",
            Self::SelfTransition => "self transition edge",
            Self::UnknownRelationKind => "unknown relation kind",
            Self::DuplicateRelationEdge => "duplicate relation edge",
            Self::InvalidWirePayload => "invalid relation wire payload",
            Self::UnsupportedWireVersion => "unsupported relation wire version",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RelationError {}

#[cfg(test)]
mod tests {
    use super::RelationError;

    #[test]
    fn error_messages_are_stable_and_redacted() {
        for (error, message) in [
            (
                RelationError::ReverseTemporalOrder,
                "reverse temporal order on transition edge",
            ),
            (
                RelationError::UncertainTemporalOrder,
                "uncertain temporal order on transition edge",
            ),
            (RelationError::TransitionCycle, "forward transition cycle"),
            (RelationError::SelfTransition, "self transition edge"),
            (RelationError::UnknownRelationKind, "unknown relation kind"),
            (
                RelationError::DuplicateRelationEdge,
                "duplicate relation edge",
            ),
            (
                RelationError::InvalidWirePayload,
                "invalid relation wire payload",
            ),
            (
                RelationError::UnsupportedWireVersion,
                "unsupported relation wire version",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
