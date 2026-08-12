//! Typed temporal precision, certainty, boundaries, and intervals.

use crate::{TemporalClock, TemporalError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The source precision retained for a temporal value or interval.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalPrecision {
    /// Nanosecond precision.
    Nanosecond,
    /// Microsecond precision.
    Microsecond,
    /// Millisecond precision.
    Millisecond,
    /// Whole-second precision.
    Second,
    /// Whole-minute precision.
    Minute,
    /// Whole-hour precision.
    Hour,
    /// Calendar-day precision.
    Day,
    /// Calendar-month precision.
    Month,
    /// Calendar-quarter precision.
    Quarter,
    /// Calendar-year precision.
    Year,
    /// The source did not provide a usable precision.
    Unknown,
}

/// Whether a temporal representation is exact, bounded, or unknown.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalCertainty {
    /// Both boundaries are the same included instant.
    Exact,
    /// At least one validated boundary constrains the possible time.
    Bounded,
    /// No usable temporal boundary is known.
    Unknown,
}

/// One lower or upper interval boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TemporalBoundary<T> {
    /// No boundary is known in this direction.
    Unbounded,
    /// The boundary instant belongs to the interval.
    Included(T),
    /// The boundary instant does not belong to the interval.
    Excluded(T),
}

/// A validated interval whose boundaries use one nominal TEPP clock.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TemporalInterval<T: TemporalClock> {
    lower: TemporalBoundary<T>,
    upper: TemporalBoundary<T>,
    precision: TemporalPrecision,
    certainty: TemporalCertainty,
}

impl<T: TemporalClock> TemporalInterval<T> {
    /// Construct an exact single-instant interval.
    ///
    /// # Errors
    ///
    /// Returns [`TemporalError::InvalidTemporalPrecision`] when `precision` is
    /// [`TemporalPrecision::Unknown`].
    pub fn exact(value: T, precision: TemporalPrecision) -> Result<Self, TemporalError> {
        validate_known_precision(precision)?;
        Ok(Self {
            lower: TemporalBoundary::Included(value),
            upper: TemporalBoundary::Included(value),
            precision,
            certainty: TemporalCertainty::Exact,
        })
    }

    /// Construct a nonempty bounded or open-ended interval.
    ///
    /// At least one boundary must be known. Equal known boundaries are rejected
    /// because exact single-instant representations must use [`Self::exact`].
    /// Two excluded boundaries separated by only one nanosecond are also empty
    /// because no representable TEPP instant lies strictly between them.
    ///
    /// # Errors
    ///
    /// Returns a precision, certainty, order, or emptiness error when the
    /// proposed interval is not semantically valid.
    pub fn bounded(
        lower: TemporalBoundary<T>,
        upper: TemporalBoundary<T>,
        precision: TemporalPrecision,
    ) -> Result<Self, TemporalError> {
        validate_known_precision(precision)?;
        if matches!(lower, TemporalBoundary::Unbounded)
            && matches!(upper, TemporalBoundary::Unbounded)
        {
            return Err(TemporalError::InvalidIntervalCertainty);
        }

        if let (Some(lower_value), Some(upper_value)) =
            (boundary_value(lower), boundary_value(upper))
        {
            if lower_value > upper_value {
                return Err(TemporalError::InvalidIntervalOrder);
            }
            if lower_value == upper_value {
                return Err(TemporalError::EmptyInterval);
            }
            if matches!(lower, TemporalBoundary::Excluded(_))
                && matches!(upper, TemporalBoundary::Excluded(_))
                && upper_value.instant().as_nanosecond()
                    - lower_value.instant().as_nanosecond()
                    == 1
            {
                return Err(TemporalError::EmptyInterval);
            }
        }

        Ok(Self {
            lower,
            upper,
            precision,
            certainty: TemporalCertainty::Bounded,
        })
    }

    /// Construct an explicitly unknown interval.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            lower: TemporalBoundary::Unbounded,
            upper: TemporalBoundary::Unbounded,
            precision: TemporalPrecision::Unknown,
            certainty: TemporalCertainty::Unknown,
        }
    }

    /// Return the lower boundary.
    #[must_use]
    pub const fn lower(&self) -> TemporalBoundary<T> {
        self.lower
    }

    /// Return the upper boundary.
    #[must_use]
    pub const fn upper(&self) -> TemporalBoundary<T> {
        self.upper
    }

    /// Return the source precision.
    #[must_use]
    pub const fn precision(&self) -> TemporalPrecision {
        self.precision
    }

    /// Return whether the representation is exact, bounded, or unknown.
    #[must_use]
    pub const fn certainty(&self) -> TemporalCertainty {
        self.certainty
    }

    /// Return whether at least one usable temporal boundary is known.
    #[must_use]
    pub const fn is_known(&self) -> bool {
        !matches!(self.certainty, TemporalCertainty::Unknown)
    }

    /// Return whether `value` belongs to this interval.
    ///
    /// Unknown intervals deliberately return `false` rather than claiming that
    /// any candidate instant is supported by evidence.
    #[must_use]
    pub fn contains(&self, value: T) -> bool {
        if !self.is_known() {
            return false;
        }
        lower_contains(self.lower, &value) && upper_contains(self.upper, &value)
    }

    /// Serialize this interval through the strict versioned JSON contract.
    ///
    /// # Errors
    ///
    /// Returns [`TemporalError::InvalidWirePayload`] if serialization cannot
    /// represent this validated interval.
    pub fn to_wire_json(self) -> Result<String, TemporalError> {
        crate::wire::serialize_interval(self)
    }

    /// Reconstruct and validate an interval from versioned JSON.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, version, clock, timestamp, precision,
    /// certainty, order, or emptiness error when the payload is invalid.
    pub fn from_wire_json(payload: &str) -> Result<Self, TemporalError> {
        crate::wire::deserialize_interval(payload)
    }

    /// Return the Draft 2020-12 JSON Schema for this interval's wire record.
    #[must_use]
    pub fn wire_json_schema() -> Value {
        crate::wire::interval_json_schema::<T>()
    }
}

fn validate_known_precision(precision: TemporalPrecision) -> Result<(), TemporalError> {
    if matches!(precision, TemporalPrecision::Unknown) {
        Err(TemporalError::InvalidTemporalPrecision)
    } else {
        Ok(())
    }
}

fn boundary_value<T: Copy>(boundary: TemporalBoundary<T>) -> Option<T> {
    match boundary {
        TemporalBoundary::Unbounded => None,
        TemporalBoundary::Included(value) | TemporalBoundary::Excluded(value) => Some(value),
    }
}

fn lower_contains<T: Ord>(boundary: TemporalBoundary<T>, value: &T) -> bool {
    match boundary {
        TemporalBoundary::Unbounded => true,
        TemporalBoundary::Included(lower) => value >= &lower,
        TemporalBoundary::Excluded(lower) => value > &lower,
    }
}

fn upper_contains<T: Ord>(boundary: TemporalBoundary<T>, value: &T) -> bool {
    match boundary {
        TemporalBoundary::Unbounded => true,
        TemporalBoundary::Included(upper) => value <= &upper,
        TemporalBoundary::Excluded(upper) => value < &upper,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TemporalBoundary, TemporalCertainty, TemporalInterval, TemporalPrecision, boundary_value,
    };
    use crate::{EventTime, TemporalError};

    fn event_time(value: &str) -> EventTime {
        EventTime::parse_rfc3339(value).expect("event time must parse")
    }

    #[test]
    fn excluded_boundaries_apply_strict_inequalities() {
        let start = event_time("2026-01-01T00:00:00Z");
        let middle = event_time("2026-06-01T00:00:00Z");
        let end = event_time("2027-01-01T00:00:00Z");
        let interval = TemporalInterval::bounded(
            TemporalBoundary::Excluded(start),
            TemporalBoundary::Included(end),
            TemporalPrecision::Year,
        )
        .expect("interval must validate");

        assert!(!interval.contains(start));
        assert!(interval.contains(middle));
        assert!(interval.contains(end));
    }

    #[test]
    fn upper_open_interval_excludes_values_after_the_boundary() {
        let earlier = event_time("2025-01-01T00:00:00Z");
        let end = event_time("2026-01-01T00:00:00Z");
        let later = event_time("2027-01-01T00:00:00Z");
        let interval = TemporalInterval::bounded(
            TemporalBoundary::Unbounded,
            TemporalBoundary::Excluded(end),
            TemporalPrecision::Year,
        )
        .expect("interval must validate");

        assert!(interval.contains(earlier));
        assert!(!interval.contains(end));
        assert!(!interval.contains(later));
    }

    #[test]
    fn lower_open_interval_applies_only_the_known_boundary() {
        let earlier = event_time("2025-01-01T00:00:00Z");
        let start = event_time("2026-01-01T00:00:00Z");
        let later = event_time("2027-01-01T00:00:00Z");
        let interval = TemporalInterval::bounded(
            TemporalBoundary::Included(start),
            TemporalBoundary::Unbounded,
            TemporalPrecision::Year,
        )
        .expect("interval must validate");

        assert!(!interval.contains(earlier));
        assert!(interval.contains(start));
        assert!(interval.contains(later));
    }

    #[test]
    fn closed_interval_rejects_values_outside_either_boundary() {
        let before = event_time("2025-12-31T23:59:59Z");
        let start = event_time("2026-01-01T00:00:00Z");
        let middle = event_time("2026-06-01T00:00:00Z");
        let end = event_time("2027-01-01T00:00:00Z");
        let after = event_time("2027-01-01T00:00:01Z");
        let interval = TemporalInterval::bounded(
            TemporalBoundary::Included(start),
            TemporalBoundary::Included(end),
            TemporalPrecision::Year,
        )
        .expect("interval must validate");

        assert!(!interval.contains(before));
        assert!(interval.contains(start));
        assert!(interval.contains(middle));
        assert!(interval.contains(end));
        assert!(!interval.contains(after));
    }

    #[test]
    fn bounded_constructor_rejects_every_invalid_semantic_shape() {
        let first = event_time("2026-01-01T00:00:00Z");
        let second = event_time("2027-01-01T00:00:00Z");

        assert_eq!(
            TemporalInterval::<EventTime>::bounded(
                TemporalBoundary::Unbounded,
                TemporalBoundary::Unbounded,
                TemporalPrecision::Year,
            ),
            Err(TemporalError::InvalidIntervalCertainty)
        );
        assert_eq!(
            TemporalInterval::bounded(
                TemporalBoundary::Included(second),
                TemporalBoundary::Included(first),
                TemporalPrecision::Year,
            ),
            Err(TemporalError::InvalidIntervalOrder)
        );
        assert_eq!(
            TemporalInterval::bounded(
                TemporalBoundary::Included(first),
                TemporalBoundary::Excluded(first),
                TemporalPrecision::Year,
            ),
            Err(TemporalError::EmptyInterval)
        );
        assert_eq!(
            TemporalInterval::bounded(
                TemporalBoundary::Included(first),
                TemporalBoundary::Included(second),
                TemporalPrecision::Unknown,
            ),
            Err(TemporalError::InvalidTemporalPrecision)
        );
    }

    #[test]
    fn boundary_value_distinguishes_unbounded_and_known_values() {
        assert_eq!(boundary_value::<u8>(TemporalBoundary::Unbounded), None);
        assert_eq!(boundary_value(TemporalBoundary::Included(3_u8)), Some(3));
        assert_eq!(boundary_value(TemporalBoundary::Excluded(4_u8)), Some(4));
    }

    #[test]
    fn unknown_interval_accessors_and_containment_are_consistent() {
        let interval = TemporalInterval::<EventTime>::unknown();
        let candidate = event_time("2026-01-01T00:00:00Z");

        assert_eq!(interval.certainty(), TemporalCertainty::Unknown);
        assert_eq!(interval.precision(), TemporalPrecision::Unknown);
        assert!(!interval.is_known());
        assert!(!interval.contains(candidate));
    }

    #[test]
    fn exact_interval_rejects_unknown_precision() {
        let value = event_time("2026-01-01T00:00:00Z");
        assert_eq!(
            TemporalInterval::exact(value, TemporalPrecision::Unknown),
            Err(TemporalError::InvalidTemporalPrecision)
        );
    }
}
