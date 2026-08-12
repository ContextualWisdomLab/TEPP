//! True and noisy observed relations for recovery studies.

use uuid::Uuid;

/// Closed simulation relation vocabulary for truth corpora.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SimulatedRelationKind {
    /// Forward state transition between latent events.
    TransitionsTo,
    /// Document revision provenance.
    Revises,
    /// Retrospective reporting of an earlier event.
    RetrospectivelyReports,
    /// Template/copy provenance between documents.
    TemplateCopyOf,
    /// Generic citation/reference noise target.
    References,
}

impl SimulatedRelationKind {
    /// Stable wire name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::TransitionsTo => "transitions_to",
            Self::Revises => "revises",
            Self::RetrospectivelyReports => "retrospectively_reports",
            Self::TemplateCopyOf => "template_copy_of",
            Self::References => "references",
        }
    }

    /// Whether this kind is a forward state-transition edge.
    #[must_use]
    pub const fn is_transition(self) -> bool {
        matches!(self, Self::TransitionsTo)
    }
}

/// One true generative relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrueRelation {
    relation_id: Uuid,
    kind: SimulatedRelationKind,
    source_id: Uuid,
    target_id: Uuid,
}

impl TrueRelation {
    /// Construct a true relation.
    #[must_use]
    pub const fn new(
        relation_id: Uuid,
        kind: SimulatedRelationKind,
        source_id: Uuid,
        target_id: Uuid,
    ) -> Self {
        Self {
            relation_id,
            kind,
            source_id,
            target_id,
        }
    }

    /// Relation identity.
    #[must_use]
    pub const fn relation_id(&self) -> Uuid {
        self.relation_id
    }

    /// Relation kind.
    #[must_use]
    pub const fn kind(&self) -> SimulatedRelationKind {
        self.kind
    }

    /// Source endpoint identity.
    #[must_use]
    pub const fn source_id(&self) -> Uuid {
        self.source_id
    }

    /// Target endpoint identity.
    #[must_use]
    pub const fn target_id(&self) -> Uuid {
        self.target_id
    }
}

/// One observed relation after noise application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedRelation {
    relation_id: Uuid,
    kind: SimulatedRelationKind,
    source_id: Uuid,
    target_id: Uuid,
    is_true_positive: bool,
}

impl ObservedRelation {
    /// Construct an observed relation.
    #[must_use]
    pub const fn new(
        relation_id: Uuid,
        kind: SimulatedRelationKind,
        source_id: Uuid,
        target_id: Uuid,
        is_true_positive: bool,
    ) -> Self {
        Self {
            relation_id,
            kind,
            source_id,
            target_id,
            is_true_positive,
        }
    }

    /// Relation identity.
    #[must_use]
    pub const fn relation_id(&self) -> Uuid {
        self.relation_id
    }

    /// Relation kind.
    #[must_use]
    pub const fn kind(&self) -> SimulatedRelationKind {
        self.kind
    }

    /// Source endpoint identity.
    #[must_use]
    pub const fn source_id(&self) -> Uuid {
        self.source_id
    }

    /// Target endpoint identity.
    #[must_use]
    pub const fn target_id(&self) -> Uuid {
        self.target_id
    }

    /// Whether the observation matches a true generative edge.
    #[must_use]
    pub const fn is_true_positive(&self) -> bool {
        self.is_true_positive
    }
}

#[cfg(test)]
mod tests {
    use super::{ObservedRelation, SimulatedRelationKind, TrueRelation};
    use uuid::Uuid;

    #[test]
    fn relation_kinds_and_rows_expose_stable_accessors() {
        assert_eq!(
            SimulatedRelationKind::TransitionsTo.wire_name(),
            "transitions_to"
        );
        assert_eq!(SimulatedRelationKind::Revises.wire_name(), "revises");
        assert_eq!(
            SimulatedRelationKind::RetrospectivelyReports.wire_name(),
            "retrospectively_reports"
        );
        assert_eq!(
            SimulatedRelationKind::TemplateCopyOf.wire_name(),
            "template_copy_of"
        );
        assert_eq!(SimulatedRelationKind::References.wire_name(), "references");
        assert!(SimulatedRelationKind::TransitionsTo.is_transition());
        assert!(!SimulatedRelationKind::References.is_transition());

        let id = Uuid::nil();
        let true_rel = TrueRelation::new(id, SimulatedRelationKind::Revises, id, Uuid::max());
        assert_eq!(true_rel.relation_id(), id);
        assert_eq!(true_rel.kind(), SimulatedRelationKind::Revises);
        assert_eq!(true_rel.source_id(), id);
        assert_eq!(true_rel.target_id(), Uuid::max());

        let observed = ObservedRelation::new(
            id,
            SimulatedRelationKind::References,
            id,
            Uuid::max(),
            false,
        );
        assert_eq!(observed.relation_id(), id);
        assert_eq!(observed.kind(), SimulatedRelationKind::References);
        assert_eq!(observed.source_id(), id);
        assert_eq!(observed.target_id(), Uuid::max());
        assert!(!observed.is_true_positive());
    }
}
