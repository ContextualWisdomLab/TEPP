#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Six nominal clocks, strict absolute instants, and uncertain intervals.
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

mod clock;
mod error;
mod instant;
mod interval;
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
/// The only temporal JSON wire-schema version accepted by this crate.
pub use wire::TEMPORAL_WIRE_SCHEMA_VERSION;
