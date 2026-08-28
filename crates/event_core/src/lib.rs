#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned event instances, mentions, roles, subevents, and provenance.
//!
//! TEPP separates **fallible event mentions** grounded in evidence from
//! **versioned event instances** used for temporal state, multilevel membership,
//! and scientific estimation. [`EventMention`] is the only constructible
//! mention type: it cites one exact source extent, document identity,
//! six-clock evidence, extractor version, and review status, and the surface
//! form is the document substring at that span. Mentions, first-story
//! detections, TDT detections, and CHRONOS predictions never silently become
//! instances. [`EventIntelligenceComposition`] is the versioned TDT/CHRONOS
//! workflow over admitted artifacts; promotion still requires the existing
//! evidence-backed gate. Track assignments, story segmentations, CHRONOS
//! schema-slot predictions, and occurrence forecasts remain measurement or
//! hypothesis artifacts and cannot promote an instance without that gate.
//! Bounded Allen/CHRONOS interval consistency
//! ([`IntervalConsistencyNetwork`]) derives implications and rejects
//! contradictions without claiming unrestricted global satisfiability.

mod composition;
mod confidence;
mod criterion_posterior;
mod error;
mod event_time_posterior;
mod first_story;
mod identifier;
mod instance;
mod intelligence;
mod interval_consistency;
mod interval_consistency_artifact;
mod link;
mod mention;
mod prediction;
mod registry;
mod role;
mod schema;
mod segment;
mod span_mention;
mod temporal_relation_posterior;
mod track;

/// Wire schema version for the unified event-intelligence workflow.
pub use composition::EVENT_INTELLIGENCE_WORKFLOW_VERSION;
/// Versioned TDT/CHRONOS composition over admitted artifacts.
pub use composition::EventIntelligenceComposition;
/// Named thresholds and version for one reproducible intelligence run.
pub use composition::EventIntelligenceWorkflowConfig;
/// Admit already-extracted TDT/CHRONOS artifacts into one versioned workflow.
pub use composition::compose_event_intelligence;
/// Explicit refusal to treat a composition as an event instance.
pub use composition::refuse_composition_as_instance;
/// Explicit refusal to treat a composition as a state transition.
pub use composition::refuse_composition_as_transition;
/// Finite confidence on the closed unit interval.
pub use confidence::EventConfidence;
/// Mean squared error of mention probabilities against binary truth.
pub use confidence::mention_brier_score;
/// Identified Jeffreys posterior for independent criterion observations.
pub use criterion_posterior::CriterionPosterior;
/// Fail-closed independent criterion posterior errors.
pub use criterion_posterior::CriterionPosteriorError;
/// Independent binary criterion observation counts.
pub use criterion_posterior::IndependentCriterionCounts;
/// Fit the paper-grounded independent criterion posterior on CPU f64.
pub use criterion_posterior::fit_independent_criterion_posterior;
/// Fail-closed event-ontology errors.
pub use error::EventError;
/// Exact discrete event-time posterior atom.
pub use event_time_posterior::EventTimePosteriorAtom;
/// Complete canonical event-time posterior draws.
pub use event_time_posterior::EventTimePosteriorDraws;
/// Fail-closed event-time posterior errors.
pub use event_time_posterior::EventTimePosteriorError;
/// Materialize a producer-owned discrete event-time posterior exactly.
pub use event_time_posterior::materialize_event_time_posterior;
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
/// Bounded CHRONOS-style interval-consistency network for event intelligence.
pub use interval_consistency::IntervalConsistencyNetwork;
/// Summary of one successful bounded interval-consistency closure.
pub use interval_consistency::IntervalConsistencyReport;
/// Explicit refusal to treat bounded path consistency as unrestricted SAT.
pub use interval_consistency::refuse_interval_consistency_as_unrestricted_satisfiability;
/// Explicit refusal to promote an interval contradiction into an instance.
pub use interval_consistency::refuse_interval_contradiction_as_instance;
/// Durable JSON and `GraphML` projection of one bounded consistency result.
pub use interval_consistency_artifact::{
    INTERVAL_CONSISTENCY_ARTIFACT_TYPE, IntervalConsistencyArtifact,
    IntervalConsistencyArtifactRelation,
};
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
/// Fallible textual event mention grounded in one exact source extent.
pub use mention::EventMention;
/// Explicit refusal to treat a span-grounded mention as an instance.
pub use mention::refuse_span_mention_as_instance;
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
/// Six-clock evidence bound to one mention.
pub use span_mention::MentionEvidenceClocks;
/// Proposed-versus-reviewed mention inspection status.
pub use span_mention::MentionReviewStatus;
/// Precision of recovered mention extents against known truth.
pub use span_mention::mention_span_precision;
/// Recall of recovered mention extents against known truth.
pub use span_mention::mention_span_recall;
/// Posterior qualitative temporal relation for one common event-time draw.
pub use temporal_relation_posterior::DrawTemporalRelation;
/// Posterior relation frequencies derived without a date threshold.
pub use temporal_relation_posterior::TemporalRelationPosterior;
/// Fail-closed temporal relation posterior errors.
pub use temporal_relation_posterior::TemporalRelationPosteriorError;
/// Derive CHRONOS-compatible qualitative relation draws from common event-time draws.
pub use temporal_relation_posterior::infer_temporal_relation_posterior;
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
