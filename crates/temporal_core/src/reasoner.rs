//! Bounded path-consistency closure for qualitative interval constraints.

use crate::RelationSet;
use std::collections::BTreeSet;
use std::fmt;

/// An opaque identifier for one interval variable in a reasoner instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TemporalVariableId(usize);

/// An opaque identifier for one observed relation assertion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConstraintId(usize);

/// The bounded resource whose configured maximum was exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasonerLimitKind {
    /// The maximum number of interval variables.
    Variables,
    /// The maximum number of observed constraints.
    Constraints,
    /// The maximum number of path-consistency propagation steps.
    PropagationSteps,
}

/// Explicit capacity bounds for one temporal reasoner instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemporalReasonerLimits {
    maximum_variables: usize,
    maximum_constraints: usize,
    maximum_propagation_steps: usize,
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
        if maximum_variables == 0
            || maximum_constraints == 0
            || maximum_propagation_steps == 0
        {
            Err(TemporalReasonerError::InvalidLimits)
        } else {
            Ok(Self {
                maximum_variables,
                maximum_constraints,
                maximum_propagation_steps,
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

    /// Return the conservative set of observed assertions supporting the contradiction.
    #[must_use]
    pub fn support(&self) -> &[ConstraintId] {
        &self.support
    }

    fn from_support(
        left: TemporalVariableId,
        right: TemporalVariableId,
        support: BTreeSet<ConstraintId>,
    ) -> Self {
        Self {
            left,
            right,
            support: support.into_iter().collect(),
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
    /// An observed constraint supplied no possible elementary relation.
    EmptyRelationSet,
    /// A configured reasoner resource maximum was exceeded.
    LimitExceeded(ReasonerLimitKind),
    /// Constraint propagation proved that no relation remains possible.
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

    /// Return the conservative observed-assertion support for this relation.
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
/// network to its pre-closure state.
pub struct TemporalReasoner {
    limits: TemporalReasonerLimits,
    cells: Vec<Vec<RelationCell>>,
    constraint_count: usize,
}

impl TemporalReasoner {
    /// Create an empty reasoner with validated explicit limits.
    #[must_use]
    pub const fn with_limits(limits: TemporalReasonerLimits) -> Self {
        Self {
            limits,
            cells: Vec::new(),
            constraint_count: 0,
        }
    }

    /// Add one interval variable.
    ///
    /// # Errors
    ///
    /// Returns [`TemporalReasonerError::LimitExceeded`] when the configured
    /// variable capacity is exhausted.
    pub fn add_variable(&mut self) -> Result<TemporalVariableId, TemporalReasonerError> {
        if self.cells.len() >= self.limits.maximum_variables {
            return Err(TemporalReasonerError::LimitExceeded(
                ReasonerLimitKind::Variables,
            ));
        }

        let identifier = TemporalVariableId(self.cells.len());
        for row in &mut self.cells {
            row.push(RelationCell::unconstrained());
        }
        let mut new_row = vec![RelationCell::unconstrained(); self.cells.len() + 1];
        new_row[identifier.0] = RelationCell::identity();
        self.cells.push(new_row);
        Ok(identifier)
    }

    /// Assert a nonempty relation set for an ordered variable pair.
    ///
    /// The reverse pair is narrowed by the inverse relation set. The returned
    /// identifier is retained as provenance for later derived relations.
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
        if self.constraint_count >= self.limits.maximum_constraints {
            return Err(TemporalReasonerError::LimitExceeded(
                ReasonerLimitKind::Constraints,
            ));
        }
        if relations.is_empty() {
            return Err(TemporalReasonerError::EmptyRelationSet);
        }

        let identifier = ConstraintId(self.constraint_count);
        let narrowed = self.cells[left.0][right.0]
            .relations
            .intersection(relations);
        if narrowed.is_empty() {
            let mut support = self.cells[left.0][right.0].support.clone();
            support.insert(identifier);
            return Err(TemporalReasonerError::Contradiction(
                TemporalContradiction::from_support(left, right, support),
            ));
        }

        let mut support = self.cells[left.0][right.0].support.clone();
        support.insert(identifier);
        self.cells[left.0][right.0] = RelationCell {
            relations: narrowed,
            observed: true,
            support: support.clone(),
        };
        self.cells[right.0][left.0] = RelationCell {
            relations: narrowed.inverse(),
            observed: true,
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
        let cell = &self.cells[left.0][right.0];
        Ok(DerivedRelation {
            relations: cell.relations,
            observed: cell.observed,
            support: cell.support.iter().copied().collect(),
        })
    }

    fn validate_pair(
        &self,
        left: TemporalVariableId,
        right: TemporalVariableId,
    ) -> Result<(), TemporalReasonerError> {
        if left.0 >= self.cells.len() || right.0 >= self.cells.len() {
            Err(TemporalReasonerError::UnknownVariable)
        } else {
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
                        if propagation_steps >= self.limits.maximum_propagation_steps {
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
                                    TemporalVariableId(left),
                                    TemporalVariableId(right),
                                    support,
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
                        self.cells[left][right] = RelationCell {
                            relations: narrowed,
                            observed,
                            support: support.clone(),
                        };
                        self.cells[right][left] = RelationCell {
                            relations: narrowed.inverse(),
                            observed,
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
