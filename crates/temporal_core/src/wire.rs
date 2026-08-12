//! Versioned JSON wire records and schemas for temporal domain values.

use crate::{
    TemporalBoundary, TemporalCertainty, TemporalClock, TemporalError, TemporalInstant,
    TemporalInterval, TemporalPrecision,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The only temporal JSON wire-schema version accepted by this crate.
pub const TEMPORAL_WIRE_SCHEMA_VERSION: u16 = 1;

const STRICT_TIMESTAMP_PATTERN: &str = r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]{1,9})?(?:Z|[+-][0-9]{2}:[0-9]{2})$";
const UNKNOWN_LOCAL_OFFSET_PATTERN: &str = r"-00:00$";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClockWire {
    schema_version: u16,
    clock_type: String,
    timestamp: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IntervalWire {
    schema_version: u16,
    clock_type: String,
    certainty: TemporalCertainty,
    precision: TemporalPrecision,
    lower: BoundaryWire,
    upper: BoundaryWire,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BoundaryWire {
    kind: BoundaryKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum BoundaryKind {
    Unbounded,
    Included,
    Excluded,
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn serialize_clock<T: TemporalClock>(clock: T) -> Result<String, TemporalError> {
    Ok(json!({
        "schema_version": TEMPORAL_WIRE_SCHEMA_VERSION,
        "clock_type": T::WIRE_NAME,
        "timestamp": clock.instant().to_rfc3339(),
    })
    .to_string())
}

pub(crate) fn deserialize_clock<T: TemporalClock>(payload: &str) -> Result<T, TemporalError> {
    let wire: ClockWire = deserialize_wire(payload)?;
    validate_header(wire.schema_version, &wire.clock_type, T::WIRE_NAME)?;
    TemporalInstant::parse_rfc3339(&wire.timestamp).map(T::from_instant)
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn serialize_interval<T: TemporalClock>(
    interval: TemporalInterval<T>,
) -> Result<String, TemporalError> {
    Ok(json!({
        "schema_version": TEMPORAL_WIRE_SCHEMA_VERSION,
        "clock_type": T::WIRE_NAME,
        "certainty": interval.certainty(),
        "precision": interval.precision(),
        "lower": BoundaryWire::from_boundary(interval.lower()),
        "upper": BoundaryWire::from_boundary(interval.upper()),
    })
    .to_string())
}

pub(crate) fn deserialize_interval<T: TemporalClock>(
    payload: &str,
) -> Result<TemporalInterval<T>, TemporalError> {
    let wire: IntervalWire = deserialize_wire(payload)?;
    validate_header(wire.schema_version, &wire.clock_type, T::WIRE_NAME)?;
    let lower = map_instant_boundary::<T>(wire.lower.into_instant_boundary()?);
    let upper = map_instant_boundary::<T>(wire.upper.into_instant_boundary()?);

    match wire.certainty {
        TemporalCertainty::Exact => reconstruct_exact(lower, upper, wire.precision),
        TemporalCertainty::Bounded => TemporalInterval::bounded(lower, upper, wire.precision),
        TemporalCertainty::Unknown => {
            if wire.precision == TemporalPrecision::Unknown
                && matches!(lower, TemporalBoundary::Unbounded)
                && matches!(upper, TemporalBoundary::Unbounded)
            {
                Ok(TemporalInterval::unknown())
            } else {
                Err(TemporalError::InvalidIntervalCertainty)
            }
        }
    }
}

pub(crate) fn clock_json_schema<T: TemporalClock>() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": format!("TEPP {} clock wire record", T::WIRE_NAME),
        "type": "object",
        "additionalProperties": false,
        "required": ["schema_version", "clock_type", "timestamp"],
        "properties": {
            "schema_version": {"const": TEMPORAL_WIRE_SCHEMA_VERSION},
            "clock_type": {"const": T::WIRE_NAME},
            "timestamp": timestamp_json_schema()
        }
    })
}

pub(crate) fn interval_json_schema<T: TemporalClock>() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": format!("TEPP {} interval wire record", T::WIRE_NAME),
        "description": "TEPP interval wire version 1. Runtime validation additionally enforces interval ordering and emptiness. Exact certainty requires two included boundaries with matching included timestamps; JSON Schema cannot express cross-field timestamp equality, so runtime reconstruction remains authoritative.",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "schema_version",
            "clock_type",
            "certainty",
            "precision",
            "lower",
            "upper"
        ],
        "properties": {
            "schema_version": {"const": TEMPORAL_WIRE_SCHEMA_VERSION},
            "clock_type": {"const": T::WIRE_NAME},
            "certainty": {"enum": ["exact", "bounded", "unknown"]},
            "precision": {
                "enum": [
                    "nanosecond",
                    "microsecond",
                    "millisecond",
                    "second",
                    "minute",
                    "hour",
                    "day",
                    "month",
                    "quarter",
                    "year",
                    "unknown"
                ]
            },
            "lower": boundary_json_schema(),
            "upper": boundary_json_schema()
        },
        "allOf": [
            {
                "if": {
                    "properties": {"certainty": {"const": "unknown"}},
                    "required": ["certainty"]
                },
                "then": {
                    "properties": {
                        "precision": {"const": "unknown"},
                        "lower": {
                            "properties": {"kind": {"const": "unbounded"}},
                            "required": ["kind"]
                        },
                        "upper": {
                            "properties": {"kind": {"const": "unbounded"}},
                            "required": ["kind"]
                        }
                    }
                }
            },
            {
                "if": {
                    "properties": {"certainty": {"const": "exact"}},
                    "required": ["certainty"]
                },
                "then": {
                    "properties": {
                        "precision": {"not": {"const": "unknown"}},
                        "lower": {
                            "properties": {"kind": {"const": "included"}},
                            "required": ["kind", "timestamp"]
                        },
                        "upper": {
                            "properties": {"kind": {"const": "included"}},
                            "required": ["kind", "timestamp"]
                        }
                    }
                }
            },
            {
                "if": {
                    "properties": {"certainty": {"const": "bounded"}},
                    "required": ["certainty"]
                },
                "then": {
                    "properties": {
                        "precision": {"not": {"const": "unknown"}}
                    }
                }
            }
        ]
    })
}

impl BoundaryWire {
    fn from_boundary<T: TemporalClock>(boundary: TemporalBoundary<T>) -> Self {
        match boundary {
            TemporalBoundary::Unbounded => Self {
                kind: BoundaryKind::Unbounded,
                timestamp: None,
            },
            TemporalBoundary::Included(value) => Self {
                kind: BoundaryKind::Included,
                timestamp: Some(value.instant().to_rfc3339()),
            },
            TemporalBoundary::Excluded(value) => Self {
                kind: BoundaryKind::Excluded,
                timestamp: Some(value.instant().to_rfc3339()),
            },
        }
    }

    fn into_instant_boundary(self) -> Result<TemporalBoundary<TemporalInstant>, TemporalError> {
        match (self.kind, self.timestamp) {
            (BoundaryKind::Unbounded, None) => Ok(TemporalBoundary::Unbounded),
            (BoundaryKind::Included, Some(timestamp)) => {
                TemporalInstant::parse_rfc3339(&timestamp).map(TemporalBoundary::Included)
            }
            (BoundaryKind::Excluded, Some(timestamp)) => {
                TemporalInstant::parse_rfc3339(&timestamp).map(TemporalBoundary::Excluded)
            }
            _ => Err(TemporalError::InvalidWirePayload),
        }
    }
}

fn map_instant_boundary<T: TemporalClock>(
    boundary: TemporalBoundary<TemporalInstant>,
) -> TemporalBoundary<T> {
    match boundary {
        TemporalBoundary::Unbounded => TemporalBoundary::Unbounded,
        TemporalBoundary::Included(value) => TemporalBoundary::Included(T::from_instant(value)),
        TemporalBoundary::Excluded(value) => TemporalBoundary::Excluded(T::from_instant(value)),
    }
}

fn reconstruct_exact<T: TemporalClock>(
    lower: TemporalBoundary<T>,
    upper: TemporalBoundary<T>,
    precision: TemporalPrecision,
) -> Result<TemporalInterval<T>, TemporalError> {
    match (lower, upper) {
        (TemporalBoundary::Included(lower), TemporalBoundary::Included(upper)) => {
            if lower == upper {
                TemporalInterval::exact(lower, precision)
            } else {
                Err(TemporalError::InvalidIntervalCertainty)
            }
        }
        _ => Err(TemporalError::InvalidIntervalCertainty),
    }
}

fn validate_header(
    version: u16,
    clock_type: &str,
    expected_clock_type: &str,
) -> Result<(), TemporalError> {
    if version != TEMPORAL_WIRE_SCHEMA_VERSION {
        return Err(TemporalError::UnsupportedWireVersion);
    }
    if clock_type != expected_clock_type {
        return Err(TemporalError::ClockTypeMismatch);
    }
    Ok(())
}

fn timestamp_json_schema() -> Value {
    json!({
        "type": "string",
        "format": "date-time",
        "pattern": STRICT_TIMESTAMP_PATTERN,
        "not": {"pattern": UNKNOWN_LOCAL_OFFSET_PATTERN},
        "description": "TEPP wire-version 1 strict RFC 3339 lexical profile with explicit seconds, optional one-to-nine fractional ASCII digits, and Z or a known numeric offset. Runtime parsing additionally validates Gregorian calendar values and numeric offset ranges and rejects RFC 3339's unknown -00:00 offset."
    })
}

fn boundary_json_schema() -> Value {
    json!({
        "oneOf": [
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["kind"],
                "properties": {"kind": {"const": "unbounded"}}
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["kind", "timestamp"],
                "properties": {
                    "kind": {"const": "included"},
                    "timestamp": timestamp_json_schema()
                }
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["kind", "timestamp"],
                "properties": {
                    "kind": {"const": "excluded"},
                    "timestamp": timestamp_json_schema()
                }
            }
        ]
    })
}

fn deserialize_wire<'payload, T>(payload: &'payload str) -> Result<T, TemporalError>
where
    T: Deserialize<'payload>,
{
    serde_json::from_str(payload).map_err(|_| TemporalError::InvalidWirePayload)
}

#[cfg(test)]
mod tests {
    use super::{
        BoundaryKind, BoundaryWire, ClockWire, clock_json_schema, deserialize_clock,
        deserialize_interval, deserialize_wire, interval_json_schema, map_instant_boundary,
        reconstruct_exact, serialize_clock, serialize_interval, validate_header,
    };
    use crate::{
        EventTime, TemporalBoundary, TemporalClock, TemporalError, TemporalInstant,
        TemporalInterval, TemporalPrecision,
    };
    use serde_json::json;

    #[test]
    fn generic_wire_functions_execute_in_the_library_test_binary() {
        let value =
            EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("event time must parse");
        let clock_wire = serialize_clock(value).expect("validated clock must serialize");
        assert_eq!(
            deserialize_clock::<EventTime>(&clock_wire).expect("clock must reconstruct"),
            value
        );
        assert_eq!(
            clock_json_schema::<EventTime>()["properties"]["clock_type"]["const"],
            json!("event_time")
        );

        let interval = TemporalInterval::exact(value, TemporalPrecision::Second)
            .expect("exact interval must validate");
        let interval_wire =
            serialize_interval(interval).expect("validated interval must serialize");
        assert_eq!(
            deserialize_interval::<EventTime>(&interval_wire).expect("interval must reconstruct"),
            interval
        );
        assert_eq!(
            interval_json_schema::<EventTime>()["properties"]["clock_type"]["const"],
            json!("event_time")
        );
    }

    #[test]
    fn deserialization_failures_are_redacted() {
        assert_eq!(
            deserialize_wire::<ClockWire>("not JSON").map(|_| ()),
            Err(TemporalError::InvalidWirePayload)
        );
    }

    #[test]
    fn header_validation_distinguishes_version_and_clock_failures() {
        assert_eq!(validate_header(1, "event_time", "event_time"), Ok(()));
        assert_eq!(
            validate_header(2, "event_time", "event_time"),
            Err(TemporalError::UnsupportedWireVersion)
        );
        assert_eq!(
            validate_header(1, "document_time", "event_time"),
            Err(TemporalError::ClockTypeMismatch)
        );
    }

    #[test]
    fn boundary_wire_rejects_semantically_inconsistent_timestamp_presence() {
        let unbounded_with_timestamp = BoundaryWire {
            kind: BoundaryKind::Unbounded,
            timestamp: Some("2026-01-01T00:00:00Z".to_owned()),
        };
        let included_without_timestamp = BoundaryWire {
            kind: BoundaryKind::Included,
            timestamp: None,
        };

        assert_eq!(
            unbounded_with_timestamp.into_instant_boundary(),
            Err(TemporalError::InvalidWirePayload)
        );
        assert_eq!(
            included_without_timestamp.into_instant_boundary(),
            Err(TemporalError::InvalidWirePayload)
        );
    }

    #[test]
    fn instant_boundary_mapping_preserves_all_boundary_kinds() {
        let instant =
            TemporalInstant::parse_rfc3339("2026-01-01T00:00:00Z").expect("instant must parse");

        assert_eq!(
            map_instant_boundary::<EventTime>(TemporalBoundary::Unbounded),
            TemporalBoundary::Unbounded
        );
        assert_eq!(
            map_instant_boundary::<EventTime>(TemporalBoundary::Included(instant)),
            TemporalBoundary::Included(EventTime::from_instant(instant))
        );
        assert_eq!(
            map_instant_boundary::<EventTime>(TemporalBoundary::Excluded(instant)),
            TemporalBoundary::Excluded(EventTime::from_instant(instant))
        );
    }

    #[test]
    fn exact_reconstruction_requires_equal_included_boundaries_and_known_precision() {
        let first = EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("first must parse");
        let second = EventTime::parse_rfc3339("2027-01-01T00:00:00Z").expect("second must parse");

        assert_eq!(
            reconstruct_exact(
                TemporalBoundary::Included(first),
                TemporalBoundary::Included(second),
                TemporalPrecision::Year,
            ),
            Err(TemporalError::InvalidIntervalCertainty)
        );
        assert_eq!(
            reconstruct_exact(
                TemporalBoundary::Excluded(first),
                TemporalBoundary::Included(first),
                TemporalPrecision::Year,
            ),
            Err(TemporalError::InvalidIntervalCertainty)
        );
        assert_eq!(
            reconstruct_exact(
                TemporalBoundary::Included(first),
                TemporalBoundary::Included(first),
                TemporalPrecision::Unknown,
            ),
            Err(TemporalError::InvalidTemporalPrecision)
        );
        assert_eq!(
            reconstruct_exact(
                TemporalBoundary::Included(first),
                TemporalBoundary::Included(first),
                TemporalPrecision::Year,
            ),
            TemporalInterval::exact(first, TemporalPrecision::Year)
        );
    }
}
