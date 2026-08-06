//! Strict syntax branches beyond calendar-semantic validation.

use temporal_core::{EventTime, TemporalError};

#[test]
fn strict_parser_rejects_non_ascii_bad_separators_and_nondigits() {
    let invalid = [
        "2026-08-06T01:30:00é",
        "2026/08-06T01:30:00Z",
        "202x-08-06T01:30:00Z",
        "2026-08-06t01:30:00Z",
        "2026-08-06T0x:30:00Z",
        "2026-08-06T01:3x:00Z",
    ];

    for input in invalid {
        assert_eq!(
            EventTime::parse_rfc3339(input).unwrap_err(),
            TemporalError::InvalidTimestamp
        );
    }
}

#[test]
fn strict_parser_rejects_bad_offset_shape_and_digits() {
    let invalid = [
        "2026-08-06T01:30:00+05-30",
        "2026-08-06T01:30:00+0x:30",
        "2026-08-06T01:30:00+05:3x",
        "2026-08-06T01:30:00z",
    ];

    for input in invalid {
        assert_eq!(
            EventTime::parse_rfc3339(input).unwrap_err(),
            TemporalError::InvalidTimestamp
        );
    }
}

#[test]
fn strict_parser_accepts_fractional_seconds_with_an_explicit_offset() {
    let value = EventTime::parse_rfc3339("2026-08-06T10:30:00.123456789+09:00")
        .expect("strict timestamp must parse");

    assert_eq!(value.to_rfc3339(), "2026-08-06T01:30:00.123456789Z");
}
