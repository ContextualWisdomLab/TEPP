//! Provider-owned project-history cancel HTTP contracts.
//!
//! GAP-003A unique slice: `POST /v1/project-histories/{idempotency_key}/cancel`
//! removes one accepted cutoff-safe identity from `AnalysisRunLiveService` /
//! `tepp-loopback`. The receipt stays metric-free with
//! `inference_status=temporal_association_only` and `cancelled=true`.
//! `tepp.scientific_acceptance.v1` never appears. Cancel does not infer
//! causality. This module does not duplicate project-history POST CLI (#420),
//! collection GET (#424), collection CLI (#428), GET-by-id (#429), retrieval
//! CLI (#431), export cancel HTTP (#445), interpretation-run cancel HTTP
//! (#440), analysis-run cancel (#361), Leiden, or GAP-010 Figma/export.
//! Persistence remains GAP-003B. Naruon is refused. `NaruonLiveService` stays
//! POST-only.

use crate::naruon_http::{compose_https_target, NaruonHttpExchange};
use crate::project_history::validate_project_history_registry_identity;
use crate::project_history_collection_http::{
    refuse_metrics_on_project_history_collection_payload,
    PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS,
};
use crate::project_history_retrieval_http::PROJECT_HISTORY_RETRIEVAL_TENANT_HEADER;
use crate::wire::{require_byte_limit, require_nonempty, to_json};
use crate::{
    ApiError, ProjectHistoryProjection, ProjectHistoryRequest, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT,
    PROJECT_HISTORY_PATH,
};
use serde::{Deserialize, Serialize};

/// Maximum opaque idempotency-key length on the cancel path.
pub const PROJECT_HISTORY_CANCEL_ID_MAX_LEN: usize = 256;

/// Metric-free cancelled project-history identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectHistoryCancelled {
    /// Consumer-owned stable project key.
    pub project_key: String,
    /// Exact request idempotency key that minted the stored projection.
    pub idempotency_key: String,
    /// Knowledge cutoff applied to the stored projection.
    pub knowledge_cutoff: String,
    /// Fixed claim boundary: sequence is association, not causation.
    pub inference_status: String,
    /// Always `true` on a successful cancel receipt.
    pub cancelled: bool,
}

impl ProjectHistoryCancelled {
    /// Construct a validated cancelled identity from a stored projection.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities, a causal inference
    /// status, or `cancelled` that would not be true.
    pub fn from_stored(
        request: &ProjectHistoryRequest,
        projection: &ProjectHistoryProjection,
    ) -> Result<Self, ApiError> {
        let cancelled = Self {
            project_key: request.project_key.clone(),
            idempotency_key: request.idempotency_key.clone(),
            knowledge_cutoff: projection.knowledge_cutoff.clone(),
            inference_status: projection.inference_status.clone(),
            cancelled: true,
        };
        cancelled.validate()?;
        Ok(cancelled)
    }

    /// Parse and validate a cancelled identity with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        require_byte_limit(payload, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)?;
        refuse_metrics_on_project_history_collection_payload(payload)?;
        let cancelled: Self = crate::wire::from_json(payload)?;
        cancelled.validate()?;
        Ok(cancelled)
    }

    /// Serialize this cancelled identity after metric refusal.
    ///
    /// # Errors
    ///
    /// Returns a validation or metric-key error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)?;
        refuse_metrics_on_project_history_collection_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_nonempty(&self.project_key)?;
        require_nonempty(&self.idempotency_key)?;
        require_nonempty(&self.knowledge_cutoff)?;
        if self.idempotency_key.contains('/') || self.idempotency_key.contains('\0') {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.idempotency_key.len() > PROJECT_HISTORY_CANCEL_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        if self.inference_status != PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS {
            return Err(ApiError::InvalidWirePayload);
        }
        if !self.cancelled {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

/// Extract the opaque idempotency key from
/// `POST /v1/project-histories/{key}/cancel`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for the collection path, GET-by-id
/// path, extra segments, a hostile encoding, empty identity, slash, or NUL,
/// and [`ApiError::LimitExceeded`] when oversized.
pub fn project_history_cancel_path_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(PROJECT_HISTORY_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = encoded
        .strip_suffix("/cancel")
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let idempotency_key = decode_path_segment(encoded)?;
    require_nonempty(&idempotency_key)?;
    if idempotency_key.contains('/') || idempotency_key.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if idempotency_key.len() > PROJECT_HISTORY_CANCEL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(idempotency_key)
}

/// Build a credential-free `LineageWeave` cancel POST exchange.
///
/// Empty body is admitted. The identity travels in the path; the builder does
/// not send an `idempotency-key` header.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// identity, and [`ApiError::LimitExceeded`] when oversized.
pub fn lineageweave_project_history_cancel_exchange(
    origin: &str,
    tenant_workspace_id: &str,
    idempotency_key: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    validate_project_history_registry_identity(tenant_workspace_id)?;
    validate_project_history_registry_identity(idempotency_key)?;
    if idempotency_key.contains('/') || idempotency_key.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if idempotency_key.len() > PROJECT_HISTORY_CANCEL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_id = encode_path_segment(idempotency_key);
    let target_path = format!("{PROJECT_HISTORY_PATH}/{encoded_id}/cancel");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "POST",
        target_url,
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "lineageweave".into()),
            ("tepp-contract-version".into(), "1".into()),
            (
                PROJECT_HISTORY_RETRIEVAL_TENANT_HEADER.into(),
                tenant_workspace_id.into(),
            ),
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
        lineageweave_project_history_cancel_exchange, project_history_cancel_path_id,
        PROJECT_HISTORY_CANCEL_ID_MAX_LEN,
    };
    use crate::project_history_collection_http::refuse_metrics_on_project_history_collection_payload;
    use crate::{ApiError, PROJECT_HISTORY_PATH};

    #[test]
    fn cancel_exchange_is_metric_free_post_without_credentials() {
        let exchange = lineageweave_project_history_cancel_exchange(
            "https://tepp.example.test",
            "history-tenant",
            "idem-a",
        )
        .expect("exchange");
        assert_eq!(exchange.method, "POST");
        assert!(exchange
            .target_url
            .ends_with("/v1/project-histories/idem-a/cancel"));
        assert!(exchange.body.is_empty());
        assert!(!exchange
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                || name.eq_ignore_ascii_case("idempotency-key")));
        assert!(exchange
            .headers
            .iter()
            .any(|(name, value)| name == "tepp-consumer" && value == "lineageweave"));
        assert_eq!(
            project_history_cancel_path_id("/v1/project-histories/idem-a/cancel").expect("id"),
            "idem-a"
        );
        assert_eq!(PROJECT_HISTORY_PATH, "/v1/project-histories");
        assert_eq!(
            refuse_metrics_on_project_history_collection_payload(""),
            Ok(())
        );
    }

    #[test]
    fn cancel_path_and_payloads_fail_closed() {
        assert_eq!(
            project_history_cancel_path_id("/v1/project-histories"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            project_history_cancel_path_id("/v1/project-histories/idem-a"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            project_history_cancel_path_id("/v1/project-histories/idem-a/extra/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            project_history_cancel_path_id("/v1/exports/idem-a/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            project_history_cancel_path_id(&format!(
                "/v1/project-histories/{}/cancel",
                "e".repeat(PROJECT_HISTORY_CANCEL_ID_MAX_LEN + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            lineageweave_project_history_cancel_exchange(
                "http://tepp.example.test",
                "history-tenant",
                "idem-a"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            lineageweave_project_history_cancel_exchange(
                "https://tepp.example.test",
                "history-tenant",
                "a/b"
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_collection_payload(r#"{"rmse":1.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_collection_payload(r#"{"findings":[]}"#),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
