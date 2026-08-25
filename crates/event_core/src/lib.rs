#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. Mentions and first-story detections never
//! silently become instances, and TDT detections and CHRONOS predictions
//! remain measurement or hypothesis artifacts until independently promoted.
//! and scientific estimation. Mentions never silently become instances. TDT
//! detections and CHRONOS predictions remain measurement or hypothesis
//! artifacts until independently promoted. Track assignments, story
//! segmentations, CHRONOS schema-slot predictions, and occurrence forecasts
//! remain measurement or hypothesis artifacts and cannot promote an instance
//! without an explicit evidence-backed promotion gate.

mod confidence;
mod error;
mod first_story;
mod identifier;
mod instance;
mod intelligence;
mod mention;
mod prediction;
mod registry;
mod role;
mod schema;
mod segment;
mod track;

/// Finite confidence on the closed unit interval.
pub use confidence::EventConfidence;
/// Mean squared error of mention probabilities against binary truth.
pub use confidence::mention_brier_score;
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
