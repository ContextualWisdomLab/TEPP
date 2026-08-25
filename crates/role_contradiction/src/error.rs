//! Fail-closed role-contradiction errors.

use std::fmt;

/// A fail-closed role-contradiction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RoleContradictionError {
    /// Customer and competitor were assigned in the same group.
    CustomerCompetitorOverlap,
    /// A contextual role was treated as a permanent entity class.
    RoleIsNotEntityClass,
    /// A recovery slice was empty or length-mismatched.
    InvalidRolePayload,
}

impl fmt::Display for RoleContradictionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::CustomerCompetitorOverlap => {
                "customer and competitor cannot occupy the same group"
            }
            Self::RoleIsNotEntityClass => "contextual role is not an entity class",
            Self::InvalidRolePayload => "invalid role-contradiction payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RoleContradictionError {}

#[cfg(test)]
mod tests {
    use super::RoleContradictionError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                RoleContradictionError::CustomerCompetitorOverlap,
                "customer and competitor cannot occupy the same group",
            ),
            (
                RoleContradictionError::RoleIsNotEntityClass,
                "contextual role is not an entity class",
            ),
            (
                RoleContradictionError::InvalidRolePayload,
                "invalid role-contradiction payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
