#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions never silently become instances. TDT
//! track assignments remain measurement evidence and cannot promote an instance.

mod confidence;
mod error;
mod identifier;
mod instance;
mod mention;
mod registry;
mod role;
mod track;

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
