//! Concrete complete-data draw authority for Rubin projection activation.
//!
//! The underlying activation policy owns generator/analysis/evidence approval,
//! temporal provenance, source-snapshot authority, and validated design points.
//! This boundary additionally binds one activation receipt to the exact
//! complete-data draw payload consumed by the run. A class-level approved
//! generator and a matching matrix shape are not authority for arbitrary values.

use psychometric_core::IndicatorKind;
use serde_json::Value;
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, KnowledgeCutoff};

use crate::{AnalysisEngineError, format_digest};
use crate::rubin_projection_activation::{
    RUBIN_PROJECTION_ACTIVATION_RECEIPT_BYTE_LIMIT as INNER_RECEIPT_BYTE_LIMIT,
    RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION as INNER_RECEIPT_SCHEMA_VERSION,
    RubinProjectionActivationDecision,
    RubinProjectionActivationReceiptV1 as InnerRubinProjectionActivationReceiptV1,
    decide_rubin_projection_activation as decide_inner_rubin_projection_activation,
};

/// Versioned public wire schema for the draw-bound Rubin activation receipt.
///
/// This contract is still Draft and has never been released from protected
/// `main`; the draw commitment is therefore part of the eventual v1 definition,
/// not a post-release incompatible mutation.
pub const RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION: &str = INNER_RECEIPT_SCHEMA_VERSION;
/// Maximum canonical JSON size accepted for one draw-bound activation receipt.
pub const RUBIN_PROJECTION_ACTIVATION_RECEIPT_BYTE_LIMIT: usize = INNER_RECEIPT_BYTE_LIMIT;

/// Draw-bound Rubin projection activation receipt.
///
/// The nested activation authority remains owned by the existing projection
/// policy. This type adds a canonical SHA-256 commitment to the concrete
/// complete-data draw payload and flattens that commitment into the public wire
/// receipt. The digest prevents a receipt issued for one payload from being
/// replayed for different draw values with the same dimensions. It does not, by
/// itself, attest that a caller executed an approved generator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RubinProjectionActivationReceiptV1 {
    inner: InnerRubinProjectionActivationReceiptV1,
    complete_data_draws_sha256: String,
}

impl RubinProjectionActivationReceiptV1 {
    /// Construct a draw-bound canonical activation receipt.
    ///
    /// `generator_contract` and `analysis_contract` are `(id, version)` pairs.
    /// `validation_evidence` is `(id, sha256, available_time)`. Snapshot and draw
    /// payload digests are independent commitments and must both be lowercase
    /// canonical SHA-256 values.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed validation error when the underlying authority
    /// receipt or concrete draw-payload digest is invalid.
    pub fn new(
        generator_contract: (&str, &str),
        analysis_contract: (&str, &str),
        validation_evidence: (&str, &str, AvailableTime),
        source_snapshot_id: impl Into<String>,
        source_snapshot_sha256: impl Into<String>,
        complete_data_draws_sha256: impl Into<String>,
        knowledge_cutoff: KnowledgeCutoff,
        design_envelope_id: impl Into<String>,
    ) -> Result<Self, AnalysisEngineError> {
        let complete_data_draws_sha256 = complete_data_draws_sha256.into();
        if !valid_sha256(&complete_data_draws_sha256) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let inner = InnerRubinProjectionActivationReceiptV1::new(
            generator_contract,
            analysis_contract,
            validation_evidence,
            source_snapshot_id,
            source_snapshot_sha256,
            knowledge_cutoff,
            design_envelope_id,
        )?;
        Ok(Self {
            inner,
            complete_data_draws_sha256,
        })
    }

    /// Parse and fully validate bounded draw-bound receipt JSON.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::LimitExceeded`] before parsing an
    /// oversized payload and [`AnalysisEngineError::InvalidEvidence`] when the
    /// draw commitment, schema version, or underlying activation contract is invalid.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        require_receipt_byte_limit(payload.len())?;
        let mut value: Value =
            serde_json::from_str(payload).map_err(|_| AnalysisEngineError::InvalidEvidence)?;
        let object = value
            .as_object_mut()
            .ok_or(AnalysisEngineError::InvalidEvidence)?;
        if object.get("schema_version").and_then(Value::as_str)
            != Some(RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION)
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let complete_data_draws_sha256 = object
            .remove("complete_data_draws_sha256")
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .ok_or(AnalysisEngineError::InvalidEvidence)?;
        if !valid_sha256(&complete_data_draws_sha256) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let inner_json =
            serde_json::to_string(object).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        let inner = InnerRubinProjectionActivationReceiptV1::from_json(&inner_json)?;
        Ok(Self {
            inner,
            complete_data_draws_sha256,
        })
    }

    /// Serialize canonical validated receipt JSON with the draw commitment.
    ///
    /// # Errors
    ///
    /// Returns a validation, size, or serialization failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        if !valid_sha256(&self.complete_data_draws_sha256) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let inner_json = self.inner.to_json()?;
        let mut value: Value =
            serde_json::from_str(&inner_json).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        value
            .as_object_mut()
            .ok_or(AnalysisEngineError::SerializationFailure)?
            .insert(
                "complete_data_draws_sha256".into(),
                Value::String(self.complete_data_draws_sha256.clone()),
            );
        let payload =
            serde_json::to_string(&value).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        require_receipt_byte_limit(payload.len())?;
        Ok(payload)
    }

    /// Return the lowercase SHA-256 digest of canonical draw-bound receipt JSON.
    ///
    /// # Errors
    ///
    /// Returns a validation, size, or serialization failure.
    pub fn sha256(&self) -> Result<String, AnalysisEngineError> {
        self.to_json()
            .map(|json| format_digest(Sha256::digest(json.into_bytes())))
    }

    /// Return the immutable source snapshot identity bound into the receipt.
    #[must_use]
    pub fn source_snapshot_id(&self) -> &str {
        self.inner.source_snapshot_id()
    }

    /// Return the canonical source snapshot SHA-256 bound into the receipt.
    #[must_use]
    pub fn source_snapshot_sha256(&self) -> &str {
        self.inner.source_snapshot_sha256()
    }

    /// Return the canonical SHA-256 of the exact complete-data draw payload.
    #[must_use]
    pub fn complete_data_draws_sha256(&self) -> &str {
        &self.complete_data_draws_sha256
    }

    /// Return the canonical knowledge-cutoff wire value.
    #[must_use]
    pub fn knowledge_cutoff(&self) -> &str {
        self.inner.knowledge_cutoff()
    }
}

/// Decide whether a draw-bound receipt authorizes Rubin projection.
///
/// The runtime draw digest is independent input. It must be canonical and must
/// match the receipt before the underlying generator/analysis/evidence authority
/// decision is evaluated. Production approval remains controlled exclusively by
/// the underlying owner registry.
#[must_use]
pub fn decide_rubin_projection_activation(
    receipt: Option<&RubinProjectionActivationReceiptV1>,
    expected_snapshot_id: &str,
    expected_snapshot_sha256: &str,
    expected_complete_data_draws_sha256: &str,
    expected_knowledge_cutoff: KnowledgeCutoff,
    observation_count: u64,
    draw_count: u64,
    indicator_kind: IndicatorKind,
) -> RubinProjectionActivationDecision {
    let Some(receipt) = receipt else {
        return decide_inner_rubin_projection_activation(
            None,
            expected_snapshot_id,
            expected_snapshot_sha256,
            expected_knowledge_cutoff,
            observation_count,
            draw_count,
            indicator_kind,
        );
    };
    if validated_inner_for_runtime_draws(receipt, expected_complete_data_draws_sha256).is_err() {
        return RubinProjectionActivationDecision::Rejected;
    }
    decide_inner_rubin_projection_activation(
        Some(&receipt.inner),
        expected_snapshot_id,
        expected_snapshot_sha256,
        expected_knowledge_cutoff,
        observation_count,
        draw_count,
        indicator_kind,
    )
}

fn validated_inner_for_runtime_draws<'a>(
    receipt: &'a RubinProjectionActivationReceiptV1,
    expected_complete_data_draws_sha256: &str,
) -> Result<&'a InnerRubinProjectionActivationReceiptV1, RubinProjectionActivationDecision> {
    if !valid_sha256(expected_complete_data_draws_sha256)
        || receipt.complete_data_draws_sha256 != expected_complete_data_draws_sha256
    {
        return Err(RubinProjectionActivationDecision::Rejected);
    }
    Ok(&receipt.inner)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn require_receipt_byte_limit(payload_len: usize) -> Result<(), AnalysisEngineError> {
    if payload_len > RUBIN_PROJECTION_ACTIVATION_RECEIPT_BYTE_LIMIT {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION, RubinProjectionActivationReceiptV1,
        validated_inner_for_runtime_draws,
    };
    use crate::RUBIN_LOADING_MODEL_CONTRACT_VERSION;
    use temporal_core::{AvailableTime, KnowledgeCutoff};

    const SNAPSHOT_DIGEST: &str =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DRAW_PAYLOAD_DIGEST: &str =
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const OTHER_DRAW_PAYLOAD_DIGEST: &str =
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const EVIDENCE_DIGEST: &str =
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn receipt() -> RubinProjectionActivationReceiptV1 {
        RubinProjectionActivationReceiptV1::new(
            ("gaussian_complete_data_draws", "candidate-v1"),
            (
                "rubin_loading_uncertainty",
                RUBIN_LOADING_MODEL_CONTRACT_VERSION,
            ),
            (
                "validation-evidence-rubin-candidate-v1",
                EVIDENCE_DIGEST,
                AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability"),
            ),
            "snapshot-rubin-activation",
            SNAPSHOT_DIGEST,
            DRAW_PAYLOAD_DIGEST,
            KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
            "rubin-gaussian-single-level-candidate-v1",
        )
        .expect("receipt")
    }

    #[test]
    fn concrete_draw_payload_gate_rejects_substitution_before_authority_decision() {
        let receipt = receipt();
        assert!(validated_inner_for_runtime_draws(&receipt, DRAW_PAYLOAD_DIGEST).is_ok());
        assert!(validated_inner_for_runtime_draws(&receipt, OTHER_DRAW_PAYLOAD_DIGEST).is_err());
        assert!(validated_inner_for_runtime_draws(&receipt, "not-a-digest").is_err());
    }

    #[test]
    fn draw_digest_is_part_of_canonical_receipt_and_receipt_digest() {
        let receipt = receipt();
        let canonical = receipt.to_json().expect("json");
        let wire: serde_json::Value = serde_json::from_str(&canonical).expect("json");
        assert_eq!(
            wire.get("schema_version").and_then(serde_json::Value::as_str),
            Some(RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION)
        );
        assert!(canonical.contains(DRAW_PAYLOAD_DIGEST));
        let reparsed = RubinProjectionActivationReceiptV1::from_json(&canonical).expect("receipt");
        assert_eq!(reparsed, receipt);

        let mut changed = wire;
        changed["complete_data_draws_sha256"] = serde_json::json!(OTHER_DRAW_PAYLOAD_DIGEST);
        let changed = RubinProjectionActivationReceiptV1::from_json(&changed.to_string())
            .expect("alternate valid digest");
        assert_ne!(receipt.sha256().expect("digest"), changed.sha256().expect("digest"));
    }
}
