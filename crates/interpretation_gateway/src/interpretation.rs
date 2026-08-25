//! Hypothetical interpretations that must cite evidence spans.

use crate::{InterpretationError, InterpretationId};
use uuid::Uuid;

/// An LLM interpretation remains hypothetical until independently promoted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpretationStatus {
    /// The claim is an untrusted proposal bound to cited evidence.
    Hypothetical,
}

impl InterpretationStatus {
    /// Return whether this status is hypothetical.
    #[must_use]
    pub const fn is_hypothetical(self) -> bool {
        matches!(self, Self::Hypothetical)
    }
}

/// One evidence-bounded interpretation proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceBoundInterpretation {
    interpretation_id: InterpretationId,
    evidence_span_ids: Vec<Uuid>,
    status: InterpretationStatus,
}

impl EvidenceBoundInterpretation {
    /// Propose an interpretation that cites at least one evidence span.
    ///
    /// # Errors
    ///
    /// Returns [`InterpretationError::MissingEvidenceSpan`] when no span is
    /// supplied.
    pub fn propose(
        interpretation_id: InterpretationId,
        evidence_span_ids: &[Uuid],
    ) -> Result<Self, InterpretationError> {
        if evidence_span_ids.is_empty() {
            return Err(InterpretationError::MissingEvidenceSpan);
        }
        Ok(Self {
            interpretation_id,
            evidence_span_ids: evidence_span_ids.to_vec(),
            status: InterpretationStatus::Hypothetical,
        })
    }

    /// Return the interpretation identity.
    #[must_use]
    pub const fn interpretation_id(&self) -> InterpretationId {
        self.interpretation_id
    }

    /// Borrow the cited evidence span identities.
    #[must_use]
    pub fn evidence_span_ids(&self) -> &[Uuid] {
        &self.evidence_span_ids
    }

    /// Return whether the interpretation is still hypothetical.
    #[must_use]
    pub const fn is_hypothetical(&self) -> bool {
        self.status.is_hypothetical()
    }
}

/// Explicit refusal to treat an interpretation as an estimator result.
///
/// # Errors
///
/// Always returns [`InterpretationError::InterpretationIsNotEstimatorResult`].
pub fn refuse_interpretation_as_estimator_result(
    _interpretation_id: InterpretationId,
) -> Result<(), InterpretationError> {
    Err(InterpretationError::InterpretationIsNotEstimatorResult)
}

/// Explicit refusal to treat an interpretation as an observed fact.
///
/// # Errors
///
/// Always returns [`InterpretationError::InterpretationIsNotObservedFact`].
pub fn refuse_interpretation_as_observed_fact(
    _interpretation_id: InterpretationId,
) -> Result<(), InterpretationError> {
    Err(InterpretationError::InterpretationIsNotObservedFact)
}

#[cfg(test)]
mod tests {
    use super::{
        EvidenceBoundInterpretation, InterpretationStatus,
        refuse_interpretation_as_estimator_result, refuse_interpretation_as_observed_fact,
    };
    use crate::{InterpretationError, InterpretationId};
    use uuid::Uuid;

    #[test]
    fn accessors_and_refusals_cover_local_branches() {
        let id = InterpretationId::from_uuid(Uuid::from_u128(2));
        let span = Uuid::from_u128(3);
        let interpretation = EvidenceBoundInterpretation::propose(id, &[span]).expect("cited");
        assert_eq!(interpretation.interpretation_id(), id);
        assert_eq!(interpretation.evidence_span_ids(), &[span]);
        assert!(interpretation.is_hypothetical());
        assert!(InterpretationStatus::Hypothetical.is_hypothetical());
        assert_eq!(
            refuse_interpretation_as_estimator_result(id),
            Err(InterpretationError::InterpretationIsNotEstimatorResult)
        );
        assert_eq!(
            refuse_interpretation_as_observed_fact(id),
            Err(InterpretationError::InterpretationIsNotObservedFact)
        );
    }
}
