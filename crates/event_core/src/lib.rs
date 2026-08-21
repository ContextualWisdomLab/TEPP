#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions and CHRONOS occurrence forecasts never
//! silently become instances.

mod confidence;
mod error;
mod identifier;
mod instance;
mod mention;
mod prediction;
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
/// Fallible textual event mention.
pub use mention::EventMention;
/// One CHRONOS occurrence forecast that remains hypothetical.
pub use prediction::ChronosOccurrenceForecast;
/// Opaque CHRONOS occurrence-prediction identity.
pub use prediction::ChronosPredictionId;
/// Later-observed occurrence truth for a CHRONOS forecast.
pub use prediction::OccurrenceTruth;
/// Mean squared error of CHRONOS occurrence forecasts against later truth.
pub use prediction::chronos_prediction_brier_score;
/// Explicit refusal to treat a CHRONOS prediction as an event instance.
pub use prediction::refuse_prediction_as_instance;
/// In-memory registry separating mentions from instances.
pub use registry::EventRegistry;
/// Typed event role kind.
pub use role::EventRoleKind;
