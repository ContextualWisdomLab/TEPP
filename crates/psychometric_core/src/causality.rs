//! Refusal of causal language from non-identifying heuristics.

use crate::error::PsychometricError;

/// A heuristic that is not, by itself, causal identification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CausalHeuristic {
    /// Event-time or document-time precedence.
    TemporalPrecedence,
    /// A citation, revision, or other document link.
    DocumentLinkage,
    /// TDT-style event tracking or coreference.
    EventTracking,
    /// A model prediction or schema completion.
    ModelPrediction,
}

impl CausalHeuristic {
    /// Stable wire name for the heuristic.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TemporalPrecedence => "temporal_precedence",
            Self::DocumentLinkage => "document_linkage",
            Self::EventTracking => "event_tracking",
            Self::ModelPrediction => "model_prediction",
        }
    }
}

/// Refuse a causal-effect claim that rests only on a non-identifying heuristic.
///
/// ADR 0005: temporal precedence, document linkage, event tracking, or model
/// prediction alone do not justify causal language.
///
/// # Errors
///
/// Always returns [`PsychometricError::CausalUnderidentified`].
pub fn claim_causal_effect(_heuristic: CausalHeuristic) -> Result<(), PsychometricError> {
    Err(PsychometricError::CausalUnderidentified)
}

#[cfg(test)]
mod tests {
    use super::{CausalHeuristic, claim_causal_effect};
    use crate::error::PsychometricError;

    #[test]
    fn every_heuristic_is_underidentified() {
        assert_eq!(
            claim_causal_effect(CausalHeuristic::DocumentLinkage),
            Err(PsychometricError::CausalUnderidentified)
        );
        assert_eq!(
            CausalHeuristic::TemporalPrecedence.as_str(),
            "temporal_precedence"
        );
        assert_eq!(CausalHeuristic::EventTracking.as_str(), "event_tracking");
        assert_eq!(
            CausalHeuristic::ModelPrediction.as_str(),
            "model_prediction"
        );
    }
}
