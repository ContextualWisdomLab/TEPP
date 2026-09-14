//! Concrete numeric estimator-payload authority for Rubin projection activation.
//!
//! The underlying activation policy owns generator/analysis/evidence approval,
//! temporal provenance, source-snapshot authority, and validated design points.
//! This boundary additionally binds one activation receipt to the exact
//! cutoff-admitted numeric payload consumed by the run: the factor-score design
//! vector and complete-data indicator-draw matrix. A class-level approved
//! generator and a matching matrix shape are not authority for arbitrary values.

use psychometric_core::IndicatorKind;
use serde::{
    Deserialize,
    de::{Error as _, IgnoredAny, MapAccess, Visitor},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;
use temporal_core::{AvailableTime, KnowledgeCutoff};

use crate::rubin_projection_activation::{
    RUBIN_PROJECTION_ACTIVATION_RECEIPT_BYTE_LIMIT as INNER_RECEIPT_BYTE_LIMIT,
    RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION as INNER_RECEIPT_SCHEMA_VERSION,
    RubinProjectionActivationDecision,
    RubinProjectionActivationReceiptV1 as InnerRubinProjectionActivationReceiptV1,
    decide_rubin_projection_activation as decide_inner_rubin_projection_activation,
};
use crate::{AnalysisEngineError, format_digest};

/// Versioned public wire schema for the payload-bound Rubin activation receipt.
///
/// This contract is still Draft and has never been released from protected
/// `main`; the per-run payload commitment is therefore part of the eventual v1
/// definition, not a post-release incompatible mutation.
pub const RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION: &str = INNER_RECEIPT_SCHEMA_VERSION;
/// Maximum canonical JSON size accepted for one payload-bound activation receipt.
pub const RUBIN_PROJECTION_ACTIVATION_RECEIPT_BYTE_LIMIT: usize = INNER_RECEIPT_BYTE_LIMIT;

/// Payload-bound Rubin projection activation receipt.
///
/// The nested activation authority remains owned by the existing projection
/// policy. This type adds a canonical SHA-256 commitment to the concrete
/// cutoff-admitted numeric estimator payload. The digest commits both the
/// factor-score design vector and the draw matrix produced by the executor. It
/// prevents a receipt issued for one payload from being replayed for different
/// factor scores or draw values with the same dimensions. It does not, by
/// itself, attest that a caller executed an approved generator or factor-mapping
/// implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RubinProjectionActivationReceiptV1 {
    inner: InnerRubinProjectionActivationReceiptV1,
    estimator_payload_sha256: String,
}

impl RubinProjectionActivationReceiptV1 {
    /// Construct a payload-bound canonical activation receipt.
    ///
    /// `generator_contract` and `analysis_contract` are `(id, version)` pairs.
    /// `validation_evidence` is `(id, sha256, available_time)`. Snapshot and
    /// estimator-payload digests are independent commitments and must both be
    /// lowercase canonical SHA-256 values.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed validation error when the underlying authority
    /// receipt or concrete estimator-payload digest is invalid.
    pub fn new(
        generator_contract: (&str, &str),
        analysis_contract: (&str, &str),
        validation_evidence: (&str, &str, AvailableTime),
        source_snapshot_id: impl Into<String>,
        source_snapshot_sha256: impl Into<String>,
        estimator_payload_sha256: impl Into<String>,
        knowledge_cutoff: KnowledgeCutoff,
        design_envelope_id: impl Into<String>,
    ) -> Result<Self, AnalysisEngineError> {
        let estimator_payload_sha256 = estimator_payload_sha256.into();
        if !valid_sha256(&estimator_payload_sha256) {
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
            estimator_payload_sha256,
        })
    }

    /// Parse and fully validate bounded payload-bound receipt JSON.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::LimitExceeded`] before parsing an
    /// oversized payload and [`AnalysisEngineError::InvalidEvidence`] when the
    /// payload commitment, schema version, duplicate authority members, or
    /// underlying activation contract is invalid.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        require_receipt_byte_limit(payload.len())?;
        require_unique_top_level_keys(payload)?;
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
        let estimator_payload_sha256 = object
            .remove("estimator_payload_sha256")
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .ok_or(AnalysisEngineError::InvalidEvidence)?;
        if !valid_sha256(&estimator_payload_sha256) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let inner_json =
            serde_json::to_string(object).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        let inner = InnerRubinProjectionActivationReceiptV1::from_json(&inner_json)?;
        Ok(Self {
            inner,
            estimator_payload_sha256,
        })
    }

    /// Serialize canonical validated receipt JSON with the estimator-payload commitment.
    ///
    /// # Errors
    ///
    /// Returns a validation, size, or serialization failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        if !valid_sha256(&self.estimator_payload_sha256) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let inner_json = self.inner.to_json()?;
        let mut value: Value =
            serde_json::from_str(&inner_json).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        value
            .as_object_mut()
            .ok_or(AnalysisEngineError::SerializationFailure)?
            .insert(
                "estimator_payload_sha256".into(),
                Value::String(self.estimator_payload_sha256.clone()),
            );
        let payload =
            serde_json::to_string(&value).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        require_receipt_byte_limit(payload.len())?;
        Ok(payload)
    }

    /// Return the lowercase SHA-256 digest of canonical payload-bound receipt JSON.
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

    /// Return the canonical SHA-256 of the exact numeric estimator payload.
    #[must_use]
    pub fn estimator_payload_sha256(&self) -> &str {
        &self.estimator_payload_sha256
    }

    /// Return the canonical knowledge-cutoff wire value.
    #[must_use]
    pub fn knowledge_cutoff(&self) -> &str {
        self.inner.knowledge_cutoff()
    }
}

/// Decide whether a payload-bound receipt authorizes Rubin projection.
///
/// The runtime estimator-payload digest is independent input. It must be
/// canonical and must match the receipt before the underlying
/// generator/analysis/evidence authority decision is evaluated. Production
/// approval remains controlled exclusively by the underlying owner registry.
#[must_use]
pub fn decide_rubin_projection_activation(
    receipt: Option<&RubinProjectionActivationReceiptV1>,
    expected_snapshot_id: &str,
    expected_snapshot_sha256: &str,
    expected_estimator_payload_sha256: &str,
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
    if validated_inner_for_runtime_payload(receipt, expected_estimator_payload_sha256).is_err() {
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

fn validated_inner_for_runtime_payload<'a>(
    receipt: &'a RubinProjectionActivationReceiptV1,
    expected_estimator_payload_sha256: &str,
) -> Result<&'a InnerRubinProjectionActivationReceiptV1, RubinProjectionActivationDecision> {
    if !valid_sha256(expected_estimator_payload_sha256)
        || receipt.estimator_payload_sha256 != expected_estimator_payload_sha256
    {
        return Err(RubinProjectionActivationDecision::Rejected);
    }
    Ok(&receipt.inner)
}

fn require_unique_top_level_keys(payload: &str) -> Result<(), AnalysisEngineError> {
    let mut deserializer = serde_json::Deserializer::from_str(payload);
    UniqueTopLevelKeys::deserialize(&mut deserializer)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    deserializer
        .end()
        .map_err(|_| AnalysisEngineError::InvalidEvidence)
}

struct UniqueTopLevelKeys;

impl<'de> Deserialize<'de> for UniqueTopLevelKeys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(UniqueTopLevelKeysVisitor)
    }
}

struct UniqueTopLevelKeysVisitor;

impl<'de> Visitor<'de> for UniqueTopLevelKeysVisitor {
    type Value = UniqueTopLevelKeys;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON object with unique top-level member names")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key) {
                return Err(M::Error::custom("duplicate top-level JSON member"));
            }
            map.next_value::<IgnoredAny>()?;
        }
        Ok(UniqueTopLevelKeys)
    }
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
        validated_inner_for_runtime_payload,
    };
    use crate::RUBIN_LOADING_MODEL_CONTRACT_VERSION;
    use temporal_core::{AvailableTime, KnowledgeCutoff};

    const SNAPSHOT_DIGEST: &str =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const ESTIMATOR_PAYLOAD_DIGEST: &str =
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const OTHER_ESTIMATOR_PAYLOAD_DIGEST: &str =
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
            ESTIMATOR_PAYLOAD_DIGEST,
            KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
            "rubin-gaussian-single-level-candidate-v1",
        )
        .expect("receipt")
    }

    #[test]
    fn estimator_payload_gate_rejects_substitution_before_authority_decision() {
        let receipt = receipt();
        assert!(validated_inner_for_runtime_payload(&receipt, ESTIMATOR_PAYLOAD_DIGEST).is_ok());
        assert!(
            validated_inner_for_runtime_payload(&receipt, OTHER_ESTIMATOR_PAYLOAD_DIGEST).is_err()
        );
        assert!(validated_inner_for_runtime_payload(&receipt, "not-a-digest").is_err());
    }

    #[test]
    fn estimator_payload_digest_is_part_of_canonical_receipt_and_receipt_digest() {
        let receipt = receipt();
        let canonical = receipt.to_json().expect("json");
        let wire: serde_json::Value = serde_json::from_str(&canonical).expect("json");
        assert_eq!(
            wire.get("schema_version").and_then(serde_json::Value::as_str),
            Some(RUBIN_PROJECTION_ACTIVATION_RECEIPT_SCHEMA_VERSION)
        );
        assert!(canonical.contains(ESTIMATOR_PAYLOAD_DIGEST));
        let reparsed = RubinProjectionActivationReceiptV1::from_json(&canonical).expect("receipt");
        assert_eq!(reparsed, receipt);

        let mut changed = wire;
        changed["estimator_payload_sha256"] = serde_json::json!(OTHER_ESTIMATOR_PAYLOAD_DIGEST);
        let changed = RubinProjectionActivationReceiptV1::from_json(&changed.to_string())
            .expect("alternate valid digest");
        assert_ne!(receipt.sha256().expect("digest"), changed.sha256().expect("digest"));
    }
}
