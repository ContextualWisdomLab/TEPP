//! Fail-closed activation authority for Rubin loading projection.
//!
//! Rubin combination arithmetic remains in `psychometric_core`. This module
//! only decides whether a digest-bound generator/analysis/evidence pairing is
//! authorized to project beyond the descriptive result produced for unbound
//! draw-generation provenance.

use psychometric_core::IndicatorKind;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, KnowledgeCutoff};

use crate::{AnalysisEngineError, format_digest, valid_identifier};

/// Versioned wire schema for a Rubin projection activation receipt.
pub const RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION: &str =
    "tepp.rubin_projection_activation_receipt.v1";
/// Maximum canonical JSON size accepted for one activation receipt.
pub const RUBIN_PROJECTION_ACTIVATION_RECEIPT_BYTE_LIMIT: usize = 16 * 1024;
/// Exact analysis-contract identity authorized by this projection policy.
pub const RUBIN_LOADING_ANALYSIS_CONTRACT_ID: &str = "rubin_loading_uncertainty";

/// Immutable authority receipt offered to the Rubin projection decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RubinProjectionActivationReceiptV1 {
    schema_version: String,
    generator_contract_id: String,
    generator_contract_version: String,
    analysis_contract_id: String,
    analysis_contract_version: String,
    validation_evidence_id: String,
    validation_evidence_sha256: String,
    validation_evidence_available_at: String,
    source_snapshot_id: String,
    knowledge_cutoff: String,
    design_envelope_id: String,
}

impl RubinProjectionActivationReceiptV1 {
    /// Construct one canonical activation receipt from typed temporal clocks.
    ///
    /// Construction validates identifiers and digest syntax but does not grant
    /// authority. In particular, late Validation Evidence remains a valid wire
    /// object that the projection decision rejects by cutoff.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed validation error when an identifier or digest is
    /// malformed.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        generator_contract_id: impl Into<String>,
        generator_contract_version: impl Into<String>,
        analysis_contract_id: impl Into<String>,
        analysis_contract_version: impl Into<String>,
        validation_evidence_id: impl Into<String>,
        validation_evidence_sha256: impl Into<String>,
        validation_evidence_available_at: AvailableTime,
        source_snapshot_id: impl Into<String>,
        knowledge_cutoff: KnowledgeCutoff,
        design_envelope_id: impl Into<String>,
    ) -> Result<Self, AnalysisEngineError> {
        let receipt = Self {
            schema_version: RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION.into(),
            generator_contract_id: generator_contract_id.into(),
            generator_contract_version: generator_contract_version.into(),
            analysis_contract_id: analysis_contract_id.into(),
            analysis_contract_version: analysis_contract_version.into(),
            validation_evidence_id: validation_evidence_id.into(),
            validation_evidence_sha256: validation_evidence_sha256.into(),
            validation_evidence_available_at: validation_evidence_available_at.to_rfc3339(),
            source_snapshot_id: source_snapshot_id.into(),
            knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
            design_envelope_id: design_envelope_id.into(),
        };
        receipt.validate()?;
        Ok(receipt)
    }

    /// Parse and fully validate bounded receipt JSON.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::LimitExceeded`] before parsing an
    /// oversized payload and [`AnalysisEngineError::InvalidEvidence`] for an
    /// invalid receipt contract.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        require_receipt_byte_limit(payload.len())?;
        let receipt: Self =
            serde_json::from_str(payload).map_err(|_| AnalysisEngineError::InvalidEvidence)?;
        receipt.validate()?;
        Ok(receipt)
    }

    /// Serialize canonical validated receipt JSON.
    ///
    /// # Errors
    ///
    /// Returns a validation, size, or serialization failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        require_receipt_byte_limit(payload.len())?;
        Ok(payload)
    }

    /// Return the lowercase SHA-256 digest of canonical receipt JSON.
    ///
    /// # Errors
    ///
    /// Returns a validation, size, or serialization failure.
    pub fn sha256(&self) -> Result<String, AnalysisEngineError> {
        self.to_json()
            .map(|json| format_digest(Sha256::digest(json.into_bytes())))
    }

    /// Return the immutable source snapshot bound into the receipt.
    #[must_use]
    pub fn source_snapshot_id(&self) -> &str {
        &self.source_snapshot_id
    }

    /// Return the canonical knowledge-cutoff wire value.
    #[must_use]
    pub fn knowledge_cutoff(&self) -> &str {
        &self.knowledge_cutoff
    }

    fn validate(&self) -> Result<(), AnalysisEngineError> {
        let identifiers = [
            self.generator_contract_id.as_str(),
            self.generator_contract_version.as_str(),
            self.analysis_contract_id.as_str(),
            self.analysis_contract_version.as_str(),
            self.validation_evidence_id.as_str(),
            self.source_snapshot_id.as_str(),
            self.design_envelope_id.as_str(),
        ];
        if self.schema_version != RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION
            || identifiers.iter().any(|value| !valid_identifier(value))
            || !valid_sha256(&self.validation_evidence_sha256)
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let available = AvailableTime::parse_rfc3339(&self.validation_evidence_available_at)
            .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
        let cutoff = KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff)
            .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
        if available.to_rfc3339() != self.validation_evidence_available_at
            || cutoff.to_rfc3339() != self.knowledge_cutoff
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(())
    }
}

/// Outcome of evaluating a Rubin projection activation receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RubinProjectionActivationDecision {
    /// No activation receipt was offered; the artifact remains descriptive.
    DescriptiveOnly,
    /// An offered receipt failed authority, provenance, or registry matching.
    Rejected,
    /// The exact immutable pairing is approved for the declared design envelope.
    Eligible,
}

#[derive(Clone, Copy, Debug)]
struct ApprovedRubinProjectionPairing {
    generator_contract_id: &'static str,
    generator_contract_version: &'static str,
    analysis_contract_id: &'static str,
    analysis_contract_version: &'static str,
    validation_evidence_id: &'static str,
    validation_evidence_sha256: &'static str,
    design_envelope_id: &'static str,
    indicator_kind: &'static str,
}

// Intentionally empty until claim-specific Validation Evidence crosses the
// independent ADR 0014 promotion gate. Draft scientific branches are not
// production authority.
const PRODUCTION_APPROVED_PAIRINGS: &[ApprovedRubinProjectionPairing] = &[];

/// Decide whether an optional activation receipt authorizes Rubin projection.
///
/// Production approval data are deliberately not caller-supplied. The registry
/// remains empty until an independently accepted Validation Evidence package is
/// promoted through the owner path, so a syntactically valid candidate receipt
/// cannot self-authorize.
#[must_use]
pub fn decide_rubin_projection_activation(
    receipt: Option<&RubinProjectionActivationReceiptV1>,
    expected_snapshot_id: &str,
    expected_knowledge_cutoff: KnowledgeCutoff,
    indicator_kind: IndicatorKind,
) -> RubinProjectionActivationDecision {
    decide_with_registry(
        receipt,
        expected_snapshot_id,
        expected_knowledge_cutoff,
        indicator_kind,
        PRODUCTION_APPROVED_PAIRINGS,
    )
}

fn decide_with_registry(
    receipt: Option<&RubinProjectionActivationReceiptV1>,
    expected_snapshot_id: &str,
    expected_knowledge_cutoff: KnowledgeCutoff,
    indicator_kind: IndicatorKind,
    approved_pairings: &[ApprovedRubinProjectionPairing],
) -> RubinProjectionActivationDecision {
    let Some(receipt) = receipt else {
        return RubinProjectionActivationDecision::DescriptiveOnly;
    };
    if receipt.validate().is_err() || !valid_identifier(expected_snapshot_id) {
        return RubinProjectionActivationDecision::Rejected;
    }
    if receipt.source_snapshot_id != expected_snapshot_id {
        return RubinProjectionActivationDecision::Rejected;
    }
    let Ok(receipt_cutoff) = KnowledgeCutoff::parse_rfc3339(&receipt.knowledge_cutoff) else {
        return RubinProjectionActivationDecision::Rejected;
    };
    if receipt_cutoff.instant() != expected_knowledge_cutoff.instant() {
        return RubinProjectionActivationDecision::Rejected;
    }
    let Ok(evidence_available_at) =
        AvailableTime::parse_rfc3339(&receipt.validation_evidence_available_at)
    else {
        return RubinProjectionActivationDecision::Rejected;
    };
    if evidence_available_at.instant() > expected_knowledge_cutoff.instant() {
        return RubinProjectionActivationDecision::Rejected;
    }

    let approved = approved_pairings.iter().any(|pairing| {
        receipt.generator_contract_id == pairing.generator_contract_id
            && receipt.generator_contract_version == pairing.generator_contract_version
            && receipt.analysis_contract_id == pairing.analysis_contract_id
            && receipt.analysis_contract_version == pairing.analysis_contract_version
            && receipt.validation_evidence_id == pairing.validation_evidence_id
            && receipt.validation_evidence_sha256 == pairing.validation_evidence_sha256
            && receipt.design_envelope_id == pairing.design_envelope_id
            && indicator_kind.as_str() == pairing.indicator_kind
    });
    if approved {
        RubinProjectionActivationDecision::Eligible
    } else {
        RubinProjectionActivationDecision::Rejected
    }
}

fn require_receipt_byte_limit(payload_len: usize) -> Result<(), AnalysisEngineError> {
    if payload_len > RUBIN_PROJECTION_ACTIVATION_RECEIPT_BYTE_LIMIT {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{
        ApprovedRubinProjectionPairing, RUBIN_LOADING_ANALYSIS_CONTRACT_ID,
        RUBIN_PROJECTION_ACTIVATION_RECEIPT_BYTE_LIMIT, RubinProjectionActivationDecision,
        RubinProjectionActivationReceiptV1, decide_with_registry,
    };
    use crate::{AnalysisEngineError, RUBIN_LOADING_MODEL_CONTRACT_VERSION};
    use psychometric_core::IndicatorKind;
    use temporal_core::{AvailableTime, KnowledgeCutoff};

    const SNAPSHOT_ID: &str = "snapshot-rubin-activation";
    const EVIDENCE_ID: &str = "validation-evidence-rubin-approved-v1";
    const EVIDENCE_DIGEST: &str =
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const DESIGN_ENVELOPE: &str = "rubin-gaussian-single-level-approved-v1";

    const APPROVED: ApprovedRubinProjectionPairing = ApprovedRubinProjectionPairing {
        generator_contract_id: "gaussian_complete_data_draws",
        generator_contract_version: "approved-v1",
        analysis_contract_id: RUBIN_LOADING_ANALYSIS_CONTRACT_ID,
        analysis_contract_version: RUBIN_LOADING_MODEL_CONTRACT_VERSION,
        validation_evidence_id: EVIDENCE_ID,
        validation_evidence_sha256: EVIDENCE_DIGEST,
        design_envelope_id: DESIGN_ENVELOPE,
        indicator_kind: "alr",
    };

    fn cutoff(value: &str) -> KnowledgeCutoff {
        KnowledgeCutoff::parse_rfc3339(value).expect("cutoff")
    }

    fn receipt(
        available_at: &str,
        receipt_cutoff: &str,
    ) -> RubinProjectionActivationReceiptV1 {
        RubinProjectionActivationReceiptV1::new(
            APPROVED.generator_contract_id,
            APPROVED.generator_contract_version,
            APPROVED.analysis_contract_id,
            APPROVED.analysis_contract_version,
            APPROVED.validation_evidence_id,
            APPROVED.validation_evidence_sha256,
            AvailableTime::parse_rfc3339(available_at).expect("availability"),
            SNAPSHOT_ID,
            cutoff(receipt_cutoff),
            APPROVED.design_envelope_id,
        )
        .expect("receipt")
    }

    #[test]
    fn exact_fake_pairing_is_eligible_and_equivalent_cutoffs_match_by_instant() {
        let receipt = receipt("2026-07-31T23:59:59Z", "2026-08-01T01:00:00+01:00");
        assert_eq!(
            decide_with_registry(
                Some(&receipt),
                SNAPSHOT_ID,
                cutoff("2026-08-01T00:00:00Z"),
                IndicatorKind::AdditiveLogRatio,
                &[APPROVED],
            ),
            RubinProjectionActivationDecision::Eligible
        );
    }

    #[test]
    fn authority_mismatches_and_late_evidence_fail_closed() {
        let valid = receipt("2026-07-31T23:59:59Z", "2026-08-01T00:00:00Z");
        assert_eq!(
            decide_with_registry(
                Some(&valid),
                "different-snapshot",
                cutoff("2026-08-01T00:00:00Z"),
                IndicatorKind::AdditiveLogRatio,
                &[APPROVED],
            ),
            RubinProjectionActivationDecision::Rejected
        );
        assert_eq!(
            decide_with_registry(
                Some(&valid),
                SNAPSHOT_ID,
                cutoff("2026-08-02T00:00:00Z"),
                IndicatorKind::AdditiveLogRatio,
                &[APPROVED],
            ),
            RubinProjectionActivationDecision::Rejected
        );
        let late = receipt("2026-08-01T00:00:01Z", "2026-08-01T00:00:00Z");
        assert_eq!(
            decide_with_registry(
                Some(&late),
                SNAPSHOT_ID,
                cutoff("2026-08-01T00:00:00Z"),
                IndicatorKind::AdditiveLogRatio,
                &[APPROVED],
            ),
            RubinProjectionActivationDecision::Rejected
        );
        assert_eq!(
            decide_with_registry(
                Some(&valid),
                SNAPSHOT_ID,
                cutoff("2026-08-01T00:00:00Z"),
                IndicatorKind::IsometricLogRatio,
                &[APPROVED],
            ),
            RubinProjectionActivationDecision::Rejected
        );
    }

    #[test]
    fn registry_requires_exact_generator_analysis_evidence_digest_and_envelope() {
        let valid = receipt("2026-07-31T23:59:59Z", "2026-08-01T00:00:00Z");
        let mismatches = [
            ApprovedRubinProjectionPairing {
                generator_contract_version: "other-generator-version",
                ..APPROVED
            },
            ApprovedRubinProjectionPairing {
                analysis_contract_version: "other-analysis-version",
                ..APPROVED
            },
            ApprovedRubinProjectionPairing {
                validation_evidence_id: "other-evidence",
                ..APPROVED
            },
            ApprovedRubinProjectionPairing {
                validation_evidence_sha256:
                    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                ..APPROVED
            },
            ApprovedRubinProjectionPairing {
                design_envelope_id: "other-envelope",
                ..APPROVED
            },
        ];
        for mismatch in mismatches {
            assert_eq!(
                decide_with_registry(
                    Some(&valid),
                    SNAPSHOT_ID,
                    cutoff("2026-08-01T00:00:00Z"),
                    IndicatorKind::AdditiveLogRatio,
                    &[mismatch],
                ),
                RubinProjectionActivationDecision::Rejected
            );
        }
    }

    #[test]
    fn receipt_wire_refuses_unknown_fields_noncanonical_time_and_invalid_identity() {
        let receipt = receipt("2026-07-31T23:59:59Z", "2026-08-01T00:00:00Z");
        let canonical = receipt.to_json().expect("json");
        let mut unknown: serde_json::Value = serde_json::from_str(&canonical).expect("json");
        unknown["unexpected"] = serde_json::json!(true);
        assert_eq!(
            RubinProjectionActivationReceiptV1::from_json(&unknown.to_string()),
            Err(AnalysisEngineError::InvalidEvidence)
        );

        let mut noncanonical: serde_json::Value = serde_json::from_str(&canonical).expect("json");
        noncanonical["knowledge_cutoff"] = serde_json::json!("2026-08-01T01:00:00+01:00");
        assert_eq!(
            RubinProjectionActivationReceiptV1::from_json(&noncanonical.to_string()),
            Err(AnalysisEngineError::InvalidEvidence)
        );

        let invalid = RubinProjectionActivationReceiptV1::new(
            "",
            APPROVED.generator_contract_version,
            APPROVED.analysis_contract_id,
            APPROVED.analysis_contract_version,
            APPROVED.validation_evidence_id,
            "ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789",
            AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability"),
            SNAPSHOT_ID,
            cutoff("2026-08-01T00:00:00Z"),
            APPROVED.design_envelope_id,
        );
        assert_eq!(invalid, Err(AnalysisEngineError::InvalidEvidence));
    }

    #[test]
    fn receipt_size_is_checked_before_parse_and_digest_is_stable() {
        let oversized = "x".repeat(RUBIN_PROJECTION_ACTIVATION_RECEIPT_BYTE_LIMIT + 1);
        assert_eq!(
            RubinProjectionActivationReceiptV1::from_json(&oversized),
            Err(AnalysisEngineError::LimitExceeded)
        );
        let receipt = receipt("2026-07-31T23:59:59Z", "2026-08-01T00:00:00Z");
        assert_eq!(receipt.sha256().expect("digest").len(), 64);
        assert_eq!(
            RubinProjectionActivationReceiptV1::from_json(&receipt.to_json().expect("json")),
            Ok(receipt)
        );
    }
}
