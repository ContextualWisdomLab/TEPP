//! Contextual commercial roles that are not permanent entity classes.

use crate::RoleContradictionError;

/// Closed vocabulary of commercial roles that can change over time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextualRole {
    /// Customer role in one commercial context.
    Customer,
    /// Partner role in one commercial context.
    Partner,
    /// Competitor role in one commercial context.
    Competitor,
}

impl ContextualRole {
    /// Return the stable wire role name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Customer => "customer",
            Self::Partner => "partner",
            Self::Competitor => "competitor",
        }
    }

    /// Parse a stable wire role name.
    ///
    /// # Errors
    ///
    /// Returns [`RoleContradictionError::InvalidRolePayload`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, RoleContradictionError> {
        match name {
            "customer" => Ok(Self::Customer),
            "partner" => Ok(Self::Partner),
            "competitor" => Ok(Self::Competitor),
            _ => Err(RoleContradictionError::InvalidRolePayload),
        }
    }
}

/// Return whether two roles contradict in the same group.
///
/// Customer and competitor cannot occupy the same group. Partner may coexist
/// with either role (coopetition or a customer-partner) without rewriting
/// entity identity.
#[must_use]
pub const fn roles_contradict(left: ContextualRole, right: ContextualRole) -> bool {
    matches!(
        (left, right),
        (ContextualRole::Customer, ContextualRole::Competitor)
            | (ContextualRole::Competitor, ContextualRole::Customer)
    )
}

/// Refuse a contradictory customer/competitor pair in one group.
///
/// # Errors
///
/// Returns [`RoleContradictionError::CustomerCompetitorOverlap`] when the pair
/// is customer and competitor. Compatible pairs succeed.
pub fn refuse_contradictory_roles(
    left: ContextualRole,
    right: ContextualRole,
) -> Result<(), RoleContradictionError> {
    if roles_contradict(left, right) {
        return Err(RoleContradictionError::CustomerCompetitorOverlap);
    }
    Ok(())
}

/// Refuse to treat a contextual role as a permanent entity class.
///
/// # Errors
///
/// Always returns [`RoleContradictionError::RoleIsNotEntityClass`].
pub fn refuse_role_as_entity_class(_role: ContextualRole) -> Result<(), RoleContradictionError> {
    Err(RoleContradictionError::RoleIsNotEntityClass)
}

/// Fraction of recovered contextual roles that match known truth.
///
/// # Errors
///
/// Returns [`RoleContradictionError::InvalidRolePayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[ContextualRole],
    decided: &[ContextualRole],
) -> Result<f64, RoleContradictionError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(RoleContradictionError::InvalidRolePayload);
    }
    let mut matches = 0_u32;
    for (truth_role, decided_role) in truth.iter().zip(decided) {
        if truth_role == decided_role {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        identity_recovery_rate, refuse_contradictory_roles, refuse_role_as_entity_class,
        roles_contradict, ContextualRole,
    };
    use crate::RoleContradictionError;

    #[test]
    fn local_branches_cover_pairs_payloads_and_wire_names() {
        assert!(roles_contradict(
            ContextualRole::Customer,
            ContextualRole::Competitor
        ));
        assert!(roles_contradict(
            ContextualRole::Competitor,
            ContextualRole::Customer
        ));
        assert!(!roles_contradict(
            ContextualRole::Customer,
            ContextualRole::Partner
        ));
        assert!(!roles_contradict(
            ContextualRole::Partner,
            ContextualRole::Competitor
        ));
        assert!(!roles_contradict(
            ContextualRole::Customer,
            ContextualRole::Customer
        ));
        assert_eq!(
            refuse_contradictory_roles(ContextualRole::Customer, ContextualRole::Competitor),
            Err(RoleContradictionError::CustomerCompetitorOverlap)
        );
        assert_eq!(
            refuse_contradictory_roles(ContextualRole::Competitor, ContextualRole::Customer),
            Err(RoleContradictionError::CustomerCompetitorOverlap)
        );
        refuse_contradictory_roles(ContextualRole::Customer, ContextualRole::Partner)
            .expect("compatible");
        refuse_contradictory_roles(ContextualRole::Partner, ContextualRole::Competitor)
            .expect("coopetition");
        for role in [
            ContextualRole::Customer,
            ContextualRole::Partner,
            ContextualRole::Competitor,
        ] {
            assert_eq!(
                ContextualRole::from_wire_name(role.wire_name()).expect("round-trip"),
                role
            );
            assert_eq!(
                refuse_role_as_entity_class(role),
                Err(RoleContradictionError::RoleIsNotEntityClass)
            );
        }
        assert_eq!(
            ContextualRole::from_wire_name("organization"),
            Err(RoleContradictionError::InvalidRolePayload)
        );
        let matched =
            identity_recovery_rate(&[ContextualRole::Customer], &[ContextualRole::Customer])
                .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(RoleContradictionError::InvalidRolePayload)
        );
        assert_eq!(
            identity_recovery_rate(&[ContextualRole::Customer], &[]),
            Err(RoleContradictionError::InvalidRolePayload)
        );
    }
}
