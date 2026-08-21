#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions and TDT story segmentations never
//! silently become instances.

mod confidence;
mod error;
mod identifier;
mod instance;
mod mention;
mod registry;
mod role;
mod segment;

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
/// TDT story-boundary versus continuation label.
pub use segment::StoryBoundaryLabel;
/// Ordered TDT story/event segmentation.
pub use segment::StorySegmentation;
/// Threshold a boundary probability into a detection label.
pub use segment::decide_story_boundary;
/// Explicit refusal to treat a story segmentation as an instance.
pub use segment::refuse_story_segmentation_as_instance;
/// Explicit refusal to treat a story segmentation as a state transition.
pub use segment::refuse_story_segmentation_as_transition;
/// Precision of recovered interior story boundaries against known truth.
pub use segment::story_boundary_precision;
/// Recall of recovered interior story boundaries against known truth.
pub use segment::story_boundary_recall;
/// Beeferman Pk against a known-truth segmentation.
pub use segment::story_pk;
/// Pevzner–Hearst `WindowDiff` against a known-truth segmentation.
pub use segment::story_window_diff;
