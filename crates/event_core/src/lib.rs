#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions and first-story detections never
//! silently become instances.

mod confidence;
mod error;
mod first_story;
mod identifier;
mod instance;
mod mention;
mod registry;
mod role;

/// Finite confidence on the closed unit interval.
pub use confidence::EventConfidence;
/// Fail-closed event-ontology errors.
pub use error::EventError;
/// First-story versus follow-up detection label.
pub use first_story::FirstStoryLabel;
/// Threshold a first-story probability into a detection label.
pub use first_story::decide_first_story;
/// False-alarm rate for first-story detections.
pub use first_story::first_story_false_alarm_rate;
/// Miss rate for first-story detections.
pub use first_story::first_story_miss_rate;
/// Explicit refusal to treat a first-story detection as an instance.
pub use first_story::refuse_first_story_as_instance;
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
