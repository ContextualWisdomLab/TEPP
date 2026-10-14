//! Fail-closed membership-domain validation errors.

use std::fmt;

/// A fail-closed membership-domain error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MembershipError {
    /// A membership weight was outside `[0, 1]`, non-finite, or otherwise invalid.
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
    /// A released projection referenced a design name not owned by Membership.
    UnknownMembershipDesign,
    /// The same member/group/role identity had overlapping or endpoint-touching validity.
    DuplicateMembershipAssignment,
    /// A full single-membership estimator cannot represent the active membership design without loss.
    SingleMembershipProfileInapplicable,
    /// A requested longitudinal observation has no active membership at its own event time.
    MissingObservationMembership,
    /// Nested ICC is undefined for cross-classified or multiple-membership designs.
    NestedIccInapplicable,
    /// Clusters or within-group residual degrees of freedom are insufficient.
    InsufficientClusterStructure,
    /// An outcome value was non-finite.
    InvalidOutcome,
    /// The same member contributed more than one nested ICC outcome.
    DuplicateOutcomeMember,
    /// An outcome member has no active nested membership at the requested time.
    UnknownOutcomeMember,
}

impl fmt::Display for MembershipError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidMembershipWeight => "invalid membership weight",
            Self::InvalidValidityInterval => "invalid membership validity interval",
            Self::InvalidWirePayload => "invalid membership wire payload",
            Self::UnsupportedWireVersion => "unsupported membership wire version",
            Self::UnknownMembershipRole => "unknown membership role",
            Self::UnknownMembershipDesign => "unknown membership design",
            Self::DuplicateMembershipAssignment => "duplicate membership assignment",
            Self::SingleMembershipProfileInapplicable => {
                "single-membership profile is inapplicable to this membership design"
            }
            Self::MissingObservationMembership => {
                "longitudinal observation has no active membership at its event time"
            }
            Self::NestedIccInapplicable => "nested ICC is inapplicable to this membership design",
            Self::InsufficientClusterStructure => "insufficient cluster structure for nested ICC",
            Self::InvalidOutcome => "invalid nested ICC outcome",
            Self::DuplicateOutcomeMember => "duplicate nested ICC outcome member",
            Self::UnknownOutcomeMember => "unknown nested ICC outcome member",
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
                MembershipError::UnknownMembershipDesign,
                "unknown membership design",
            ),
            (
                MembershipError::DuplicateMembershipAssignment,
                "duplicate membership assignment",
            ),
            (
                MembershipError::SingleMembershipProfileInapplicable,
                "single-membership profile is inapplicable to this membership design",
            ),
            (
                MembershipError::MissingObservationMembership,
                "longitudinal observation has no active membership at its event time",
            ),
            (
                MembershipError::NestedIccInapplicable,
                "nested ICC is inapplicable to this membership design",
            ),
            (
                MembershipError::InsufficientClusterStructure,
                "insufficient cluster structure for nested ICC",
            ),
            (
                MembershipError::InvalidOutcome,
                "invalid nested ICC outcome",
            ),
            (
                MembershipError::DuplicateOutcomeMember,
                "duplicate nested ICC outcome member",
            ),
            (
                MembershipError::UnknownOutcomeMember,
                "unknown nested ICC outcome member",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
