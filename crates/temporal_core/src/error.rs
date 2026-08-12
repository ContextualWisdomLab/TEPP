//! Fail-closed errors for temporal values, intervals, and wire records.

use std::fmt;

/// A fail-closed temporal-domain validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TemporalError {
    /// A supplied timestamp was not an accepted strict RFC 3339 instant.
    InvalidTimestamp,
    /// A known interval used the `unknown` precision marker.
    InvalidTemporalPrecision,
    /// A lower boundary occurred after its upper boundary.
    InvalidIntervalOrder,
    /// A bounded interval selected no instant.
    EmptyInterval,
    /// Interval boundaries, precision, and certainty disagreed.
    InvalidIntervalCertainty,
    /// Qualitative relation classification received a nonproper or open interval.
    RelationRequiresProperBoundedInterval,
    /// A JSON wire payload was malformed, incomplete, or contained unknown fields.
    InvalidWirePayload,
    /// A JSON wire payload used a schema version this crate does not support.
    UnsupportedWireVersion,
    /// A JSON wire record declared a different nominal clock type.
    ClockTypeMismatch,
}

impl fmt::Display for TemporalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidTimestamp => "invalid temporal timestamp",
            Self::InvalidTemporalPrecision => "invalid temporal precision",
            Self::InvalidIntervalOrder => "invalid temporal interval order",
            Self::EmptyInterval => "temporal interval is empty",
            Self::InvalidIntervalCertainty => "invalid temporal interval certainty",
            Self::RelationRequiresProperBoundedInterval => {
                "temporal relation requires proper bounded intervals"
            }
            Self::InvalidWirePayload => "invalid temporal wire payload",
            Self::UnsupportedWireVersion => "unsupported temporal wire version",
            Self::ClockTypeMismatch => "temporal clock type mismatch",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TemporalError {}
