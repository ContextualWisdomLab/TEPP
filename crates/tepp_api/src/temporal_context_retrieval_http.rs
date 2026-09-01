//! Provider-owned temporal-context GET-by-id contracts.
//!
//! GAP-003A unique slice: `GET /v1/temporal-context/{idempotency_key}` returns
//! one accepted metric-free `LineageWeave` identity on
//! `AnalysisRunLiveService` / `tepp-loopback` so operators who hold a stored
//! key do not replay POST. `tepp.scientific_acceptance.v1` never appears. Event
//! labels, actor lists, and timeline events stay off the retrieval. The
//! retrieval does not infer causality. This module does not re-open collection
//! GET (#449 closed), collection CLI (#450 closed), temporal-context CLI
//! (#414), project-history GET-by-id (#429), interpretation-run GET-by-id
//! (#438), export retrieval GET (#411), cancel lineages, or GAP-010
//! Figma/export. Persistence remains GAP-003B. `NaruonLiveService` stays
//! POST-only.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{ApiError, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, TEMPORAL_CONTEXT_PATH};
use serde::{Deserialize, Serialize};

/// Supported temporal-context retrieval contract version.
pub const TEMPORAL_CONTEXT_RETRIEVAL_CONTRACT_VERSION: u16 = 1;

/// Maximum opaque idempotency-key length on the retrieval path.
pub const TEMPORAL_CONTEXT_RETRIEVAL_ID_MAX_LEN: usize = 128;

/// Fixed non-causal claim boundary echoed on every retrieval.
pub const TEMPORAL_CONTEXT_RETRIEVAL_INFERENCE_STATUS: &str = "temporal_association_only";

const FORBIDDEN_RETRIEVAL_KEYS: [&str; 16] = [
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
    "evidence_text",
    "findings",
    "causal_score",
];

/// One metric-free identity projection for an accepted temporal-context POST.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalContextRetrieved {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Exact request idempotency key that minted the stored identity.
    pub idempotency_key: String,
    /// Knowledge cutoff applied to the stored identity.
    pub knowledge_cutoff: String,
    /// Fixed claim boundary: sequence is association, not causation.
    pub inference_status: String,
}

impl TemporalContextRetrieved {
    /// Construct a validated metric-free retrieval identity.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, slash/NUL, an
    /// oversized key, or a causal inference status.
    pub fn new(
        idempotency_key: impl Into<String>,
        knowledge_cutoff: impl Into<String>,
        inference_status: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let retrieved = Self {
            contract_version: TEMPORAL_CONTEXT_RETRIEVAL_CONTRACT_VERSION,
            idempotency_key: idempotency_key.into(),
            knowledge_cutoff: knowledge_cutoff.into(),
            inference_status: inference_status.into(),
        };
        retrieved.validate()?;
        Ok(retrieved)
    }

    /// Parse and validate a retrieval payload with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)
    }

    /// Parse and validate a retrieval payload with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_temporal_context_retrieval_payload(payload)?;
        let retrieved: Self = from_json(payload)?;
        retrieved.validate()?;
        Ok(retrieved)
    }

    /// Serialize this retrieval after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)?;
        refuse_metrics_on_temporal_context_retrieval_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            TEMPORAL_CONTEXT_RETRIEVAL_CONTRACT_VERSION,
        )?;
        validate_temporal_context_registry_identity(&self.idempotency_key)?;
        require_nonempty(&self.knowledge_cutoff)?;
        if self.inference_status != TEMPORAL_CONTEXT_RETRIEVAL_INFERENCE_STATUS {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

/// Refuse an empty, oversized, slash, NUL, or control-bearing identity.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`].
pub fn validate_temporal_context_registry_identity(identity: &str) -> Result<(), ApiError> {
    require_nonempty(identity)?;
    if identity.contains('/') || identity.contains('\0') || identity.chars().any(char::is_control)
    {
        return Err(ApiError::InvalidWirePayload);
    }
    if identity.len() > TEMPORAL_CONTEXT_RETRIEVAL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(())
}

/// Extract the opaque idempotency key from `GET /v1/temporal-context/{key}`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for the collection path, extra
/// segments, a hostile encoding, or an empty identity, and
/// [`ApiError::LimitExceeded`] when the decoded identity exceeds
/// [`TEMPORAL_CONTEXT_RETRIEVAL_ID_MAX_LEN`].
pub fn temporal_context_retrieval_path_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(TEMPORAL_CONTEXT_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let idempotency_key = decode_path_segment(encoded)?;
    validate_temporal_context_registry_identity(&idempotency_key)?;
    Ok(idempotency_key)
}

/// Refuse retrieval JSON that already carries scientific-metric or evidence keys.
///
/// Empty payloads are admitted for the GET request body.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric, evidence,
/// event-label, actor, or causal-score key is present.
pub fn refuse_metrics_on_temporal_context_retrieval_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    if payload.contains("tepp.scientific_acceptance.v1")
        || payload.contains("event_label")
        || payload.contains("actor_references")
        || payload.contains("timeline_events")
    {
        return Err(ApiError::InvalidWirePayload);
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    if !value.is_object() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_metrics_on_json(&value)
}

fn refuse_metrics_on_json(value: &serde_json::Value) -> Result<(), ApiError> {
    match value {
        serde_json::Value::Object(object) => {
            if FORBIDDEN_RETRIEVAL_KEYS
                .iter()
                .any(|key| object.contains_key(*key))
            {
                return Err(ApiError::InvalidWirePayload);
            }
            for nested in object.values() {
                refuse_metrics_on_json(nested)?;
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            for nested in items {
                refuse_metrics_on_json(nested)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Build a provider-owned `GET` temporal-context retrieval exchange.
///
/// The builder refuses non-`https` origins and empty or oversized identities.
/// It does not inject credentials. The GET body is empty. The identity
/// travels in the path.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// identity, and [`ApiError::LimitExceeded`] when the identity exceeds
/// [`TEMPORAL_CONTEXT_RETRIEVAL_ID_MAX_LEN`] bytes.
pub fn lineageweave_temporal_context_retrieval_exchange(
    origin: &str,
    idempotency_key: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    validate_temporal_context_registry_identity(idempotency_key)?;
    let encoded_id = encode_path_segment(idempotency_key);
    let target_path = format!("{TEMPORAL_CONTEXT_PATH}/{encoded_id}");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "GET",
        target_url,
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "lineageweave".into()),
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
    if decoded.chars().any(char::is_control) {
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
        TEMPORAL_CONTEXT_RETRIEVAL_ID_MAX_LEN, TEMPORAL_CONTEXT_RETRIEVAL_INFERENCE_STATUS,
        TemporalContextRetrieved, lineageweave_temporal_context_retrieval_exchange,
        refuse_metrics_on_temporal_context_retrieval_payload, temporal_context_retrieval_path_id,
    };
    use crate::ApiError;

    #[test]
    fn retrieval_round_trips_and_refuses_hostile_shapes() {
        let retrieved = TemporalContextRetrieved::new(
            "idem-a",
            "2026-08-20T00:00:00Z",
            TEMPORAL_CONTEXT_RETRIEVAL_INFERENCE_STATUS,
        )
        .expect("row");
        let json = retrieved.to_json().expect("json");
        assert_eq!(
            TemporalContextRetrieved::from_json(&json).expect("decode"),
            retrieved
        );
        assert!(!json.contains("rmse"));
        assert!(!json.contains("event_label"));
        assert!(!json.contains("actor_references"));
        assert_eq!(
            TemporalContextRetrieved::new(
                "a/b",
                "2026-08-20T00:00:00Z",
                TEMPORAL_CONTEXT_RETRIEVAL_INFERENCE_STATUS
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            TemporalContextRetrieved::new(
                "a".repeat(TEMPORAL_CONTEXT_RETRIEVAL_ID_MAX_LEN + 1),
                "2026-08-20T00:00:00Z",
                TEMPORAL_CONTEXT_RETRIEVAL_INFERENCE_STATUS
            ),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            temporal_context_retrieval_path_id("/v1/temporal-context"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            temporal_context_retrieval_path_id("/v1/temporal-context/idem-a/extra"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            temporal_context_retrieval_path_id("/v1/temporal-context/idem-a").expect("id"),
            "idem-a"
        );
        assert_eq!(
            refuse_metrics_on_temporal_context_retrieval_payload(r#"{"rmse":1}"#),
            Err(ApiError::InvalidWirePayload)
        );
        let exchange = lineageweave_temporal_context_retrieval_exchange(
            "https://tepp.example.test",
            "idem-a",
        )
        .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(exchange.target_url.ends_with("/v1/temporal-context/idem-a"));
        assert!(exchange.body.is_empty());
        assert_eq!(
            lineageweave_temporal_context_retrieval_exchange("http://insecure.example", "idem-a"),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
