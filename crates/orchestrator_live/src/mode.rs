//! ADR 0010 orchestration-mode vocabulary.

use serde::{Deserialize, Serialize};

/// Versioned orchestration mode selected for one interpretation run.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationMode {
    /// Single-model direct routing with a recorded budget.
    Direct,
    /// Direct routing plus an independent verifier role.
    Verify,
    /// Fixed multi-role committee with recorded disagreement.
    Committee,
    /// Adaptive conductor-style routing under an explicit budget.
    Conductor,
    /// Explicit abstention without a scientific claim.
    Abstain,
}

impl OrchestrationMode {
    /// Return the stable `snake_case` wire label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Verify => "verify",
            Self::Committee => "committee",
            Self::Conductor => "conductor",
            Self::Abstain => "abstain",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OrchestrationMode;

    #[test]
    fn mode_labels_cover_the_closed_vocabulary() {
        assert_eq!(OrchestrationMode::Direct.as_str(), "direct");
        assert_eq!(OrchestrationMode::Verify.as_str(), "verify");
        assert_eq!(OrchestrationMode::Committee.as_str(), "committee");
        assert_eq!(OrchestrationMode::Conductor.as_str(), "conductor");
        assert_eq!(OrchestrationMode::Abstain.as_str(), "abstain");
    }
}
