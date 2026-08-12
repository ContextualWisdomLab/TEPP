//! Typed relation edges with derived transition classification.

use crate::{
    RelationEdgeId, RelationEndpointId, RelationError, RelationEvidenceStatus, RelationKind,
    validate_forward_event_order,
};
use temporal_core::{EventTime, TemporalInterval};

/// One typed relation edge between opaque endpoints.
///
/// Transition kinds carry validated forward event-time order. Provenance kinds
/// may point backward without becoming reverse state transitions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RelationEdge {
    edge_id: RelationEdgeId,
    kind: RelationKind,
    source: RelationEndpointId,
    target: RelationEndpointId,
    evidence_status: RelationEvidenceStatus,
    source_event_time: TemporalInterval<EventTime>,
    target_event_time: TemporalInterval<EventTime>,
}

impl RelationEdge {
    /// Construct a validated relation edge.
    ///
    /// Transition kinds reject self-loops and reverse or uncertain event-time
    /// order. Provenance kinds accept backward temporal orientation.
    ///
    /// # Errors
    ///
    /// Returns a transition, temporal-order, or identity error when validation fails.
    pub fn new(
        kind: RelationKind,
        source: RelationEndpointId,
        target: RelationEndpointId,
        evidence_status: RelationEvidenceStatus,
        source_event_time: TemporalInterval<EventTime>,
        target_event_time: TemporalInterval<EventTime>,
    ) -> Result<Self, RelationError> {
        if kind.is_transition_edge() {
            if source == target {
                return Err(RelationError::SelfTransition);
            }
            validate_forward_event_order(&source_event_time, &target_event_time)?;
        }
        Ok(Self {
            edge_id: RelationEdgeId::new(),
            kind,
            source,
            target,
            evidence_status,
            source_event_time,
            target_event_time,
        })
    }

    /// Return the edge identifier.
    #[must_use]
    pub const fn edge_id(self) -> RelationEdgeId {
        self.edge_id
    }

    /// Return the closed relation kind.
    #[must_use]
    pub const fn kind(self) -> RelationKind {
        self.kind
    }

    /// Return whether this edge is a forward state transition.
    #[must_use]
    pub const fn is_transition_edge(self) -> bool {
        self.kind.is_transition_edge()
    }

    /// Return the source endpoint.
    #[must_use]
    pub const fn source(self) -> RelationEndpointId {
        self.source
    }

    /// Return the target endpoint.
    #[must_use]
    pub const fn target(self) -> RelationEndpointId {
        self.target
    }

    /// Return observed-versus-inferred evidence status.
    #[must_use]
    pub const fn evidence_status(self) -> RelationEvidenceStatus {
        self.evidence_status
    }

    /// Return the source event-time interval.
    #[must_use]
    pub const fn source_event_time(self) -> TemporalInterval<EventTime> {
        self.source_event_time
    }

    /// Return the target event-time interval.
    #[must_use]
    pub const fn target_event_time(self) -> TemporalInterval<EventTime> {
        self.target_event_time
    }
}
