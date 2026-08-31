//! Operator-usable `tepp.scientific_acceptance.v1` terminal-result wire.
//!
//! GAP-003A second slice: `AnalysisRunRequest` and `AnalysisRunAccepted` stay
//! metric-free receipts. Only a succeeded terminal result with output profile
//! `scientific_acceptance_v1` may carry the scientific-acceptance artifact.
//! Persistence and Compose recovery remain GAP-003B.

use crate::ApiError;
use crate::wire::{from_json, require_byte_limit, require_nonempty, to_json_with_limit};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Versioned scientific-acceptance artifact schema.
pub const SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION: &str = "tepp.scientific_acceptance.v1";
/// Output profile that authorizes this artifact on a terminal result.
pub const SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE: &str = "scientific_acceptance_v1";
/// CPU `f64` reference model identity for scientific acceptance.
pub const SCIENTIFIC_ACCEPTANCE_MODEL: &str = "validation_cpu_f64_v1";
/// Backend identity bound into the artifact.
pub const SCIENTIFIC_ACCEPTANCE_BACKEND: &str = "cpu";
/// Numeric precision bound into the artifact.
pub const SCIENTIFIC_ACCEPTANCE_PRECISION: &str = "f64";
/// Default maximum serialized artifact size.
pub const DEFAULT_SCIENTIFIC_ACCEPTANCE_BYTE_LIMIT: usize = 16 * 1024;

const FORBIDDEN_RECEIPT_KEYS: [&str; 8] = [
    "rmse",
    "mean_bias",
    "interval_coverage",
    "se_gate_accepted",
    "se_gate_k",
    "scientific_acceptance",
    "report",
    "coverage_wilson_lower",
];

/// Nested recovery report carried by the terminal artifact.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[serde(deny_unknown_fields)]
pub struct ScientificAcceptanceReport {
    /// Study label (not free-form PII).
    pub study_label: String,
    /// Root-mean-square error.
    pub rmse: f64,
    /// RMSE standard error.
    pub rmse_standard_error: f64,
    /// Mean signed bias.
    pub mean_bias: f64,
    /// Bias standard error.
    pub bias_standard_error: f64,
    /// Empirical interval coverage.
    pub interval_coverage: f64,
    /// Wilson lower bound for coverage.
    pub coverage_wilson_lower: f64,
    /// Wilson upper bound for coverage.
    pub coverage_wilson_upper: f64,
    /// Temporal-order accuracy.
    pub temporal_order_accuracy: f64,
}

impl ScientificAcceptanceReport {
    fn validate(&self) -> Result<(), ApiError> {
        require_nonempty(&self.study_label)?;
        if self.study_label.trim() != self.study_label {
            return Err(ApiError::InvalidWirePayload);
        }
        for value in [
            self.rmse,
            self.rmse_standard_error,
            self.mean_bias,
            self.bias_standard_error,
            self.interval_coverage,
            self.coverage_wilson_lower,
            self.coverage_wilson_upper,
            self.temporal_order_accuracy,
        ] {
            if !value.is_finite() {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        Ok(())
    }
}

/// Digest-bound scientific-acceptance artifact for one completed run.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[serde(deny_unknown_fields)]
pub struct ScientificAcceptanceArtifact {
    /// Versioned artifact schema.
    pub schema_version: String,
    /// Durable run identity echoed from the receipt.
    pub run_id: String,
    /// Canonical lowercase SHA-256 of the run binding.
    pub binding_sha256: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical cutoff applied to availability.
    pub knowledge_cutoff: String,
    /// Bound model identity.
    pub model: String,
    /// Bound numeric seed.
    pub seed: u64,
    /// Bound compute backend.
    pub backend: String,
    /// Bound numeric precision.
    pub precision: String,
    /// Output profile that selected this executor.
    pub output_profile: String,
    /// Number of cutoff-eligible evidence identities.
    pub eligible_evidence_count: u64,
    /// Whether RMSE toward 0 passed the SE-aware gate.
    pub se_gate_accepted: bool,
    /// SE-gate multiplier used for acceptance.
    pub se_gate_k: f64,
    /// Machine-readable recovery report.
    pub report: ScientificAcceptanceReport,
}

impl ScientificAcceptanceArtifact {
    /// Parse and validate an artifact with the default payload limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, identity, digest, or numeric error.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_SCIENTIFIC_ACCEPTANCE_BYTE_LIMIT)
    }

    /// Parse and validate an artifact with a caller-supplied payload limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, identity, digest, limit, or numeric error.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        let value: Self = from_json(payload)?;
        value.validate()?;
        Ok(value)
    }

    /// Serialize this validated artifact.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed validation or serialization error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json_with_limit(self, DEFAULT_SCIENTIFIC_ACCEPTANCE_BYTE_LIMIT)
    }

    /// Canonical lowercase SHA-256 of the validated artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::to_json`].
    pub fn sha256(&self) -> Result<String, ApiError> {
        let json = self.to_json()?;
        Ok(encode_hex(&Sha256::digest(json.as_bytes())))
    }

    pub(crate) fn validate(&self) -> Result<(), ApiError> {
        if self.schema_version != SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION
            || self.output_profile != SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE
            || self.model != SCIENTIFIC_ACCEPTANCE_MODEL
            || self.backend != SCIENTIFIC_ACCEPTANCE_BACKEND
            || self.precision != SCIENTIFIC_ACCEPTANCE_PRECISION
            || self.eligible_evidence_count == 0
            || !self.se_gate_k.is_finite()
            || self.se_gate_k < 0.0
        {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.snapshot_id)?;
        require_nonempty(&self.knowledge_cutoff)?;
        if !is_canonical_sha256(&self.binding_sha256)
            || self.binding_sha256.bytes().all(|byte| byte == b'0')
        {
            return Err(ApiError::InvalidWirePayload);
        }
        self.report.validate()
    }
}

impl Eq for ScientificAcceptanceReport {}

impl Eq for ScientificAcceptanceArtifact {}

/// Return whether a receipt JSON object carries scientific-metric keys.
///
/// Request and accepted receipts must remain metric-free. Unknown-field denial
/// is the wire gate; this helper names the forbidden keys for operators.
#[must_use]
pub fn receipt_json_carries_scientific_metrics(payload: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    FORBIDDEN_RECEIPT_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
}

/// Refuse a request or accepted receipt that already carries metrics.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present on a receipt object.
pub fn refuse_metrics_on_receipt(payload: &str) -> Result<(), ApiError> {
    if receipt_json_carries_scientific_metrics(payload) {
        Err(ApiError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_SCIENTIFIC_ACCEPTANCE_BYTE_LIMIT, SCIENTIFIC_ACCEPTANCE_BACKEND,
        SCIENTIFIC_ACCEPTANCE_MODEL, SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
        SCIENTIFIC_ACCEPTANCE_PRECISION, SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION,
        ScientificAcceptanceArtifact, ScientificAcceptanceReport, encode_hex, is_canonical_sha256,
        receipt_json_carries_scientific_metrics, refuse_metrics_on_receipt,
    };
    use crate::ApiError;

    fn report() -> ScientificAcceptanceReport {
        ScientificAcceptanceReport {
            study_label: "gap-003a-terminal".into(),
            rmse: 0.04,
            rmse_standard_error: 0.01,
            mean_bias: 0.0,
            bias_standard_error: 0.02,
            interval_coverage: 0.95,
            coverage_wilson_lower: 0.90,
            coverage_wilson_upper: 0.98,
            temporal_order_accuracy: 1.0,
        }
    }

    fn artifact() -> ScientificAcceptanceArtifact {
        ScientificAcceptanceArtifact {
            schema_version: SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION.into(),
            run_id: "tepp-validation-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            binding_sha256: "a".repeat(64),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model: SCIENTIFIC_ACCEPTANCE_MODEL.into(),
            seed: 7,
            backend: SCIENTIFIC_ACCEPTANCE_BACKEND.into(),
            precision: SCIENTIFIC_ACCEPTANCE_PRECISION.into(),
            output_profile: SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE.into(),
            eligible_evidence_count: 4,
            se_gate_accepted: true,
            se_gate_k: 3.0,
            report: report(),
        }
    }

    #[test]
    fn artifact_round_trips_and_rejects_hostile_payloads() {
        let value = artifact();
        let json = value.to_json().expect("json");
        assert_eq!(
            ScientificAcceptanceArtifact::from_json(&json).expect("decode"),
            value
        );
        assert_eq!(value.sha256().expect("digest").len(), 64);
        assert!(json.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION));

        let unknown = json.replacen('{', r#"{"extra":true,"#, 1);
        assert_eq!(
            ScientificAcceptanceArtifact::from_json(&unknown),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ScientificAcceptanceArtifact::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert!(DEFAULT_SCIENTIFIC_ACCEPTANCE_BYTE_LIMIT >= json.len());
        assert_eq!(
            ScientificAcceptanceArtifact::from_json("not-json"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn identities_metrics_and_empty_evidence_fail_closed() {
        let mut zero = artifact();
        zero.binding_sha256 = "0".repeat(64);
        assert_eq!(zero.to_json(), Err(ApiError::InvalidWirePayload));
        let mut empty = artifact();
        empty.eligible_evidence_count = 0;
        assert_eq!(empty.to_json(), Err(ApiError::InvalidWirePayload));
        let mut schema = artifact();
        schema.schema_version = "tepp.scientific_acceptance.v0".into();
        assert_eq!(schema.to_json(), Err(ApiError::InvalidWirePayload));
        let mut profile = artifact();
        profile.output_profile = "validation-report".into();
        assert_eq!(profile.to_json(), Err(ApiError::InvalidWirePayload));
        let mut model = artifact();
        model.model = "other".into();
        assert_eq!(model.to_json(), Err(ApiError::InvalidWirePayload));
        let mut backend = artifact();
        backend.backend = "gpu".into();
        assert_eq!(backend.to_json(), Err(ApiError::InvalidWirePayload));
        let mut precision = artifact();
        precision.precision = "f32".into();
        assert_eq!(precision.to_json(), Err(ApiError::InvalidWirePayload));
        let mut gate = artifact();
        gate.se_gate_k = -1.0;
        assert_eq!(gate.to_json(), Err(ApiError::InvalidWirePayload));
        let mut nan = artifact();
        nan.se_gate_k = f64::NAN;
        assert_eq!(nan.to_json(), Err(ApiError::InvalidWirePayload));
        let mut padded = artifact();
        padded.report.study_label = " padded".into();
        assert_eq!(padded.to_json(), Err(ApiError::InvalidWirePayload));
        let mut blank = artifact();
        blank.report.study_label.clear();
        assert_eq!(blank.to_json(), Err(ApiError::InvalidWirePayload));
        let mut inf = artifact();
        inf.report.rmse = f64::INFINITY;
        assert_eq!(inf.to_json(), Err(ApiError::InvalidWirePayload));
        let mut run = artifact();
        run.run_id.clear();
        assert_eq!(run.to_json(), Err(ApiError::InvalidWirePayload));
        let mut snap = artifact();
        snap.snapshot_id.clear();
        assert_eq!(snap.to_json(), Err(ApiError::InvalidWirePayload));
        let mut cutoff = artifact();
        cutoff.knowledge_cutoff.clear();
        assert_eq!(cutoff.to_json(), Err(ApiError::InvalidWirePayload));
        let mut mixed = artifact();
        mixed.binding_sha256 = "A".repeat(64);
        assert_eq!(mixed.to_json(), Err(ApiError::InvalidWirePayload));
        let mut short = artifact();
        short.binding_sha256 = "aa".into();
        assert_eq!(short.to_json(), Err(ApiError::InvalidWirePayload));
    }

    #[test]
    fn receipts_must_not_carry_scientific_metrics() {
        assert!(!receipt_json_carries_scientific_metrics(
            r#"{"contract_version":1,"run_id":"r","run_state":"accepted","idempotency_key":"i"}"#
        ));
        assert!(receipt_json_carries_scientific_metrics(
            r#"{"contract_version":1,"rmse":0.1}"#
        ));
        assert!(receipt_json_carries_scientific_metrics(
            r#"{"scientific_acceptance":{}}"#
        ));
        assert_eq!(
            refuse_metrics_on_receipt(r#"{"mean_bias":0.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_receipt(r#"{"contract_version":1}"#),
            Ok(())
        );
        assert!(!receipt_json_carries_scientific_metrics("[]"));
        assert!(!receipt_json_carries_scientific_metrics("not-json"));
        assert_eq!(encode_hex(&[0x0a, 0xff]), "0aff");
        assert!(is_canonical_sha256(&"ab".repeat(32)));
        assert!(!is_canonical_sha256("zz"));
    }
}
