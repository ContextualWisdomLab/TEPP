//! Fail-closed episode-membership errors.

use std::fmt;

/// A fail-closed episode-membership error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EpisodeMembershipError {
    /// A membership window started after it ended.
    InvertedEventWindow,
    /// A membership window escaped the episode interval.
    MembershipEscapesEpisode,
    /// A recovery slice was empty or length-mismatched.
    InvalidEpisodePayload,
}

impl fmt::Display for EpisodeMembershipError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvertedEventWindow => "event-time window is inverted",
            Self::MembershipEscapesEpisode => {
                "episode membership cannot escape the episode interval"
            }
            Self::InvalidEpisodePayload => "invalid episode-membership payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for EpisodeMembershipError {}

#[cfg(test)]
mod tests {
    use super::EpisodeMembershipError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                EpisodeMembershipError::InvertedEventWindow,
                "event-time window is inverted",
            ),
            (
                EpisodeMembershipError::MembershipEscapesEpisode,
                "episode membership cannot escape the episode interval",
            ),
            (
                EpisodeMembershipError::InvalidEpisodePayload,
                "invalid episode-membership payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
