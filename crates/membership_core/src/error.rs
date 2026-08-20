//! Fail-closed membership-domain validation errors.

use std::fmt;

/// A fail-closed membership-domain error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MembershipError {
    /// A membership weight was negative, non-finite, or otherwise invalid.
    InvalidMembershipWeight,
    /// A validity interval was empty, ordered backward, or open-ended where a
    /// known interval is required.
    InvalidValidityInterval,
    /// A wire payload was malformed, incomplete, or used an unsupported version.
    InvalidWirePayload,
    /// A wire payload used a schema version this crate does not support.
    UnsupportedWireVersion,
    /// An assignment referenced a role string that is not a TEPP membership role.
    UnknownMembershipRole,
    /// A duplicate assignment key was rejected by the membership network.
    DuplicateMembershipAssignment,
    /// An estimator collapsed multiple active memberships into one row.
    AtomisticCollapseRefused,
}

impl fmt::Display for MembershipError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidMembershipWeight => "invalid membership weight",
            Self::InvalidValidityInterval => "invalid membership validity interval",
            Self::InvalidWirePayload => "invalid membership wire payload",
            Self::UnsupportedWireVersion => "unsupported membership wire version",
            Self::UnknownMembershipRole => "unknown membership role",
            Self::DuplicateMembershipAssignment => "duplicate membership assignment",
            Self::AtomisticCollapseRefused => "atomistic membership collapse refused",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for MembershipError {}

#[cfg(test)]
mod tests {
    use super::MembershipError;

    #[test]
    fn error_messages_are_stable_and_redacted() {
        for (error, message) in [
            (
                MembershipError::InvalidMembershipWeight,
                "invalid membership weight",
            ),
            (
                MembershipError::InvalidValidityInterval,
                "invalid membership validity interval",
            ),
            (
                MembershipError::InvalidWirePayload,
                "invalid membership wire payload",
            ),
            (
                MembershipError::UnsupportedWireVersion,
                "unsupported membership wire version",
            ),
            (
                MembershipError::UnknownMembershipRole,
                "unknown membership role",
            ),
            (
                MembershipError::DuplicateMembershipAssignment,
                "duplicate membership assignment",
            ),
            (
                MembershipError::AtomisticCollapseRefused,
                "atomistic membership collapse refused",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
