//! Bounded path-consistency closure for qualitative interval constraints.

use crate::RelationSet;
use std::collections::BTreeSet;
use std::fmt;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ReasonerInstanceId(Uuid);

impl ReasonerInstanceId {
    fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

/// An opaque identifier for one interval variable in a reasoner instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TemporalVariableId {
    reasoner_instance_id: ReasonerInstanceId,
    variable_index: usize,
}

/// An opaque identifier for one accepted relation assertion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConstraintId {
    reasoner_instance_id: ReasonerInstanceId,
    constraint_index: usize,
}

impl ConstraintId {
    /// Return the zero-based accepted-assertion ordinal within this reasoner.
    #[must_use]
    pub const fn assertion_ordinal(self) -> usize {
        self.constraint_index
    }
}

/// The bounded resource whose configured maximum was exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasonerLimitKind {
    /// The maximum number of interval variables.
    Variables,
    /// The maximum number of accepted constraints.
    Constraints,
    /// The maximum number of path-consistency propagation steps.
    PropagationSteps,
}

/// Explicit capacity bounds for one temporal reasoner instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemporalReasonerLimits {
    variable_limit: usize,
    constraint_limit: usize,
    propagation_budget: usize,
}

impl TemporalReasonerLimits {
    /// Validate nonzero variable, constraint, and propagation limits.
    ///
    /// # Errors
    ///
    /// Returns [`TemporalReasonerError::InvalidLimits`] when any maximum is
    /// zero.
    pub const fn new(
        maximum_variables: usize,
        maximum_constraints: usize,
        maximum_propagation_steps: usize,
    ) -> Result<Self, TemporalReasonerError> {
        if maximum_variables == 0 || maximum_constraints == 0 || maximum_propagation_steps == 0 {
            Err(TemporalReasonerError::InvalidLimits)
        } else {
            Ok(Self {
                variable_limit: maximum_variables,
                constraint_limit: maximum_constraints,
                propagation_budget: maximum_propagation_steps,
            })
        }
    }
}

/// Evidence that a qualitative temporal network has no possible relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalContradiction {
    left: TemporalVariableId,
    right: TemporalVariableId,
    support: Vec<ConstraintId>,
    attempted_relations: Option<RelationSet>,
}

impl TemporalContradiction {
    /// Return the left variable of the contradictory pair.
    #[must_use]
    pub const fn left(&self) -> TemporalVariableId {
        self.left
    }

    /// Return the right variable of the contradictory pair.
    #[must_use]
    pub const fn right(&self) -> TemporalVariableId {
        self.right
    }

    /// Return accepted assertions supporting the contradiction.
    #[must_use]
    pub fn support(&self) -> &[ConstraintId] {
        &self.support
    }

    /// Return the rejected direct assertion when contradiction occurred before closure.
    #[must_use]
    pub const fn attempted_relations(&self) -> Option<RelationSet> {
        self.attempted_relations
    }

    fn from_support(
        left: TemporalVariableId,
        right: TemporalVariableId,
        support: BTreeSet<ConstraintId>,
        attempted_relations: Option<RelationSet>,
    ) -> Self {
        Self {
            left,
            right,
            support: support.into_iter().collect(),
            attempted_relations,
        }
    }
}

impl fmt::Display for TemporalContradiction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("temporal relation network is contradictory")
    }
}

impl std::error::Error for TemporalContradiction {}

/// A fail-closed temporal-reasoner error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TemporalReasonerError {
    /// One or more configured resource limits were zero.
    InvalidLimits,
    /// A variable identifier does not belong to this reasoner instance.
    UnknownVariable,
    /// An asserted constraint supplied no possible elementary relation.
    EmptyRelationSet,
    /// A configured reasoner resource maximum was exceeded.
    LimitExceeded(ReasonerLimitKind),
    /// An assertion or propagation step proved that no relation remains possible.
    Contradiction(TemporalContradiction),
}

impl fmt::Display for TemporalReasonerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidLimits => "invalid temporal reasoner limits",
            Self::UnknownVariable => "unknown temporal reasoner variable",
            Self::EmptyRelationSet => "temporal relation set is empty",
            Self::LimitExceeded(_) => "temporal reasoner resource limit exceeded",
            Self::Contradiction(_) => "temporal relation network is contradictory",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TemporalReasonerError {}

/// One observed or derived relation returned by the reasoner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedRelation {
    relations: RelationSet,
    observed: bool,
    support: Vec<ConstraintId>,
}

impl DerivedRelation {
    /// Return the currently possible elementary relations.
    #[must_use]
    pub const fn relations(&self) -> RelationSet {
        self.relations
    }

    /// Return whether at least one direct assertion exists for this ordered pair.
    #[must_use]
    pub const fn is_observed(&self) -> bool {
        self.observed
    }

    /// Return the conservative accepted-assertion support for this relation.
    #[must_use]
    pub fn support(&self) -> &[ConstraintId] {
        &self.support
    }
}

/// Summary of one successful bounded closure operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClosureReport {
    revisions: usize,
    propagation_steps: usize,
}

impl ClosureReport {
    /// Return whether closure narrowed at least one relation set.
    #[must_use]
    pub const fn changed(self) -> bool {
        self.revisions != 0
    }

    /// Return the number of relation-set revisions.
    #[must_use]
    pub const fn revisions(self) -> usize {
        self.revisions
    }

    /// Return the number of bounded path-consistency checks performed.
    #[must_use]
    pub const fn propagation_steps(self) -> usize {
        self.propagation_steps
    }
}

#[derive(Clone, Debug)]
struct RelationCell {
    relations: RelationSet,
    observed: bool,
    support: BTreeSet<ConstraintId>,
}

impl RelationCell {
    fn unconstrained() -> Self {
        Self {
            relations: RelationSet::all(),
            observed: false,
            support: BTreeSet::new(),
        }
    }

    fn identity() -> Self {
        Self {
            relations: RelationSet::singleton(crate::AllenRelation::Equals),
            observed: false,
            support: BTreeSet::new(),
        }
    }
}

/// A bounded qualitative interval-constraint network.
///
/// The reasoner stores direct assertions separately from derived narrowing,
/// propagates inverse relations, and applies path consistency until stable.
/// Failed closure is atomic: contradiction or resource exhaustion restores the
/// network to its pre-closure state. Opaque variable and constraint identifiers
/// are scoped to one reasoner instance and fail closed when mixed across
/// instances.
pub struct TemporalReasoner {
    reasoner_instance_id: ReasonerInstanceId,
    limits: TemporalReasonerLimits,
    cells: Vec<Vec<RelationCell>>,
    constraint_count: usize,
}

impl TemporalReasoner {
    /// Create an empty reasoner with validated explicit limits.
    #[must_use]
    pub fn with_limits(limits: TemporalReasonerLimits) -> Self {
        Self {
            reasoner_instance_id: ReasonerInstanceId::new(),
            limits,
            cells: Vec::new(),
            constraint_count: 0,
        }
    }

    /// Number of variables currently owned by this reasoner.
    #[must_use]
    pub fn variable_count(&self) -> usize {
        self.cells.len()
    }

    /// Add one interval variable.
    ///
    /// # Errors
    ///
    /// Returns [`TemporalReasonerError::LimitExceeded`] when the configured
    /// variable capacity is exhausted.
    pub fn add_variable(&mut self) -> Result<TemporalVariableId, TemporalReasonerError> {
        if self.cells.len() >= self.limits.variable_limit {
            return Err(TemporalReasonerError::LimitExceeded(
                ReasonerLimitKind::Variables,
            ));
        }

        let identifier = self.variable_id(self.cells.len());
        for row in &mut self.cells {
            row.push(RelationCell::unconstrained());
        }
        let mut new_row = vec![RelationCell::unconstrained(); self.cells.len() + 1];
        new_row[identifier.variable_index] = RelationCell::identity();
        self.cells.push(new_row);
        Ok(identifier)
    }

    /// Assert a nonempty relation set for an ordered variable pair.
    ///
    /// The reverse pair is narrowed by the inverse relation set. The returned
    /// identifier is retained as provenance for later derived relations.
    /// Rejected assertions neither consume constraint capacity nor receive an
    /// accepted [`ConstraintId`].
    ///
    /// # Errors
    ///
    /// Returns an unknown-variable, empty-set, capacity, or contradiction error
    /// when the assertion cannot be accepted.
    pub fn assert_relation(
        &mut self,
        left: TemporalVariableId,
        right: TemporalVariableId,
        relations: RelationSet,
    ) -> Result<ConstraintId, TemporalReasonerError> {
        self.validate_pair(left, right)?;
        if relations.is_empty() {
            return Err(TemporalReasonerError::EmptyRelationSet);
        }
        if self.constraint_count >= self.limits.constraint_limit {
            return Err(TemporalReasonerError::LimitExceeded(
                ReasonerLimitKind::Constraints,
            ));
        }

        let narrowed = self.cells[left.variable_index][right.variable_index]
            .relations
            .intersection(relations);
        if narrowed.is_empty() {
            let support = self.cells[left.variable_index][right.variable_index]
                .support
                .clone();
            return Err(TemporalReasonerError::Contradiction(
                TemporalContradiction::from_support(left, right, support, Some(relations)),
            ));
        }

        let identifier = self.constraint_id(self.constraint_count);
        let inverse_was_observed = self.cells[right.variable_index][left.variable_index].observed;
        let mut support = self.cells[left.variable_index][right.variable_index]
            .support
            .clone();
        support.insert(identifier);
        self.cells[left.variable_index][right.variable_index] = RelationCell {
            relations: narrowed,
            observed: true,
            support: support.clone(),
        };
        self.cells[right.variable_index][left.variable_index] = RelationCell {
            relations: narrowed.inverse(),
            observed: inverse_was_observed || left == right,
            support,
        };
        self.constraint_count += 1;
        Ok(identifier)
    }

    /// Apply bounded path-consistency closure atomically.
    ///
    /// # Errors
    ///
    /// Returns contradiction evidence when a pair becomes impossible, or a
    /// propagation-step limit error when closure exceeds its configured budget.
    pub fn close(&mut self) -> Result<ClosureReport, TemporalReasonerError> {
        let snapshot = self.cells.clone();
        match self.close_in_place() {
            Ok(report) => Ok(report),
            Err(error) => {
                self.cells = snapshot;
                Err(error)
            }
        }
    }

    /// Return the current relation and conservative provenance for an ordered pair.
    ///
    /// # Errors
    ///
    /// Returns [`TemporalReasonerError::UnknownVariable`] when either identifier
    /// does not belong to this reasoner.
    pub fn relation(
        &self,
        left: TemporalVariableId,
        right: TemporalVariableId,
    ) -> Result<DerivedRelation, TemporalReasonerError> {
        self.validate_pair(left, right)?;
        let cell = &self.cells[left.variable_index][right.variable_index];
        Ok(DerivedRelation {
            relations: cell.relations,
            observed: cell.observed,
            support: cell.support.iter().copied().collect(),
        })
    }

    fn variable_id(&self, variable_index: usize) -> TemporalVariableId {
        TemporalVariableId {
            reasoner_instance_id: self.reasoner_instance_id,
            variable_index,
        }
    }

    fn constraint_id(&self, constraint_index: usize) -> ConstraintId {
        ConstraintId {
            reasoner_instance_id: self.reasoner_instance_id,
            constraint_index,
        }
    }

    fn validate_pair(
        &self,
        left: TemporalVariableId,
        right: TemporalVariableId,
    ) -> Result<(), TemporalReasonerError> {
        // Variable indices are private to this reasoner and only issued by
        // `add_variable`. Public API therefore cannot construct an in-instance
        // out-of-range index; identity isolation is the enforceable fail-closed
        // boundary for foreign or forged identifiers.
        if left.reasoner_instance_id != self.reasoner_instance_id
            || right.reasoner_instance_id != self.reasoner_instance_id
        {
            Err(TemporalReasonerError::UnknownVariable)
        } else {
            debug_assert!(left.variable_index < self.cells.len());
            debug_assert!(right.variable_index < self.cells.len());
            Ok(())
        }
    }

    fn close_in_place(&mut self) -> Result<ClosureReport, TemporalReasonerError> {
        let mut revisions = 0_usize;
        let mut propagation_steps = 0_usize;
        let variable_count = self.cells.len();

        loop {
            let mut changed = false;
            for left in 0..variable_count {
                for right in 0..variable_count {
                    if left == right {
                        continue;
                    }
                    for middle in 0..variable_count {
                        if middle == left || middle == right {
                            continue;
                        }
                        if propagation_steps >= self.limits.propagation_budget {
                            return Err(TemporalReasonerError::LimitExceeded(
                                ReasonerLimitKind::PropagationSteps,
                            ));
                        }
                        propagation_steps += 1;

                        let composed = self.cells[left][middle]
                            .relations
                            .compose(self.cells[middle][right].relations);
                        let current = self.cells[left][right].relations;
                        let narrowed = current.intersection(composed);
                        if narrowed.is_empty() {
                            let support = union_support([
                                &self.cells[left][right].support,
                                &self.cells[left][middle].support,
                                &self.cells[middle][right].support,
                            ]);
                            return Err(TemporalReasonerError::Contradiction(
                                TemporalContradiction::from_support(
                                    self.variable_id(left),
                                    self.variable_id(right),
                                    support,
                                    None,
                                ),
                            ));
                        }
                        if narrowed == current {
                            continue;
                        }

                        let support = union_support([
                            &self.cells[left][right].support,
                            &self.cells[left][middle].support,
                            &self.cells[middle][right].support,
                        ]);
                        let observed = self.cells[left][right].observed;
                        let inverse_observed = self.cells[right][left].observed;
                        self.cells[left][right] = RelationCell {
                            relations: narrowed,
                            observed,
                            support: support.clone(),
                        };
                        self.cells[right][left] = RelationCell {
                            relations: narrowed.inverse(),
                            observed: inverse_observed,
                            support,
                        };
                        revisions += 1;
                        changed = true;
                    }
                }
            }
            if !changed {
                return Ok(ClosureReport {
                    revisions,
                    propagation_steps,
                });
            }
        }
    }
}

fn union_support<const COUNT: usize>(
    supports: [&BTreeSet<ConstraintId>; COUNT],
) -> BTreeSet<ConstraintId> {
    let mut result = BTreeSet::new();
    for support in supports {
        result.extend(support.iter().copied());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        TemporalReasoner, TemporalReasonerError, TemporalReasonerLimits, TemporalVariableId,
    };

    #[test]
    fn validate_pair_rejects_each_foreign_instance_position() {
        let limits = TemporalReasonerLimits::new(2, 2, 10).expect("limits must validate");
        let mut local = TemporalReasoner::with_limits(limits);
        let mut foreign = TemporalReasoner::with_limits(limits);
        let local_variable = local.add_variable().expect("local variable must fit");
        let foreign_variable = foreign.add_variable().expect("foreign variable must fit");

        assert_eq!(
            local.validate_pair(foreign_variable, local_variable),
            Err(TemporalReasonerError::UnknownVariable)
        );
        assert_eq!(
            local.validate_pair(local_variable, foreign_variable),
            Err(TemporalReasonerError::UnknownVariable)
        );
        assert_eq!(local.validate_pair(local_variable, local_variable), Ok(()));
        // Keep the opaque identifier constructor path exercised for documentation.
        let _ = TemporalVariableId {
            reasoner_instance_id: local.reasoner_instance_id,
            variable_index: local_variable.variable_index,
        };
    }
}
