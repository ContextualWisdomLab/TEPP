#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Six nominal clocks, strict absolute instants, uncertain intervals, and bounded temporal reasoning.
//!
//! TEPP distinguishes the time at which an event occurred, a claim was made,
//! a document was created, the platform observed data, evidence became
//! available, and a model's knowledge was cut off. All clocks share one
//! absolute nanosecond-resolution representation while remaining distinct Rust
//! types.
//!
//! ```compile_fail
//! use temporal_core::{DocumentTime, EventTime};
//!
//! let event = EventTime::parse_rfc3339("2026-08-06T01:00:00Z")?;
//! let document: DocumentTime = event;
//! # Ok::<(), temporal_core::TemporalError>(())
//! ```
//!
//! Temporal intervals preserve exact, bounded, open-ended, and unknown
//! semantics. Known intervals retain source precision; unknown intervals do not
//! claim containment. JSON interchange is explicit, versioned, clock-specific,
//! and reconstructed through the same domain validation boundary.
//!
//! Proper bounded intervals can be classified with Allen's thirteen elementary
//! relations. Relation sets support inverse and complete composition, while a
//! resource-bounded path-consistency reasoner preserves direct assertions,
//! derived narrowing, and conservative supporting-assertion provenance.

mod clock;
mod error;
mod instant;
mod interval;
mod reasoner;
mod relation;
mod wire;

/// The time at which a source asserted a claim about an event or state.
pub use clock::AssertionTime;
/// The time at which evidence became available to an analyst or model.
pub use clock::AvailableTime;
/// The creation, publication, revision, or reporting time of a document.
pub use clock::DocumentTime;
/// The time at which an event occurred or a state was valid.
pub use clock::EventTime;
/// The latest availability time permitted in one historical analysis.
pub use clock::KnowledgeCutoff;
/// The time at which TEPP observed or recorded a source-system change.
pub use clock::SystemTime;
/// A sealed nominal TEPP clock over one absolute instant representation.
pub use clock::TemporalClock;
/// A fail-closed temporal-domain validation error.
pub use error::TemporalError;
/// An absolute UTC instant represented to nanosecond precision.
pub use instant::TemporalInstant;
/// One lower or upper interval boundary.
pub use interval::TemporalBoundary;
/// Whether a temporal representation is exact, bounded, or unknown.
pub use interval::TemporalCertainty;
/// A validated interval whose boundaries use one nominal TEPP clock.
pub use interval::TemporalInterval;
/// The source precision retained for a temporal value or interval.
pub use interval::TemporalPrecision;
/// Summary of one successful bounded closure operation.
pub use reasoner::ClosureReport;
/// An opaque identifier for one accepted relation assertion.
pub use reasoner::ConstraintId;
/// One observed or derived relation returned by the reasoner.
pub use reasoner::DerivedRelation;
/// The bounded resource whose configured maximum was exceeded.
pub use reasoner::ReasonerLimitKind;
/// Evidence that a qualitative temporal network has no possible relation.
pub use reasoner::TemporalContradiction;
/// A bounded qualitative interval-constraint network.
pub use reasoner::TemporalReasoner;
/// A fail-closed temporal-reasoner error.
pub use reasoner::TemporalReasonerError;
/// Explicit capacity bounds for one temporal reasoner instance.
pub use reasoner::TemporalReasonerLimits;
/// An opaque identifier for one interval variable in a reasoner instance.
pub use reasoner::TemporalVariableId;
/// One of Allen's thirteen elementary relations between proper intervals.
pub use relation::AllenRelation;
/// A compact set of possible elementary interval relations.
pub use relation::RelationSet;
/// Classify two proper, two-sided, nonzero intervals with Allen's algebra.
pub use relation::classify_interval_relation;
/// The only temporal JSON wire-schema version accepted by this crate.
pub use wire::TEMPORAL_WIRE_SCHEMA_VERSION;
