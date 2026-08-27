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
pub fn refuse_interval_consistency_as_unrestricted_satisfiability(
    _report: &IntervalConsistencyReport,
) -> Result<(), EventError> {
    Err(EventError::IntervalConsistencyIsNotUnrestrictedSatisfiability)
}

/// Explicit refusal to promote an interval contradiction into an event instance.
///
/// # Errors
///
/// Always returns [`EventError::IntervalContradictionIsNotEventInstance`].
pub fn refuse_interval_contradiction_as_instance(
    _error: EventError,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::IntervalContradictionIsNotEventInstance)
}

fn from_closure_report(report: ClosureReport) -> IntervalConsistencyReport {
    IntervalConsistencyReport {
        revisions: report.revisions(),
        propagation_steps: report.propagation_steps(),
    }
}

fn map_reasoner_error(error: TemporalReasonerError) -> EventError {
    match error {
        TemporalReasonerError::InvalidLimits => EventError::IntervalConsistencyInvalidLimits,
        TemporalReasonerError::UnknownVariable => EventError::IntervalConsistencyUnknownVariable,
        TemporalReasonerError::EmptyRelationSet => EventError::IntervalConsistencyEmptyRelationSet,
        TemporalReasonerError::LimitExceeded(_) => EventError::IntervalConsistencyLimitExceeded,
        TemporalReasonerError::Contradiction(_) => EventError::IntervalConsistencyContradiction,
        _ => EventError::InvalidWirePayload,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IntervalConsistencyNetwork, refuse_interval_consistency_as_unrestricted_satisfiability,
        refuse_interval_contradiction_as_instance,
    };
    use crate::EventError;
    use temporal_core::{
        AllenRelation, EventTime, RelationSet, TemporalBoundary, TemporalInterval,
        TemporalPrecision,
    };

    fn closed(start: &str, end: &str) -> TemporalInterval<EventTime> {
        TemporalInterval::bounded(
            TemporalBoundary::Included(EventTime::parse_rfc3339(start).expect("start")),
            TemporalBoundary::Included(EventTime::parse_rfc3339(end).expect("end")),
            TemporalPrecision::Second,
        )
        .expect("proper closed interval")
    }

    #[test]
    fn quantitative_before_chain_narrows_and_refuses_sat_claim() {
        let mut network = IntervalConsistencyNetwork::with_limits(8, 16, 256).expect("limits");
        let a = network.add_variable().expect("a");
        let b = network.add_variable().expect("b");
        let c = network.add_variable().expect("c");
        let early = closed("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z");
        let mid = closed("2026-01-03T00:00:00Z", "2026-01-04T00:00:00Z");
        let late = closed("2026-01-05T00:00:00Z", "2026-01-06T00:00:00Z");
        let (ab, _) = network
            .assert_quantitative_allen_relation(a, b, &early, &mid)
            .expect("a before b");
        let (bc, _) = network
            .assert_quantitative_allen_relation(b, c, &mid, &late)
            .expect("b before c");
        assert_eq!(ab, AllenRelation::Before);
        assert_eq!(bc, AllenRelation::Before);
        let report = network.close().expect("path consistent");
        assert!(report.changed());
        assert!(report.revisions() > 0);
        assert!(report.propagation_steps() > 0);
        let ac = network.relation_set(a, c).expect("a to c");
        assert!(ac.contains(AllenRelation::Before));
        assert!(!ac.contains(AllenRelation::After));
        assert_eq!(
            refuse_interval_consistency_as_unrestricted_satisfiability(&report),
            Err(EventError::IntervalConsistencyIsNotUnrestrictedSatisfiability)
        );
    }

    #[test]
    fn qualitative_before_and_after_contradict_and_refuse_instance() {
        let mut network = IntervalConsistencyNetwork::with_limits(4, 8, 64).expect("limits");
        let left = network.add_variable().expect("left");
        let right = network.add_variable().expect("right");
        network
            .assert_qualitative_relations(
                left,
                right,
                RelationSet::singleton(AllenRelation::Before),
            )
            .expect("before");
        let conflict = network.assert_qualitative_relations(
            left,
            right,
            RelationSet::singleton(AllenRelation::After),
        );
        assert_eq!(conflict, Err(EventError::IntervalConsistencyContradiction));
        assert_eq!(
            refuse_interval_contradiction_as_instance(EventError::IntervalConsistencyContradiction),
            Err(EventError::IntervalContradictionIsNotEventInstance)
        );
    }

    #[test]
    fn zero_limits_and_open_intervals_fail_closed() {
        assert_eq!(
            IntervalConsistencyNetwork::with_limits(0, 1, 1).map(|_| ()),
            Err(EventError::IntervalConsistencyInvalidLimits)
        );
        assert_eq!(
            IntervalConsistencyNetwork::with_limits(1, 0, 1).map(|_| ()),
            Err(EventError::IntervalConsistencyInvalidLimits)
        );
        assert_eq!(
            IntervalConsistencyNetwork::with_limits(1, 1, 0).map(|_| ()),
            Err(EventError::IntervalConsistencyInvalidLimits)
        );
        let mut network = IntervalConsistencyNetwork::with_limits(2, 2, 8).expect("limits");
        let left = network.add_variable().expect("left");
        let right = network.add_variable().expect("right");
        let open = TemporalInterval::bounded(
            TemporalBoundary::Excluded(
                EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            ),
            TemporalBoundary::Included(
                EventTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("e"),
            ),
            TemporalPrecision::Second,
        )
        .expect("open lower");
        let closed = closed("2026-01-03T00:00:00Z", "2026-01-04T00:00:00Z");
        assert_eq!(
            network
                .assert_quantitative_allen_relation(left, right, &open, &closed)
                .map(|_| ()),
            Err(EventError::IntervalConsistencyRequiresProperBoundedInterval)
        );
        assert_eq!(
            network
                .assert_qualitative_relations(left, right, RelationSet::empty())
                .map(|_| ()),
            Err(EventError::IntervalConsistencyEmptyRelationSet)
        );
    }

    #[test]
    fn path_composition_detects_indirect_contradiction() {
        // Allen (1983) / CHRONOS path consistency: A before B, B before C, and
        // A after C is impossible after composition.
        let mut network = IntervalConsistencyNetwork::with_limits(8, 16, 512).expect("limits");
        let a = network.add_variable().expect("a");
        let b = network.add_variable().expect("b");
        let c = network.add_variable().expect("c");
        network
            .assert_qualitative_relations(a, b, RelationSet::singleton(AllenRelation::Before))
            .expect("a before b");
        network
            .assert_qualitative_relations(b, c, RelationSet::singleton(AllenRelation::Before))
            .expect("b before c");
        network
            .assert_qualitative_relations(a, c, RelationSet::singleton(AllenRelation::After))
            .expect("direct after accepted until closure");
        assert_eq!(
            network.close().map(|_| ()),
            Err(EventError::IntervalConsistencyContradiction)
        );
    }

    #[test]
    fn foreign_variable_capacity_and_quiet_close_fail_closed() {
        let mut local = IntervalConsistencyNetwork::with_limits(1, 1, 8).expect("limits");
        let mut foreign = IntervalConsistencyNetwork::with_limits(2, 2, 8).expect("foreign");
        let local_var = local.add_variable().expect("local");
        assert_eq!(
            local.add_variable().map(|_| ()),
            Err(EventError::IntervalConsistencyLimitExceeded)
        );
        let foreign_var = foreign.add_variable().expect("foreign");
        assert_eq!(
            local.relation_set(foreign_var, local_var).map(|_| ()),
            Err(EventError::IntervalConsistencyUnknownVariable)
        );
        assert_eq!(
            local
                .assert_qualitative_relations(
                    local_var,
                    foreign_var,
                    RelationSet::singleton(AllenRelation::Before),
                )
                .map(|_| ()),
            Err(EventError::IntervalConsistencyUnknownVariable)
        );
        local
            .assert_qualitative_relations(
                local_var,
                local_var,
                RelationSet::singleton(AllenRelation::Equals),
            )
            .expect("self equals");
        assert_eq!(
            local
                .assert_qualitative_relations(
                    local_var,
                    local_var,
                    RelationSet::singleton(AllenRelation::Equals),
                )
                .map(|_| ()),
            Err(EventError::IntervalConsistencyLimitExceeded)
        );

        let mut pair = IntervalConsistencyNetwork::with_limits(2, 2, 8).expect("pair");
        let left = pair.add_variable().expect("left");
        let right = pair.add_variable().expect("right");
        pair.assert_qualitative_relations(
            left,
            right,
            RelationSet::singleton(AllenRelation::Before),
        )
        .expect("before");
        let quiet = pair.close().expect("two-node close has no middle");
        assert!(!quiet.changed());
        assert_eq!(quiet.revisions(), 0);
        assert_eq!(quiet.propagation_steps(), 0);

        let mut tight = IntervalConsistencyNetwork::with_limits(8, 16, 1).expect("tight");
        let a = tight.add_variable().expect("a");
        let b = tight.add_variable().expect("b");
        let c = tight.add_variable().expect("c");
        tight
            .assert_qualitative_relations(a, b, RelationSet::singleton(AllenRelation::Before))
            .expect("a before b");
        tight
            .assert_qualitative_relations(b, c, RelationSet::singleton(AllenRelation::Before))
            .expect("b before c");
        assert_eq!(
            tight.close().map(|_| ()),
            Err(EventError::IntervalConsistencyLimitExceeded)
        );
    }
}
