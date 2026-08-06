//! Versioned JSON wire records and schemas for temporal domain values.

use crate::{
    TemporalBoundary, TemporalCertainty, TemporalClock, TemporalError, TemporalInstant,
    TemporalInterval, TemporalPrecision,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The only temporal JSON wire-schema version accepted by this crate.
pub const TEMPORAL_WIRE_SCHEMA_VERSION: u16 = 1;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ClockWire {
    schema_version: u16,
    clock_type: String,
    timestamp: String,
}

#[derive(Deserialize, Serialize)]
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

pub(crate) fn serialize_clock<T: TemporalClock>(clock: T) -> Result<String, TemporalError> {
    serialize_wire(&ClockWire {
        schema_version: TEMPORAL_WIRE_SCHEMA_VERSION,
        clock_type: T::WIRE_NAME.to_owned(),
        timestamp: clock.instant().to_rfc3339(),
    })
}

pub(crate) fn deserialize_clock<T: TemporalClock>(payload: &str) -> Result<T, TemporalError> {
    let wire: ClockWire = deserialize_wire(payload)?;
    validate_header::<T>(wire.schema_version, &wire.clock_type)?;
    TemporalInstant::parse_rfc3339(&wire.timestamp).map(T::from_instant)
}

pub(crate) fn serialize_interval<T: TemporalClock>(
    interval: TemporalInterval<T>,
) -> Result<String, TemporalError> {
    serialize_wire(&IntervalWire {
        schema_version: TEMPORAL_WIRE_SCHEMA_VERSION,
        clock_type: T::WIRE_NAME.to_owned(),
        certainty: interval.certainty(),
        precision: interval.precision(),
        lower: BoundaryWire::from_boundary(interval.lower()),
        upper: BoundaryWire::from_boundary(interval.upper()),
    })
}

pub(crate) fn deserialize_interval<T: TemporalClock>(
    payload: &str,
) -> Result<TemporalInterval<T>, TemporalError> {
    let wire: IntervalWire = deserialize_wire(payload)?;
    validate_header::<T>(wire.schema_version, &wire.clock_type)?;
    let lower = wire.lower.into_boundary::<T>()?;
    let upper = wire.upper.into_boundary::<T>()?;

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
            "timestamp": {
                "type": "string",
                "format": "date-time",
                "description": "Strict RFC 3339 timestamp with explicit seconds and offset."
            }
        }
    })
}

pub(crate) fn interval_json_schema<T: TemporalClock>() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": format!("TEPP {} interval wire record", T::WIRE_NAME),
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
        }
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

    fn into_boundary<T: TemporalClock>(self) -> Result<TemporalBoundary<T>, TemporalError> {
        match (self.kind, self.timestamp) {
            (BoundaryKind::Unbounded, None) => Ok(TemporalBoundary::Unbounded),
            (BoundaryKind::Included, Some(timestamp)) => TemporalInstant::parse_rfc3339(&timestamp)
                .map(T::from_instant)
                .map(TemporalBoundary::Included),
            (BoundaryKind::Excluded, Some(timestamp)) => TemporalInstant::parse_rfc3339(&timestamp)
                .map(T::from_instant)
                .map(TemporalBoundary::Excluded),
            _ => Err(TemporalError::InvalidWirePayload),
        }
    }
}

fn reconstruct_exact<T: TemporalClock>(
    lower: TemporalBoundary<T>,
    upper: TemporalBoundary<T>,
    precision: TemporalPrecision,
) -> Result<TemporalInterval<T>, TemporalError> {
    match (lower, upper) {
        (TemporalBoundary::Included(lower), TemporalBoundary::Included(upper))
            if lower == upper =>
        {
            TemporalInterval::exact(lower, precision)
        }
        _ => Err(TemporalError::InvalidIntervalCertainty),
    }
}

fn validate_header<T: TemporalClock>(
    version: u16,
    clock_type: &str,
) -> Result<(), TemporalError> {
    if version != TEMPORAL_WIRE_SCHEMA_VERSION {
        return Err(TemporalError::UnsupportedWireVersion);
    }
    if clock_type != T::WIRE_NAME {
        return Err(TemporalError::ClockTypeMismatch);
    }
    Ok(())
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
                    "timestamp": {"type": "string", "format": "date-time"}
                }
            },
            {
                "type": "object",
                "additionalProperties": false,
                "required": ["kind", "timestamp"],
                "properties": {
                    "kind": {"const": "excluded"},
                    "timestamp": {"type": "string", "format": "date-time"}
                }
            }
        ]
    })
}

fn serialize_wire<T: Serialize>(value: &T) -> Result<String, TemporalError> {
    serde_json::to_string(value).map_err(|_| TemporalError::InvalidWirePayload)
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
        BoundaryKind, BoundaryWire, deserialize_wire, reconstruct_exact, serialize_wire,
        validate_header,
    };
    use crate::{
        EventTime, TemporalBoundary, TemporalError, TemporalInterval, TemporalPrecision,
    };
    use serde::Serialize;
    use serde::ser::Serializer;

    struct SerializationFailure;

    impl Serialize for SerializationFailure {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("intentional test failure"))
        }
    }

    #[test]
    fn serialization_and_deserialization_failures_are_redacted() {
        assert_eq!(
            serialize_wire(&SerializationFailure),
            Err(TemporalError::InvalidWirePayload)
        );
        assert_eq!(
            deserialize_wire::<Vec<u8>>("not JSON"),
            Err(TemporalError::InvalidWirePayload)
        );
    }

    #[test]
    fn header_validation_distinguishes_version_and_clock_failures() {
        assert_eq!(validate_header::<EventTime>(1, "event_time"), Ok(()));
        assert_eq!(
            validate_header::<EventTime>(2, "event_time"),
            Err(TemporalError::UnsupportedWireVersion)
        );
        assert_eq!(
            validate_header::<EventTime>(1, "document_time"),
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
            unbounded_with_timestamp.into_boundary::<EventTime>(),
            Err(TemporalError::InvalidWirePayload)
        );
        assert_eq!(
            included_without_timestamp.into_boundary::<EventTime>(),
            Err(TemporalError::InvalidWirePayload)
        );
    }

    #[test]
    fn exact_reconstruction_requires_equal_included_boundaries() {
        let first = EventTime::parse_rfc3339("2026-01-01T00:00:00Z")
            .expect("first must parse");
        let second = EventTime::parse_rfc3339("2027-01-01T00:00:00Z")
            .expect("second must parse");

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
                TemporalBoundary::Included(first),
                TemporalBoundary::Included(first),
                TemporalPrecision::Year,
            ),
            TemporalInterval::exact(first, TemporalPrecision::Year)
        );
    }
}
