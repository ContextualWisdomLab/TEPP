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
        "2026-08-06T01:30:00Zx",
        "2026-08-06T01:30:00+05:30x",
    ];

    for input in invalid {
        assert_eq!(
            EventTime::parse_rfc3339(input).unwrap_err(),
            TemporalError::InvalidTimestamp
        );
    }
}

#[test]
fn strict_parser_rejects_unknown_negative_zero_offset() {
    assert_eq!(
        EventTime::parse_rfc3339("2026-08-06T01:30:00-00:00").unwrap_err(),
        TemporalError::InvalidTimestamp
    );

    let explicit_utc = EventTime::parse_rfc3339("2026-08-06T01:30:00+00:00")
        .expect("known zero offset must parse");
    assert_eq!(explicit_utc.to_rfc3339(), "2026-08-06T01:30:00Z");
}

#[test]
fn strict_parser_rejects_empty_and_excessive_fractional_precision() {
    for input in ["2026-08-06T01:30:00.Z", "2026-08-06T01:30:00.1234567890Z"] {
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
