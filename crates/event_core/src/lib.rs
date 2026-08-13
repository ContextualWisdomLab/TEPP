#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions and CHRONOS schema-slot predictions
//! never silently become instances.

mod confidence;
mod error;
mod identifier;
mod instance;
mod mention;
mod registry;
mod role;
mod schema;

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
/// Fallible textual event mention.
pub use mention::EventMention;
/// In-memory registry separating mentions from instances.
pub use registry::EventRegistry;
/// Typed event role kind.
pub use role::EventRoleKind;
/// Opaque CHRONOS schema-prediction identity.
pub use schema::SchemaPredictionId;
/// Predicted or observed filler for one schema slot.
pub use schema::SchemaSlotAssignment;
/// Filled-versus-empty occupancy label.
pub use schema::SchemaSlotLabel;
/// Threshold a slot-occupancy probability into a fill label.
pub use schema::decide_schema_slot;
/// Explicit refusal to treat a schema prediction as an instance.
pub use schema::refuse_schema_prediction_as_instance;
/// Explicit refusal to treat a schema prediction as a state transition.
pub use schema::refuse_schema_prediction_as_transition;
/// Precision of recovered filled slots against known truth.
pub use schema::schema_slot_precision;
/// Recall of recovered filled slots against known truth.
pub use schema::schema_slot_recall;
