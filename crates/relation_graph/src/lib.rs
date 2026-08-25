#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Typed document, segment, event, entity, and transition relations.
//!
//! TEPP represents observed and inferred relationships explicitly. Forward
//! state-transition edges never move backward in event time and never form
//! cycles, while citation, revision, translation, support, contradiction, and
//! retrospective-reporting edges may point to the past without becoming reverse
//! state transitions.

mod edge;
mod error;
mod graph;
mod identifier;
mod kind;
mod provenance;
mod transition;

/// One typed relation edge between opaque endpoints.
pub use edge::RelationEdge;
/// Fail-closed relation-graph validation errors.
pub use error::RelationError;
/// In-memory typed relation graph.
pub use graph::RelationGraph;
/// Opaque relation-edge identifier.
pub use identifier::RelationEdgeId;
/// Opaque relation-endpoint identifier.
pub use identifier::RelationEndpointId;
/// Closed relation vocabulary with derived transition classification.
pub use kind::RelationKind;
/// Refuse treating association or precedence as causation.
pub use kind::refuse_association_as_cause;
/// Observed versus inferred relation evidence status.
pub use provenance::RelationEvidenceStatus;
/// Validate forward-only event-time order for transition edges.
pub use transition::validate_forward_event_order;
