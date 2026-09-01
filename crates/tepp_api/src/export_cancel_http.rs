//! Provider-owned export cancel HTTP contracts.
//!
//! GAP-003A unique slice: `POST /v1/exports/{export_id}/cancel` removes one
//! authorized metric-free identity from `AnalysisRunLiveService` /
//! `tepp-loopback`. The receipt stays metric-free with `cancelled=true`.
//! `tepp.scientific_acceptance.v1` never appears. Cancel does not infer
//! causality. This module does not duplicate analysis-run cancel (#361),
//! interpretation-run cancel HTTP (#440), interpretation-run cancel CLI
//! (#442), export collection GET (#443), export collection CLI (#444),
//! export-retrieval CLI (#417), export retrieval GET (#411), export-authorize
//! CLI (#410), Leiden, or GAP-010 Figma/export. Persistence remains GAP-003B.
//! `LineageWeave` is refused. `NaruonLiveService` stays POST-only.

use crate::naruon_http::{compose_https_target, NaruonHttpExchange, NARUON_EXPORT_PATH};
use crate::wire::{require_byte_limit, require_nonempty, to_json};
use crate::{
    refuse_metrics_on_export_retrieval_payload, ApiError, ExportRetrieval,
    DEFAULT_ANALYSIS_RUN_BYTE_LIMIT, EXPORT_RETRIEVAL_ID_MAX_LEN,
};
use serde::{Deserialize, Serialize};

/// Maximum opaque export identity length on the cancel path.
pub const EXPORT_CANCEL_ID_MAX_LEN: usize = EXPORT_RETRIEVAL_ID_MAX_LEN;

/// Metric-free cancelled export identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportCancelled {
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
    /// Always `true` on a successful cancel receipt.
    pub cancelled: bool,
}

impl ExportCancelled {
    /// Construct a validated cancelled identity from a stored retrieval.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error when the retrieval is invalid or
    /// `cancelled` would not be true.
    pub fn from_retrieval(retrieval: ExportRetrieval) -> Result<Self, ApiError> {
        let cancelled = Self {
            export_id: retrieval.export_id,
            artifact_id: retrieval.artifact_id,
            decision_code: retrieval.decision_code,
            purpose: retrieval.purpose,
            idempotency_key: retrieval.idempotency_key,
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
        require_byte_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_export_retrieval_payload(payload)?;
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
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        refuse_metrics_on_export_retrieval_payload(&payload)?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_nonempty(&self.export_id)?;
        require_nonempty(&self.artifact_id)?;
        require_nonempty(&self.decision_code)?;
        require_nonempty(&self.purpose)?;
        require_nonempty(&self.idempotency_key)?;
        if self.export_id.len() > EXPORT_CANCEL_ID_MAX_LEN
            || self.artifact_id.len() > EXPORT_CANCEL_ID_MAX_LEN
            || self.idempotency_key.len() > EXPORT_CANCEL_ID_MAX_LEN
        {
            return Err(ApiError::LimitExceeded);
        }
        if self.export_id.contains('/')
            || self.export_id.contains('\0')
            || self.idempotency_key.contains('/')
            || self.idempotency_key.contains('\0')
        {
            return Err(ApiError::InvalidWirePayload);
        }
        if !self.cancelled {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

/// Extract the opaque export identity from `POST /v1/exports/{export_id}/cancel`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for the collection path, GET-by-id
/// path, extra segments, a hostile encoding, empty identity, slash, or NUL,
/// and [`ApiError::LimitExceeded`] when oversized.
pub fn export_cancel_path_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(NARUON_EXPORT_PATH)
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
    let export_id = decode_path_segment(encoded)?;
    require_nonempty(&export_id)?;
    if export_id.contains('/') || export_id.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if export_id.len() > EXPORT_CANCEL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(export_id)
}

/// Build a credential-free naruon cancel POST exchange.
///
/// Empty body is admitted. The identity travels in the path; the builder does
/// not send an `idempotency-key` header.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// identity, and [`ApiError::LimitExceeded`] when oversized.
pub fn naruon_export_cancel_exchange(
    origin: &str,
    export_id: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    require_nonempty(export_id)?;
    if export_id.contains('/') || export_id.contains('\0') {
        return Err(ApiError::InvalidWirePayload);
    }
    if export_id.len() > EXPORT_CANCEL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_id = encode_path_segment(export_id);
    let target_path = format!("{NARUON_EXPORT_PATH}/{encoded_id}/cancel");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "POST",
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
        export_cancel_path_id, naruon_export_cancel_exchange, ExportCancelled, EXPORT_CANCEL_ID_MAX_LEN,
    };
    use crate::export_http::EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE;
    use crate::naruon_http::NARUON_EXPORT_PATH;
    use crate::{
        refuse_metrics_on_export_retrieval_payload, ApiError, ExportRetrieval,
    };

    #[test]
    fn cancel_exchange_is_metric_free_post_without_credentials() {
        let exchange =
            naruon_export_cancel_exchange("https://tepp.example.test", "export-1").expect("exchange");
        assert_eq!(exchange.method, "POST");
        assert!(exchange
            .target_url
            .ends_with("/v1/exports/export-1/cancel"));
        assert!(exchange.body.is_empty());
        assert!(!exchange
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                || name.eq_ignore_ascii_case("idempotency-key")));
        assert_eq!(
            export_cancel_path_id("/v1/exports/export-1/cancel").expect("id"),
            "export-1"
        );
        assert!(!is_collection_or_get_by_id("/v1/exports/export-1/cancel"));
        let retrieval = ExportRetrieval::new(
            "export-1",
            "artifact-1",
            EXPORT_RETRIEVAL_ALLOWED_DECISION_CODE,
            "modular_service_consumer",
            "export-idem-1",
        )
        .expect("retrieval");
        let json = ExportCancelled::from_retrieval(retrieval)
            .expect("cancelled")
            .to_json()
            .expect("json");
        assert!(json.contains("\"cancelled\":true"));
        assert!(!json.contains("tenant_workspace_id"));
        assert!(!json.contains("rmse"));
        assert_eq!(refuse_metrics_on_export_retrieval_payload(&json), Ok(()));
        assert_eq!(NARUON_EXPORT_PATH, "/v1/exports");
    }

    #[test]
    fn cancel_path_and_payloads_fail_closed() {
        assert_eq!(
            export_cancel_path_id("/v1/exports"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_cancel_path_id("/v1/exports/export-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_cancel_path_id("/v1/exports/export-1/extra/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_cancel_path_id("/v1/analysis-runs/export-1/cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            export_cancel_path_id(&format!(
                "/v1/exports/{}/cancel",
                "e".repeat(EXPORT_CANCEL_ID_MAX_LEN + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            naruon_export_cancel_exchange("http://tepp.example.test", "export-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_export_cancel_exchange("https://tepp.example.test", "a/b"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_export_cancel_exchange("https://tepp.example.test", "a\0b"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(refuse_metrics_on_export_retrieval_payload(""), Ok(()));
        assert_eq!(
            refuse_metrics_on_export_retrieval_payload(r#"{"rmse":1.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
    }

    fn is_collection_or_get_by_id(path: &str) -> bool {
        path == NARUON_EXPORT_PATH || !path.ends_with("/cancel")
    }
}
