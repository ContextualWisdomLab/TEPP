//! Observed versus inferred relation evidence status.

use serde::{Deserialize, Serialize};

/// Whether a relation edge is directly observed or model-inferred.
///
/// Observed edges and inferred edges remain distinct so estimators and audits
/// never treat LLM or heuristic proposals as raw documentary evidence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationEvidenceStatus {
    /// Directly observed in source documents or authoritative systems.
    Observed,
    /// Derived by a model, reasoner, or heuristic and not yet promoted.
    Inferred,
}

impl RelationEvidenceStatus {
    /// Return the stable wire status name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::Inferred => "inferred",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RelationEvidenceStatus;

    #[test]
    fn evidence_status_wire_names_are_stable() {
        assert_eq!(RelationEvidenceStatus::Observed.wire_name(), "observed");
        assert_eq!(RelationEvidenceStatus::Inferred.wire_name(), "inferred");
    }
}
