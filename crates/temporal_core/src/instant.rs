//! Absolute nanosecond-resolution instants with strict RFC 3339 input.

use crate::TemporalError;
use jiff::Timestamp;
use std::fmt;
use std::str::FromStr;

/// An absolute UTC instant represented to nanosecond precision.
///
/// This type is intentionally clock-neutral. Domain clocks wrap the same
/// absolute value in distinct nominal types so event, assertion, document,
/// system, availability, and model-cutoff time cannot be interchanged by
/// accident.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TemporalInstant(Timestamp);

impl TemporalInstant {
    /// Parse a strict RFC 3339 timestamp and normalize it to a UTC instant.
    ///
    /// Accepted input requires a four-digit date, `T` separator, explicit
    /// seconds, an optional one-to-nine-digit fractional second, and either
    /// `Z` or an exact `±HH:MM` offset. Leap-second values, bracketed time-zone
    /// annotations, shortened offsets, spaces, and offset seconds are rejected.
    ///
    /// # Errors
    ///
    /// Returns [`TemporalError::InvalidTimestamp`] when syntax or calendar
    /// semantics are invalid.
    pub fn parse_rfc3339(input: &str) -> Result<Self, TemporalError> {
        validate_strict_rfc3339_syntax(input)?;
        Timestamp::from_str(input)
            .map(Self)
            .map_err(|_| TemporalError::InvalidTimestamp)
    }

    /// Return the signed number of nanoseconds since the Unix epoch.
    #[must_use]
    pub const fn as_nanosecond(self) -> i128 {
        self.0.as_nanosecond()
    }

    /// Return a canonical UTC RFC 3339 representation.
    #[must_use]
    pub fn to_rfc3339(self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for TemporalInstant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

fn validate_strict_rfc3339_syntax(input: &str) -> Result<(), TemporalError> {
    let bytes = input.as_bytes();
    if bytes.len() < 20 || !input.is_ascii() {
        return Err(TemporalError::InvalidTimestamp);
    }
    if bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return Err(TemporalError::InvalidTimestamp);
    }
    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return Err(TemporalError::InvalidTimestamp);
        }
    }
    if &bytes[17..19] == b"60" {
        return Err(TemporalError::InvalidTimestamp);
    }

    let mut cursor = 19;
    if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
        let fraction_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        let fraction_length = cursor - fraction_start;
        if !(1..=9).contains(&fraction_length) {
            return Err(TemporalError::InvalidTimestamp);
        }
    }

    match bytes.get(cursor) {
        Some(b'Z') if cursor + 1 == bytes.len() => Ok(()),
        Some(b'+' | b'-') if cursor + 6 == bytes.len() => {
            if bytes.get(cursor + 3) != Some(&b':') {
                return Err(TemporalError::InvalidTimestamp);
            }
            for index in [cursor + 1, cursor + 2, cursor + 4, cursor + 5] {
                if !bytes[index].is_ascii_digit() {
                    return Err(TemporalError::InvalidTimestamp);
                }
            }
            Ok(())
        }
        _ => Err(TemporalError::InvalidTimestamp),
    }
}

#[cfg(test)]
mod tests {
    use super::{TemporalInstant, validate_strict_rfc3339_syntax};
    use crate::TemporalError;

    #[test]
    fn fractional_second_syntax_accepts_one_through_nine_digits() {
        for digits in 1..=9 {
            let timestamp = format!("2026-08-06T01:30:00.{}Z", "1".repeat(digits));
            assert_eq!(validate_strict_rfc3339_syntax(&timestamp), Ok(()));
        }
    }

    #[test]
    fn fractional_second_syntax_rejects_empty_or_excessive_precision() {
        assert_eq!(
            validate_strict_rfc3339_syntax("2026-08-06T01:30:00.Z"),
            Err(TemporalError::InvalidTimestamp)
        );
        assert_eq!(
            validate_strict_rfc3339_syntax("2026-08-06T01:30:00.1234567890Z"),
            Err(TemporalError::InvalidTimestamp)
        );
    }

    #[test]
    fn display_matches_canonical_rfc3339_output() {
        let instant = TemporalInstant::parse_rfc3339("2026-08-06T10:30:00+09:00")
            .expect("timestamp must parse");
        assert_eq!(instant.to_string(), "2026-08-06T01:30:00Z");
    }
}
