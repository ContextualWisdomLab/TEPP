#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions never silently become instances.

mod confidence;
mod error;
mod identifier;
mod instance;
mod link;
mod mention;
mod registry;
mod role;

/// Finite confidence on the closed unit interval.
pub use confidence::EventConfidence;
/// Fail-closed event-ontology errors.
pub use error::EventError;
/// Opaque event-instance identifier.
pub use identifier::EventInstanceId;
/// Opaque event-mention identifier.
pub use identifier::EventMentionId;
/// Wire schema version for instances.
pub use instance::EVENT_INSTANCE_WIRE_SCHEMA_VERSION;
/// Versioned event instance.
pub use instance::EventInstance;
/// Explicit refusal to cast a mention as an instance.
pub use instance::refuse_mention_as_instance;
/// TDT same-event versus distinct-event link label.
pub use link::EventLinkLabel;
/// Undirected TDT link hypothesis between two mentions.
pub use link::EventLinkPair;
/// Threshold a link probability into a detection label.
pub use link::decide_event_link;
/// Precision of recovered TDT links against known-truth pairs.
pub use link::event_link_precision;
/// Recall of recovered TDT links against known-truth pairs.
pub use link::event_link_recall;
/// Explicit refusal to treat a TDT link as an event instance.
pub use link::refuse_event_link_as_instance;
/// Explicit refusal to treat a TDT link as a state transition.
pub use link::refuse_event_link_as_transition;
/// Fallible textual event mention.
pub use mention::EventMention;
/// In-memory registry separating mentions from instances.
pub use registry::EventRegistry;
/// Typed event role kind.
pub use role::EventRoleKind;
