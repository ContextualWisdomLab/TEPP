//! Provider-owned export retrieval GET contracts.
//!
//! GAP-003A unique slice: `GET /v1/exports/{export_id}` returns the
//! metric-free identity of one purpose-bound export that `tepp-loopback`
//! already authorized. `POST /v1/exports` on `NaruonLiveService` authorizes
//! without minting a retrievable identity. Operators who hold a 200 decision
//! therefore cannot jump back to that export. This module does not serve
//! GET-by-id (#359), lifecycle POST (#360), cancel HTTP (#361), collection
//! GET (#368), retry POST (#369), stored-request GET (#377), retry-lineage
//! GET (#379), lookup GET (#380), retry-parent GET (#384), wait CLI (#406),
//! or GAP-010 Figma/export. Persistence remains GAP-003B.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT, NARUON_EXPORT_PATH};
use serde::{Deserialize, Serialize};

/// Maximum length accepted for an opaque export identity in the retrieval path.
pub const EXPORT_RETRIEVAL_ID_MAX_LEN: usize = 128;

/// Supported export-retrieval contract version.
pub const EXPORT_RETRIEVAL_CONTRACT_VERSION: u16 = 1;

/// The only authorization decision that may mint a retrieval receipt.
pub const EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE: &str = "purpose_bound_export_allowed";

const FORBIDDEN_EXPORT_RETRIEVAL_KEYS: [&str; 16] = [
    "rmse",
    "rmse_standard_error",
    "mean_bias",
    "bias_standard_error",
    "interval_coverage",
    "coverage_wilson_lower",
    "coverage_wilson_upper",
    "temporal_order_accuracy",
    "se_gate_accepted",
    "se_gate_k",
    "scientific_acceptance",
    "report",
    "terminal_result",
    "tenant_workspace_id",
    "principal_id",
    "includes_source_text",
];

/// Metric-free identity of one authorized export.
///
/// Operators jump from a 200 authorization receipt to the durable
/// `export_id` without scanning artifacts. The payload never carries a
/// terminal result, source body, or scientific-acceptance artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportRetrieval {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Opaque server-assigned export identity.
    pub export_id: String,
    /// Opaque artifact identity that was authorized.
    pub artifact_id: String,
    /// Stable machine-readable authorization decision code.
    pub decision_code: String,
    /// Declared analytical purpose as a wire name.
    pub purpose: String,
    /// Exact per-export idempotency key that minted this identity.
    pub idempotency_key: String,
}

impl ExportRetrieval {
    /// Construct a validated metric-free export-retrieval payload.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, an oversized
    /// identity, an unknown purpose wire name, or an unsupported contract
    /// version.
    pub fn new(
        export_id: impl Into<String>,
        artifact_id: impl Into<String>,
        decision_code: impl Into<String>,
        purpose: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let retrieval = Self {
            contract_version: EXPORT_RETRIEVAL_CONTRACT_VERSION,
            export_id: export_id.into(),
            artifact_id: artifact_id.into(),
            decision_code: decision_code.into(),
            purpose: purpose.into(),
            idempotency_key: idempotency_key.into(),
        };
        retrieval.validate()?;
        Ok(retrieval)
    }

    /// Parse and validate an export-retrieval payload with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate an export-retrieval payload with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_export_retrieval_payload(payload)?;
        let retrieval: Self = from_json(payload)?;
        retrieval.validate()?;
        Ok(retrieval)
    }

    /// Serialize this export-retrieval payload after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_export_retrieval_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, EXPORT_RETRIEVAL_CONTRACT_VERSION)?;
        require_nonempty(&self.export_id)?;
        require_nonempty(&self.artifact_id)?;
        require_nonempty(&self.decision_code)?;
        require_nonempty(&self.purpose)?;
        require_nonempty(&self.idempotency_key)?;
        if !purpose_is_known(&self.purpose) {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.decision_code != EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE {
            return Err(ApiError::AuthorizationDenied);
        }
        if self.export_id.len() > EXPORT_RETRIEVAL_ID_MAX_LEN
            || self.artifact_id.len() > EXPORT_RETRIEVAL_ID_MAX_LEN
            || self.idempotency_key.len() > EXPORT_RETRIEVAL_ID_MAX_LEN
        {
            return Err(ApiError::LimitExceeded);
        }
        Ok(())
    }
}

fn purpose_is_known(purpose: &str) -> bool {
    matches!(
        purpose,
        "scientific_validation"
            | "operational_monitoring"
            | "partner_disclosure"
            | "modular_service_consumer"
    )
}

/// Refuse export-retrieval JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted for the GET request body. Non-object JSON
/// fails closed as invalid wire.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present or the payload is a non-empty non-object.
pub fn refuse_metrics_on_export_retrieval_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    if !value.is_object() {
        return Err(ApiError::InvalidWirePayload);
    }
    if contains_forbidden_export_key(&value) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

fn contains_forbidden_export_key(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(object) => object.iter().any(|(key, value)| {
            FORBIDDEN_EXPORT_RETRIEVAL_KEYS.contains(&key.as_str())
                || contains_forbidden_export_key(value)
        }),
        serde_json::Value::Array(values) => values.iter().any(contains_forbidden_export_key),
        _ => false,
    }
}

/// Extract the opaque export identity from `GET /v1/exports/{export_id}`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for the collection path, extra
/// segments, a hostile encoding, or an empty identity, and
/// [`ApiError::LimitExceeded`] when the decoded identity exceeds
/// [`EXPORT_RETRIEVAL_ID_MAX_LEN`].
pub(crate) fn export_retrieval_path_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(NARUON_EXPORT_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let export_id = decode_path_segment(encoded)?;
    if export_id == "by-idempotency" {
        return Err(ApiError::InvalidWirePayload);
    }
    if export_id.len() > EXPORT_RETRIEVAL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(export_id)
}

/// Build a provider-owned `GET` export-retrieval exchange.
///
/// The builder refuses non-`https` origins and empty or oversized identities.
/// It does not inject credentials. The GET body is empty. The identity
/// travels in the path; the builder does not send an `idempotency-key`
/// header.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// identity, and [`ApiError::LimitExceeded`] when the identity exceeds
/// [`EXPORT_RETRIEVAL_ID_MAX_LEN`] bytes.
pub fn naruon_export_retrieval_exchange(
    origin: &str,
    export_id: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    require_nonempty(export_id)?;
    if export_id.len() > EXPORT_RETRIEVAL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_id = encode_path_segment(export_id);
    let target_path = format!("{NARUON_EXPORT_PATH}/{encoded_id}");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "GET",
        target_url,
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "naruon".into()),
            ("tepp-contract-version".into(), "1".into()),
        ],
        body: String::new(),
    })
}

fn encode_path_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + value.len() / 2);
    let hex = b"0123456789ABCDEF";
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(hex[usize::from(byte >> 4)] as char);
                out.push(hex[usize::from(byte & 0x0F)] as char);
            }
        }
    }
    out
}

fn decode_path_segment(value: &str) -> Result<String, ApiError> {
    let mut out = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(ApiError::InvalidWirePayload);
                }
                let hi = from_hex(bytes[index + 1])?;
                let lo = from_hex(bytes[index + 2])?;
                out.push((hi << 4) | lo);
                index += 3;
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(bytes[index]);
                index += 1;
            }
            _ => return Err(ApiError::InvalidWirePayload),
        }
    }
    let decoded = String::from_utf8(out).map_err(|_| ApiError::InvalidWirePayload)?;
    if decoded.contains('/') || decoded.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(decoded)
}

fn from_hex(byte: u8) -> Result<u8, ApiError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(ApiError::InvalidWirePayload),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE, EXPORT_RETRIEVAL_CONTRACT_VERSION,
        EXPORT_RETRIEVAL_ID_MAX_LEN, ExportRetrieval, export_retrieval_path_id,
        naruon_export_retrieval_exchange, refuse_metrics_on_export_retrieval_payload,
    };
    use crate::ApiError;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn retrieval_round_trips_and_refuses_metrics() {
        let retrieval = ExportRetrieval::new(
            "export-1",
            "artifact-1",
            EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
            "modular_service_consumer",
            "export-idem-1",
        )
        .expect("new");
        assert_eq!(
            retrieval.contract_version,
            EXPORT_RETRIEVAL_CONTRACT_VERSION
        );
        let json = retrieval.to_json().expect("json");
        assert_eq!(ExportRetrieval::from_json(&json).expect("parse"), retrieval);
        assert!(!json.contains("tenant_workspace_id"));
        assert!(!json.contains("principal_id"));
        assert!(!json.contains("includes_source_text"));
        assert!(!json.contains("scientific_acceptance"));
        assert_eq!(refuse_metrics_on_export_retrieval_payload(&json), Ok(()));
        assert_eq!(refuse_metrics_on_export_retrieval_payload(""), Ok(()));
        assert_eq!(
            refuse_metrics_on_export_retrieval_payload(r#"{"nested":["safe"]}"#),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_export_retrieval_payload(r#"{"rmse":1.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_export_retrieval_payload(r#"{"nested":{"rmse":1.0}}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_export_retrieval_payload(r#"{"nested":[{"rmse":1.0}]}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_export_retrieval_payload("[]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ExportRetrieval::new(
                "",
                "a",
                EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                "modular_service_consumer",
                "k"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ExportRetrieval::new("e", "a", "denied", "modular_service_consumer", "k"),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            ExportRetrieval::new(
                "e",
                "a",
                EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                "unknown_purpose",
                "k"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ExportRetrieval::new(
                "e".repeat(EXPORT_RETRIEVAL_ID_MAX_LEN + 1),
                "a",
                EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                "modular_service_consumer",
                "k",
            ),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            ExportRetrieval::new(
                "e",
                "a".repeat(EXPORT_RETRIEVAL_ID_MAX_LEN + 1),
                EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                "modular_service_consumer",
                "k",
            ),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            ExportRetrieval::new(
                "e",
                "a",
                EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                "modular_service_consumer",
                "k".repeat(EXPORT_RETRIEVAL_ID_MAX_LEN + 1),
            ),
            Err(ApiError::LimitExceeded)
        );
        for purpose in [
            "scientific_validation",
            "operational_monitoring",
            "partner_disclosure",
            "modular_service_consumer",
        ] {
            assert!(
                ExportRetrieval::new(
                    "e",
                    "a",
                    EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
                    purpose,
                    "k"
                )
                .is_ok()
            );
        }
    }

    #[test]
    fn path_parser_and_exchange_fail_closed() {
        assert_eq!(
            export_retrieval_path_id("/v1/exports/export-1").expect("id"),
            "export-1"
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports/"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports/a/b"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports/by-idempotency"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports/%"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports/%00"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports/%2f"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports/%GG"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/exports/!"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id("/v1/analysis-runs/export-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_retrieval_path_id(&format!(
                "/v1/exports/{}",
                "a".repeat(EXPORT_RETRIEVAL_ID_MAX_LEN + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
        let exchange =
            naruon_export_retrieval_exchange("https://tepp.example.test", "export-1").expect("ex");
        assert_eq!(exchange.method, "GET");
        assert_eq!(
            exchange.target_url,
            "https://tepp.example.test/v1/exports/export-1"
        );
        assert!(exchange.body.is_empty());
        assert_eq!(
            exchange.headers,
            vec![
                ("content-type".into(), "application/json".into()),
                ("tepp-consumer".into(), "naruon".into()),
                ("tepp-contract-version".into(), "1".into()),
            ]
        );
        assert_eq!(
            naruon_export_retrieval_exchange("http://tepp.example.test", "export-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_export_retrieval_exchange("https://db.postgres.example", "export-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_export_retrieval_exchange("https://tepp.example.test", ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_export_retrieval_exchange(
                "https://tepp.example.test",
                &"a".repeat(EXPORT_RETRIEVAL_ID_MAX_LEN + 1)
            ),
            Err(ApiError::LimitExceeded)
        );
        let encoded =
            naruon_export_retrieval_exchange("https://tepp.example.test", "exp/../x").expect("enc");
        assert!(encoded.target_url.contains("exp%2F..%2Fx"));
        assert_eq!(
            export_retrieval_path_id("/v1/exports/exp%2F..%2Fx"),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
