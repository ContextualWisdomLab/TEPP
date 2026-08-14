#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions and TDT topic clusters never silently
//! become instances.

mod confidence;
mod error;
mod identifier;
mod instance;
mod mention;
mod registry;
mod role;
mod topic;

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
/// Opaque TDT topic-cluster identity.
pub use topic::TopicClusterId;
/// New-topic versus existing-topic detection label.
pub use topic::TopicDetectionLabel;
/// Threshold a new-topic probability into a detection label.
pub use topic::decide_topic_detection;
/// False-alarm rate for new-topic detections.
pub use topic::new_topic_false_alarm_rate;
/// Miss rate for new-topic detections.
pub use topic::new_topic_miss_rate;
/// Explicit refusal to treat a topic cluster as an instance.
pub use topic::refuse_topic_cluster_as_event_instance;
/// Pair precision of recovered topic clusters against known truth.
pub use topic::topic_cluster_pair_precision;
/// Pair recall of recovered topic clusters against known truth.
pub use topic::topic_cluster_pair_recall;
