//! Scientific-role gates for untrusted payloads.

use crate::PayloadSemanticsError;

/// Closed vocabulary of untrusted inbound payload kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadKind {
    /// External document bytes.
    Document,
    /// External metadata that is not the document body.
    ExternalMetadata,
    /// Serialized domain or wire record.
    SerializedRecord,
    /// LLM or agent output.
    LlmOutput,
}

impl PayloadKind {
    /// Return the stable wire payload-kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::ExternalMetadata => "external_metadata",
            Self::SerializedRecord => "serialized_record",
            Self::LlmOutput => "llm_output",
        }
    }

    /// Parse a stable wire payload-kind name.
    ///
    /// # Errors
    ///
    /// Returns [`PayloadSemanticsError::InvalidSemanticsPayload`] for
    /// unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, PayloadSemanticsError> {
        match name {
            "document" => Ok(Self::Document),
            "external_metadata" => Ok(Self::ExternalMetadata),
            "serialized_record" => Ok(Self::SerializedRecord),
            "llm_output" => Ok(Self::LlmOutput),
            _ => Err(PayloadSemanticsError::InvalidSemanticsPayload),
        }
    }
}

/// Closed vocabulary of claimed scientific roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScientificRole {
    /// Source or metadata evidence context.
    EvidenceContext,
    /// CPU `f64` estimator result.
    EstimatorResult,
    /// Posterior summary produced by an estimator.
    PosteriorSummary,
    /// Interpretation narrative that is not a measurement.
    InterpretationNarrative,
}

impl ScientificRole {
    /// Return the stable wire scientific-role name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::EvidenceContext => "evidence_context",
            Self::EstimatorResult => "estimator_result",
            Self::PosteriorSummary => "posterior_summary",
            Self::InterpretationNarrative => "interpretation_narrative",
        }
    }

    /// Parse a stable wire scientific-role name.
    ///
    /// # Errors
    ///
    /// Returns [`PayloadSemanticsError::InvalidSemanticsPayload`] for
    /// unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, PayloadSemanticsError> {
        match name {
            "evidence_context" => Ok(Self::EvidenceContext),
            "estimator_result" => Ok(Self::EstimatorResult),
            "posterior_summary" => Ok(Self::PosteriorSummary),
            "interpretation_narrative" => Ok(Self::InterpretationNarrative),
            _ => Err(PayloadSemanticsError::InvalidSemanticsPayload),
        }
    }
}

/// Refuse an untrusted payload that claims an unauthorized scientific role.
///
/// Identity, provenance, size, and depth are owned by `payload_bound`.
/// Grant presence is owned by `intake_authorization`. Checkpoints versus
/// the CPU `f64` estimator are owned by `checkpoint_authority`.
///
/// # Errors
///
/// Returns a role-mismatch error when the payload kind cannot hold `role`.
pub fn refuse_untrusted_scientific_claim(
    kind: PayloadKind,
    role: ScientificRole,
) -> Result<(), PayloadSemanticsError> {
    match role {
        ScientificRole::EvidenceContext => match kind {
            PayloadKind::Document
            | PayloadKind::ExternalMetadata
            | PayloadKind::SerializedRecord => Ok(()),
            PayloadKind::LlmOutput => Err(PayloadSemanticsError::LlmOutputIsNotEvidence),
        },
        ScientificRole::EstimatorResult | ScientificRole::PosteriorSummary => {
            Err(PayloadSemanticsError::UntrustedPayloadIsNotEstimator)
        }
        ScientificRole::InterpretationNarrative => match kind {
            PayloadKind::LlmOutput => Ok(()),
            PayloadKind::Document
            | PayloadKind::ExternalMetadata
            | PayloadKind::SerializedRecord => {
                Err(PayloadSemanticsError::EvidenceIsNotInterpretation)
            }
        },
    }
}

/// Refuse to treat identity, size, or authorization bounds as semantics.
///
/// # Errors
///
/// Always returns [`PayloadSemanticsError::BoundsAreNotSemantics`].
pub fn refuse_bounds_as_semantics() -> Result<(), PayloadSemanticsError> {
    Err(PayloadSemanticsError::BoundsAreNotSemantics)
}

/// Fraction of recovered scientific roles that match known truth.
///
/// # Errors
///
/// Returns [`PayloadSemanticsError::InvalidSemanticsPayload`] when either
/// slice is empty or the lengths differ.
pub fn semantics_recovery_rate(
    truth: &[ScientificRole],
    decided: &[ScientificRole],
) -> Result<f64, PayloadSemanticsError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(PayloadSemanticsError::InvalidSemanticsPayload);
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
        PayloadKind, ScientificRole, refuse_bounds_as_semantics, refuse_untrusted_scientific_claim,
        semantics_recovery_rate,
    };
    use crate::PayloadSemanticsError;

    #[test]
    fn local_branches_cover_kinds_roles_and_payloads() {
        for kind in [
            PayloadKind::Document,
            PayloadKind::ExternalMetadata,
            PayloadKind::SerializedRecord,
        ] {
            refuse_untrusted_scientific_claim(kind, ScientificRole::EvidenceContext)
                .expect("evidence");
            assert_eq!(
                refuse_untrusted_scientific_claim(kind, ScientificRole::EstimatorResult),
                Err(PayloadSemanticsError::UntrustedPayloadIsNotEstimator)
            );
            assert_eq!(
                refuse_untrusted_scientific_claim(kind, ScientificRole::PosteriorSummary),
                Err(PayloadSemanticsError::UntrustedPayloadIsNotEstimator)
            );
            assert_eq!(
                refuse_untrusted_scientific_claim(kind, ScientificRole::InterpretationNarrative),
                Err(PayloadSemanticsError::EvidenceIsNotInterpretation)
            );
            assert_eq!(
                PayloadKind::from_wire_name(kind.wire_name()).expect("kind"),
                kind
            );
        }
        refuse_untrusted_scientific_claim(
            PayloadKind::LlmOutput,
            ScientificRole::InterpretationNarrative,
        )
        .expect("interpretation");
        assert_eq!(
            refuse_untrusted_scientific_claim(
                PayloadKind::LlmOutput,
                ScientificRole::EvidenceContext
            ),
            Err(PayloadSemanticsError::LlmOutputIsNotEvidence)
        );
        assert_eq!(
            refuse_untrusted_scientific_claim(
                PayloadKind::LlmOutput,
                ScientificRole::EstimatorResult
            ),
            Err(PayloadSemanticsError::UntrustedPayloadIsNotEstimator)
        );
        assert_eq!(
            refuse_untrusted_scientific_claim(
                PayloadKind::LlmOutput,
                ScientificRole::PosteriorSummary
            ),
            Err(PayloadSemanticsError::UntrustedPayloadIsNotEstimator)
        );
        assert_eq!(
            refuse_bounds_as_semantics(),
            Err(PayloadSemanticsError::BoundsAreNotSemantics)
        );
        for role in [
            ScientificRole::EvidenceContext,
            ScientificRole::EstimatorResult,
            ScientificRole::PosteriorSummary,
            ScientificRole::InterpretationNarrative,
        ] {
            assert_eq!(
                ScientificRole::from_wire_name(role.wire_name()).expect("role"),
                role
            );
        }
        assert_eq!(
            PayloadKind::from_wire_name("trusted"),
            Err(PayloadSemanticsError::InvalidSemanticsPayload)
        );
        assert_eq!(
            ScientificRole::from_wire_name("causal_effect"),
            Err(PayloadSemanticsError::InvalidSemanticsPayload)
        );
        assert_eq!(
            PayloadKind::from_wire_name(PayloadKind::LlmOutput.wire_name()).expect("llm"),
            PayloadKind::LlmOutput
        );
        let matched = semantics_recovery_rate(
            &[ScientificRole::EvidenceContext],
            &[ScientificRole::EvidenceContext],
        )
        .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            semantics_recovery_rate(&[], &[]),
            Err(PayloadSemanticsError::InvalidSemanticsPayload)
        );
        assert_eq!(
            semantics_recovery_rate(&[ScientificRole::EvidenceContext], &[]),
            Err(PayloadSemanticsError::InvalidSemanticsPayload)
        );
    }
}
