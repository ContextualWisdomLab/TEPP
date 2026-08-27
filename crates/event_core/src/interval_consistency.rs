//! Bounded CHRONOS-style interval consistency for event intelligence (#170).
//!
//! Allen (1983) defines the thirteen elementary relations and composition.
//! Anagnostopoulos, Batsakis, and Petrakis (2013) apply path consistency over
//! qualitative temporal networks to infer implied relations and detect
//! inconsistencies while remaining bounded. Hobbs and Pan (2017) OWL-Time
//! names the same qualitative interval vocabulary. This module owns the
//! event-intelligence consistency gate over [`temporal_core::TemporalReasoner`]:
//! it derives bounded implications, rejects contradictions, and never claims
//! unrestricted global satisfiability. Pairwise predicted-versus-observed
//! promotion remains [`prediction_contradiction`]; this module does not replace
//! that gate.

use crate::{EventError, EventInstanceId};
use temporal_core::{
    AllenRelation, ClosureReport, ConstraintId, EventTime, RelationSet, TemporalInterval,
    TemporalReasoner, TemporalReasonerError, TemporalReasonerLimits, TemporalVariableId,
    classify_interval_relation,
};

/// Summary of one successful bounded interval-consistency closure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntervalConsistencyReport {
    revisions: usize,
    propagation_steps: usize,
}

impl IntervalConsistencyReport {
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

/// Bounded qualitative interval-constraint network for event intelligence.
///
/// Variables are opaque event-interval identities. Quantitative proper
/// intervals classify to a singleton Allen relation before assertion.
/// Qualitative assertions admit a nonempty [`RelationSet`] when extents are
/// unknown. Closure is atomic and resource-bounded.
pub struct IntervalConsistencyNetwork {
    reasoner: TemporalReasoner,
}

impl IntervalConsistencyNetwork {
    /// Open an empty network with explicit nonzero resource limits.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::IntervalConsistencyInvalidLimits`] when any limit
    /// is zero.
    pub fn with_limits(
        maximum_variables: usize,
        maximum_constraints: usize,
        maximum_propagation_steps: usize,
    ) -> Result<Self, EventError> {
        let limits = TemporalReasonerLimits::new(
            maximum_variables,
            maximum_constraints,
            maximum_propagation_steps,
        )
        .map_err(map_reasoner_error)?;
        Ok(Self {
            reasoner: TemporalReasoner::with_limits(limits),
        })
    }

    /// Add one event-interval variable.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::IntervalConsistencyLimitExceeded`] when the
    /// configured variable capacity is exhausted.
    pub fn add_variable(&mut self) -> Result<TemporalVariableId, EventError> {
        self.reasoner.add_variable().map_err(map_reasoner_error)
    }

    /// Assert a nonempty qualitative relation set for an ordered pair.
    ///
    /// Use this when temporal extents are unknown and only Allen vocabulary is
    /// available (Anagnostopoulos et al., 2013; Hobbs & Pan, 2017).
    ///
    /// # Errors
    ///
    /// Propagates unknown-variable, empty-set, capacity, or contradiction
    /// errors from the bounded reasoner.
    pub fn assert_qualitative_relations(
        &mut self,
        left: TemporalVariableId,
        right: TemporalVariableId,
        relations: RelationSet,
    ) -> Result<ConstraintId, EventError> {
        self.reasoner
            .assert_relation(left, right, relations)
            .map_err(map_reasoner_error)
    }

    /// Classify two proper closed event-time intervals and assert that singleton.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::IntervalConsistencyRequiresProperBoundedInterval`]
    /// when either interval is not a proper two-sided closed interval, and
    /// propagates reasoner errors for capacity or contradiction.
    pub fn assert_quantitative_allen_relation(
        &mut self,
        left: TemporalVariableId,
        right: TemporalVariableId,
        left_interval: &TemporalInterval<EventTime>,
        right_interval: &TemporalInterval<EventTime>,
    ) -> Result<(AllenRelation, ConstraintId), EventError> {
        let relation = classify_interval_relation(left_interval, right_interval).map_err(
            |error| match error {
                temporal_core::TemporalError::RelationRequiresProperBoundedInterval => {
                    EventError::IntervalConsistencyRequiresProperBoundedInterval
                }
                _ => EventError::InvalidWirePayload,
            },
        )?;
        let constraint = self
            .reasoner
            .assert_relation(left, right, RelationSet::singleton(relation))
            .map_err(map_reasoner_error)?;
        Ok((relation, constraint))
    }

    /// Apply bounded path-consistency closure atomically.
    ///
    /// Successful closure yields implied narrowing under the configured
    /// propagation budget. Failure restores the pre-closure network.
    ///
    /// # Errors
    ///
    /// Returns contradiction or resource-limit errors. Success is not
    /// unrestricted global satisfiability.
    pub fn close(&mut self) -> Result<IntervalConsistencyReport, EventError> {
        let report = self.reasoner.close().map_err(map_reasoner_error)?;
        Ok(from_closure_report(report))
    }

    /// Return the current relation set for an ordered pair after assertion/closure.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::IntervalConsistencyUnknownVariable`] when either
    /// identifier does not belong to this network.
    pub fn relation_set(
        &self,
        left: TemporalVariableId,
        right: TemporalVariableId,
    ) -> Result<RelationSet, EventError> {
        self.reasoner
            .relation(left, right)
            .map(|derived| derived.relations())
            .map_err(map_reasoner_error)
    }
}

/// Explicit refusal to treat bounded path consistency as unrestricted SAT.
///
/// Anagnostopoulos et al. (2013) retain soundness and completeness over the
/// supported relation set under path consistency; TEPP still refuses to claim
/// unrestricted global satisfiability for arbitrary relation vocabularies or
/// unbounded networks (#170 acceptance).
///
/// # Errors
///
/// Always returns
/// [`EventError::IntervalConsistencyIsNotUnrestrictedSatisfiability`].

// ... continued in next commit
