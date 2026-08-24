#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions never silently become instances. TDT
//! detections and CHRONOS predictions remain measurement or hypothesis
//! artifacts until independently promoted.
//! track assignments remain measurement evidence and cannot promote an instance.
//! and scientific estimation. Mentions and CHRONOS schema-slot predictions
//! never silently become instances.

mod confidence;
mod error;
mod identifier;
mod instance;
mod intelligence;
mod mention;
mod registry;
mod role;
mod track;
mod schema;

/// Finite confidence on the closed unit interval.
pub use confidence::EventConfidence;
/// Mean squared error of mention probabilities against binary truth.
pub use confidence::mention_brier_score;
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
/// Epistemic layer of an event-intelligence output.
pub use intelligence::EventEvidenceLayer;
/// Known-truth first-story detection counts.
pub use intelligence::FirstStoryRates;
/// First-story versus subsequent-track decision.
pub use intelligence::TdtStoryDecision;
/// Admit only promoted transitions into the forward state graph.
pub use intelligence::admit_state_transition;
/// Classify a candidate story as first-story or track.
pub use intelligence::classify_tdt_story;
/// Score first-story detections against a known stream.
pub use intelligence::first_story_detection_rates;
/// Fallible textual event mention.
pub use mention::EventMention;
/// In-memory registry separating mentions from instances.
pub use registry::EventRegistry;
/// Typed event role kind.
pub use role::EventRoleKind;
/// Assignment of one mention to one hypothesized TDT track.
pub use track::EventTrackAssignment;
/// Opaque TDT track identity.
pub use track::EventTrackId;
/// TDT continue-versus-switch track label.
pub use track::EventTrackLabel;
/// Threshold a same-track probability into a continue/switch label.
pub use track::decide_track_continue;
/// Explicit refusal to treat a TDT track as an event instance.
pub use track::refuse_track_as_instance;
/// Explicit refusal to treat a TDT track as a state transition.
pub use track::refuse_track_as_transition;
/// Identity-switch rate among consecutive same-truth-track mentions.
pub use track::tracking_identity_switch_rate;
/// Precision of recovered same-track mention pairs against known truth.
pub use track::tracking_pair_precision;
/// Recall of recovered same-track mention pairs against known truth.
pub use track::tracking_pair_recall;
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
