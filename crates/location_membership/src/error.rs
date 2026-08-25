//! Fail-closed location-membership errors.

use std::fmt;

/// A fail-closed location-membership error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LocationMembershipError {
    /// Location membership was treated as permanent entity identity.
    LocationIsNotEntityIdentity,
    /// Location membership was treated as a language channel.
    LocationIsNotLanguageChannel,
    /// A recovery slice was empty or length-mismatched.
    InvalidLocationPayload,
}

impl fmt::Display for LocationMembershipError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::LocationIsNotEntityIdentity => {
                "location membership is not permanent entity identity"
            }
            Self::LocationIsNotLanguageChannel => "location membership is not a language channel",
            Self::InvalidLocationPayload => "invalid location-membership payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for LocationMembershipError {}

#[cfg(test)]
mod tests {
    use super::LocationMembershipError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                LocationMembershipError::LocationIsNotEntityIdentity,
                "location membership is not permanent entity identity",
            ),
            (
                LocationMembershipError::LocationIsNotLanguageChannel,
                "location membership is not a language channel",
            ),
            (
                LocationMembershipError::InvalidLocationPayload,
                "invalid location-membership payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
