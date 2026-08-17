//! Typed membership targets that must stay distinct from entity and project.

use crate::MembershipTargetError;

/// Closed vocabulary of membership targets beyond a single entity/project pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MembershipTargetKind {
    /// Organization or person identity under role assignments.
    Entity,
    /// Project or program.
    Project,
    /// Organizational department or team.
    Department,
    /// Document or prompt template family.
    Template,
    /// Language community or locale channel.
    Language,
    /// Opportunity pool or deal context.
    OpportunityPool,
    /// Temporal episode or campaign.
    Episode,
}

impl MembershipTargetKind {
    /// Return the stable wire target name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Project => "project",
            Self::Department => "department",
            Self::Template => "template",
            Self::Language => "language",
            Self::OpportunityPool => "opportunity_pool",
            Self::Episode => "episode",
        }
    }

    /// Parse a stable wire target name.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipTargetError::InvalidTargetPayload`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, MembershipTargetError> {
        match name {
            "entity" => Ok(Self::Entity),
            "project" => Ok(Self::Project),
            "department" => Ok(Self::Department),
            "template" => Ok(Self::Template),
            "language" => Ok(Self::Language),
            "opportunity_pool" => Ok(Self::OpportunityPool),
            "episode" => Ok(Self::Episode),
            _ => Err(MembershipTargetError::InvalidTargetPayload),
        }
    }
}

/// Refuse to treat one target kind as another.
///
/// Persistence currently stores only entity or project. Language, episode,
/// template, opportunity-pool, and department memberships must stay typed.
///
/// # Errors
///
/// Returns [`MembershipTargetError::TargetKindCollapsed`] when `kind` and
/// `collapsed_as` differ.
pub fn refuse_collapsed_target(
    kind: MembershipTargetKind,
    collapsed_as: MembershipTargetKind,
) -> Result<(), MembershipTargetError> {
    if kind == collapsed_as {
        return Ok(());
    }
    Err(MembershipTargetError::TargetKindCollapsed)
}

/// Fraction of recovered target kinds that match known truth.
///
/// # Errors
///
/// Returns [`MembershipTargetError::InvalidTargetPayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[MembershipTargetKind],
    decided: &[MembershipTargetKind],
) -> Result<f64, MembershipTargetError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(MembershipTargetError::InvalidTargetPayload);
    }
    let mut matches = 0_u32;
    for (truth_kind, decided_kind) in truth.iter().zip(decided) {
        if truth_kind == decided_kind {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{identity_recovery_rate, refuse_collapsed_target, MembershipTargetKind};
    use crate::MembershipTargetError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_collapsed_target(MembershipTargetKind::Language, MembershipTargetKind::Entity),
            Err(MembershipTargetError::TargetKindCollapsed)
        );
        assert_eq!(
            refuse_collapsed_target(MembershipTargetKind::Episode, MembershipTargetKind::Entity),
            Err(MembershipTargetError::TargetKindCollapsed)
        );
        assert_eq!(
            refuse_collapsed_target(
                MembershipTargetKind::Department,
                MembershipTargetKind::Entity
            ),
            Err(MembershipTargetError::TargetKindCollapsed)
        );
        assert_eq!(
            refuse_collapsed_target(
                MembershipTargetKind::Template,
                MembershipTargetKind::Project
            ),
            Err(MembershipTargetError::TargetKindCollapsed)
        );
        assert_eq!(
            refuse_collapsed_target(
                MembershipTargetKind::OpportunityPool,
                MembershipTargetKind::Project
            ),
            Err(MembershipTargetError::TargetKindCollapsed)
        );
        refuse_collapsed_target(MembershipTargetKind::Entity, MembershipTargetKind::Entity)
            .expect("entity");
        refuse_collapsed_target(MembershipTargetKind::Project, MembershipTargetKind::Project)
            .expect("project");
        for kind in [
            MembershipTargetKind::Entity,
            MembershipTargetKind::Project,
            MembershipTargetKind::Department,
            MembershipTargetKind::Template,
            MembershipTargetKind::Language,
            MembershipTargetKind::OpportunityPool,
            MembershipTargetKind::Episode,
        ] {
            assert_eq!(
                MembershipTargetKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            MembershipTargetKind::from_wire_name("membership_target_id"),
            Err(MembershipTargetError::InvalidTargetPayload)
        );
        let matched = identity_recovery_rate(
            &[MembershipTargetKind::Language],
            &[MembershipTargetKind::Language],
        )
        .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(MembershipTargetError::InvalidTargetPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[MembershipTargetKind::Language], &[]),
            Err(MembershipTargetError::InvalidTargetPayload)
        );
    }
}
