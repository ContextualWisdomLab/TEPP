//! Nominally distinct clocks over one absolute instant representation.

use crate::{TemporalError, TemporalInstant};
use serde_json::Value;
use std::fmt;

mod sealed {
    /// Prevents downstream crates from inventing unreviewed clock semantics.
    pub trait Sealed {}
}

/// A sealed nominal clock that wraps an absolute [`TemporalInstant`].
///
/// Implementations identify their wire representation with a stable
/// `snake_case` name. External crates cannot add clocks whose semantics have
/// not been reviewed by TEPP's temporal contract.
pub trait TemporalClock:
    sealed::Sealed + Clone + Copy + fmt::Debug + Eq + Ord + Send + Sync + 'static
{
    /// Stable JSON wire identifier for this clock.
    const WIRE_NAME: &'static str;

    /// Construct this nominal clock from a validated absolute instant.
    #[must_use]
    fn from_instant(instant: TemporalInstant) -> Self;

    /// Return the wrapped absolute instant.
    #[must_use]
    fn instant(self) -> TemporalInstant;
}

macro_rules! define_clock {
    ($name:ident, $wire_name:literal, $documentation:literal) => {
        #[doc = $documentation]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(TemporalInstant);

        impl $name {
            /// Parse a strict RFC 3339 timestamp and normalize it to UTC.
            ///
            /// # Errors
            ///
            /// Returns [`TemporalError::InvalidTimestamp`] when the timestamp
            /// does not satisfy TEPP's strict absolute-time syntax or calendar
            /// semantics.
            pub fn parse_rfc3339(input: &str) -> Result<Self, TemporalError> {
                TemporalInstant::parse_rfc3339(input).map(Self)
            }

            /// Return a canonical UTC RFC 3339 representation.
            #[must_use]
            pub fn to_rfc3339(self) -> String {
                self.0.to_rfc3339()
            }

            /// Return the wrapped absolute instant.
            #[must_use]
            pub const fn instant(self) -> TemporalInstant {
                self.0
            }

            /// Serialize this clock through the strict versioned JSON contract.
            ///
            /// # Errors
            ///
            /// Returns [`TemporalError::InvalidWirePayload`] if serialization
            /// cannot represent this validated clock.
            pub fn to_wire_json(self) -> Result<String, TemporalError> {
                crate::wire::serialize_clock(self)
            }

            /// Reconstruct and validate this clock from versioned JSON.
            ///
            /// # Errors
            ///
            /// Returns a fail-closed wire, version, clock-type, or timestamp
            /// error when the payload is not exactly valid for this clock.
            pub fn from_wire_json(payload: &str) -> Result<Self, TemporalError> {
                crate::wire::deserialize_clock(payload)
            }

            /// Return the Draft 2020-12 JSON Schema for this clock's wire record.
            #[must_use]
            pub fn wire_json_schema() -> Value {
                crate::wire::clock_json_schema::<Self>()
            }
        }

        impl sealed::Sealed for $name {}

        impl TemporalClock for $name {
            const WIRE_NAME: &'static str = $wire_name;

            fn from_instant(instant: TemporalInstant) -> Self {
                Self(instant)
            }

            fn instant(self) -> TemporalInstant {
                self.0
            }
        }
    };
}

define_clock!(
    EventTime,
    "event_time",
    "The time at which an event occurred or a state was valid."
);
define_clock!(
    AssertionTime,
    "assertion_time",
    "The time at which a source asserted a claim about an event or state."
);
define_clock!(
    DocumentTime,
    "document_time",
    "The creation, publication, revision, or reporting time of a document."
);
define_clock!(
    SystemTime,
    "system_time",
    "The time at which TEPP observed or recorded a source-system change."
);
define_clock!(
    AvailableTime,
    "available_time",
    "The time at which evidence became available to an analyst or model."
);
define_clock!(
    KnowledgeCutoff,
    "knowledge_cutoff",
    "The latest availability time permitted in one historical analysis."
);
