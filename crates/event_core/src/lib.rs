#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions never silently become instances. TDT
//! detections and CHRONOS predictions remain measurement or hypothesis
//! artifacts until independently promoted.

mod confidence;
mod error;
mod identifier;
mod instance;
mod intelligence;
mod link;
mod mention;
mod registry;
mod role;

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
