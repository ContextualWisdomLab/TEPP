//! Fail-closed subevent-containment errors.

use std::fmt;

/// A fail-closed subevent-containment error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SubeventContainmentError {
    /// The subevent interval is not inside the parent interval.
    SubeventEscapesParent,
    /// An interval or recovery slice was empty, inverted, or length-mismatched.
    InvalidIntervalPayload,
}

impl fmt::Display for SubeventContainmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SubeventEscapesParent => "subevent interval escapes the parent event",
            Self::InvalidIntervalPayload => "invalid subevent-containment payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SubeventContainmentError {}

#[cfg(test)]
mod tests {
    use super::SubeventContainmentError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                SubeventContainmentError::SubeventEscapesParent,
                "subevent interval escapes the parent event",
            ),
            (
                SubeventContainmentError::InvalidIntervalPayload,
                "invalid subevent-containment payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
