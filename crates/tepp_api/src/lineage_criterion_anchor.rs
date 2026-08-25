//! Versioned TEPP criterion-validity artifact for Event Lineage channel weights.
//!
//! TEPP is the authority that evaluates an independently supplied lineage
//! criterion. This DTO only transports TEPP's decision and exact provenance;
//! it does not let a consumer calculate or reinterpret validity locally.

use serde::{Deserialize, Serialize};
use temporal_core::KnowledgeCutoff;

use crate::ApiError;
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json_with_limit,
};

/// Semantic contract version for lineage criterion anchors.
pub const LINEAGE_CRITERION_ANCHOR_CONTRACT_VERSION: u16 = 1;

/// Analysis-run model contract requested by a `LineageWeave` consumer.
pub const LINEAGE_CRITERION_MODEL_CONTRACT: &str = "tepp-lineage-criterion-v1";

/// Analysis-run output profile that produces this artifact.
pub const LINEAGE_CRITERION_OUTPUT_PROFILE: &str = "lineage_pair_criterion_anchor";

/// Result-schema identity carried by the terminal analysis result.
pub const LINEAGE_CRITERION_RESULT_SCHEMA: &str = "tepp.lineage_criterion_anchor.v1";

/// Default maximum serialized anchor artifact size.
pub const DEFAULT_LINEAGE_CRITERION_ANCHOR_BYTE_LIMIT: usize = 16 * 1024;

/// TEPP-owned criterion-validity outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CriterionValidityStatus {
    /// The proposed channel-weight run passed TEPP's registered criterion validation.
    Accepted,
    /// The proposed channel-weight run failed TEPP's registered criterion validation.
    Rejected,
}

/// Exact, identity-bound TEPP criterion-validity result for one weight run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineageCriterionAnchor {
    /// Semantic contract version, always one for this shape.
    pub contract_version: u16,
    /// Artifact kind, always `lineage_pair_criterion`.
    pub anchor_kind_code: String,
    /// Opaque consumer-owned fast-mlsirm estimation-run identity.
    pub estimation_run_id: String,
    /// Immutable source snapshot SHA-256 shared with the proposed weights.
    pub source_snapshot_sha256: String,
    /// Exact RFC 3339 knowledge cutoff shared with the proposed weights.
    pub knowledge_cutoff: String,
    /// TEPP-owned criterion-validity outcome.
    pub criterion_validity_status: CriterionValidityStatus,
    /// Number of pair outcomes that entered the independent validation.
    pub validated_pair_count: u64,
}

impl LineageCriterionAnchor {
    /// Parse and validate an anchor with the default payload limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, version, identity, digest, time, or count error.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_LINEAGE_CRITERION_ANCHOR_BYTE_LIMIT)
    }

    /// Parse and validate an anchor with a caller-supplied payload limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, version, identity, digest, time, or count error.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        let value: Self = from_json(payload)?;
        value.validate()?;
        Ok(value)
    }

    /// Serialize this validated anchor artifact.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed validation or serialization error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json_with_limit(self, DEFAULT_LINEAGE_CRITERION_ANCHOR_BYTE_LIMIT)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            LINEAGE_CRITERION_ANCHOR_CONTRACT_VERSION,
        )?;
        if self.anchor_kind_code != "lineage_pair_criterion"
            || self.validated_pair_count == 0
            || !is_canonical_sha256(&self.source_snapshot_sha256)
        {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.estimation_run_id)?;
        uuid::Uuid::parse_str(&self.estimation_run_id).map_err(|_| ApiError::InvalidWirePayload)?;
        KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff)
            .map_err(|_| ApiError::InvalidWirePayload)?;
        Ok(())
    }
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor() -> LineageCriterionAnchor {
        LineageCriterionAnchor {
            contract_version: LINEAGE_CRITERION_ANCHOR_CONTRACT_VERSION,
            anchor_kind_code: "lineage_pair_criterion".into(),
            estimation_run_id: "018f47e7-7b5b-7cc0-98c6-15fdf9e3d9b1".into(),
            source_snapshot_sha256: "a".repeat(64),
            knowledge_cutoff: "2026-08-25T00:00:00Z".into(),
            criterion_validity_status: CriterionValidityStatus::Accepted,
            validated_pair_count: 600,
        }
    }

    #[test]
    fn accepted_and_rejected_results_round_trip_without_local_reinterpretation() {
        for status in [
            CriterionValidityStatus::Accepted,
            CriterionValidityStatus::Rejected,
        ] {
            let mut value = anchor();
            value.criterion_validity_status = status;
            let json = value.to_json().expect("serialize");
            assert_eq!(
                LineageCriterionAnchor::from_json(&json).expect("parse"),
                value
            );
        }
    }

    #[test]
    fn malformed_or_unbound_artifacts_fail_closed() {
        for mutate in 0..5 {
            let mut value = anchor();
            match mutate {
                0 => value.contract_version = 2,
                1 => value.anchor_kind_code = "internal_structure".into(),
                2 => value.estimation_run_id = "not-a-uuid".into(),
                3 => value.source_snapshot_sha256 = "A".repeat(64),
                _ => value.validated_pair_count = 0,
            }
            assert!(value.to_json().is_err());
        }
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let json = anchor().to_json().expect("serialize");
        let payload = json.strip_suffix('}').expect("object");
        assert!(LineageCriterionAnchor::from_json(&format!("{payload},\"theta\":0.8}}")).is_err());
    }
}
