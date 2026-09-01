//! Provider-owned project-history GET-by-id contracts.
//!
//! GAP-003A unique slice: `GET /v1/project-histories/{idempotency_key}`
//! returns one accepted cutoff-safe `ProjectHistoryProjection` on
//! `AnalysisRunLiveService` / `tepp-loopback` so operators who hold a
//! collection identity do not replay POST. `tepp.scientific_acceptance.v1`
//! never appears. The retrieval does not infer causality. This module does
//! not duplicate collection GET (#424), collection CLI (#428), project-history
//! POST CLI (#420), temporal-context CLI (#414), export retrieval GET (#411),
//! analysis-run GET-by-id (#359), or GAP-010 Figma/export. Persistence remains
//! GAP-003B. `NaruonLiveService` stays POST-only.

use crate::naruon_http::{NaruonHttpExchange, compose_https_target};
use crate::project_history::validate_project_history_registry_identity;
use crate::wire::require_nonempty;
use crate::{ApiError, PROJECT_HISTORY_PATH};

/// Maximum opaque idempotency-key length on the retrieval path.
pub const PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN: usize = 256;

/// Header carrying the authorized project-history tenant on GET-by-id.
pub const PROJECT_HISTORY_RETRIEVAL_TENANT_HEADER: &str = "tepp-tenant-workspace-id";

const FORBIDDEN_RETRIEVAL_KEYS: [&str; 12] = [
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
    "causal_score",
];

/// Extract the opaque idempotency key from `GET /v1/project-histories/{key}`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for the collection path, extra
/// segments, a hostile encoding, or an empty identity, and
/// [`ApiError::LimitExceeded`] when the decoded identity exceeds
/// [`PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN`].
pub fn project_history_retrieval_path_id(path: &str) -> Result<String, ApiError> {
    let remainder = path
        .strip_prefix(PROJECT_HISTORY_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    let idempotency_key = decode_path_segment(encoded)?;
    require_nonempty(&idempotency_key)?;
    if idempotency_key.len() > PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(idempotency_key)
}

/// Refuse retrieval JSON that already carries scientific-metric or causal keys.
///
/// Empty payloads are admitted for the GET request body. Evidence text and
/// findings belong to the stored projection and are not refused here.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric or causal
/// key is present, `tepp.scientific_acceptance.v1` appears, or nonempty JSON
/// is not an object.
pub fn refuse_metrics_on_project_history_retrieval_payload(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    if payload.contains("tepp.scientific_acceptance.v1") {
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

/// Build a provider-owned `GET` project-history retrieval exchange.
///
/// The builder refuses non-`https` origins and empty or oversized identities.
/// It does not inject credentials. The GET body is empty. The identity
/// travels in the path.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin or empty
/// identity, and [`ApiError::LimitExceeded`] when the identity exceeds
/// [`PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN`] bytes.
pub fn lineageweave_project_history_retrieval_exchange(
    origin: &str,
    tenant_workspace_id: &str,
    idempotency_key: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    validate_project_history_registry_identity(tenant_workspace_id)?;
    validate_project_history_registry_identity(idempotency_key)?;
    let encoded_id = encode_path_segment(idempotency_key);
    let target_path = format!("{PROJECT_HISTORY_PATH}/{encoded_id}");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "GET",
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
        PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN, lineageweave_project_history_retrieval_exchange,
        project_history_retrieval_path_id, refuse_metrics_on_project_history_retrieval_payload,
    };
    use crate::ApiError;

    #[test]
    fn retrieval_exchange_is_metric_free_get_without_credentials() {
        let exchange = lineageweave_project_history_retrieval_exchange(
            "https://tepp.example.test",
            "tenant-a",
            "idem-a",
        )
        .expect("exchange");
        assert_eq!(exchange.method, "GET");
        assert!(
            exchange
                .target_url
                .ends_with("/v1/project-histories/idem-a")
        );
        assert!(exchange.body.is_empty());
        assert!(
            !exchange
                .headers
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        );
        assert_eq!(
            project_history_retrieval_path_id("/v1/project-histories/idem-a").expect("id"),
            "idem-a"
        );
        let encoded = lineageweave_project_history_retrieval_exchange(
            "https://tepp.example.test",
            "tenant-a",
            "idem/slash",
        )
        .expect("encoded");
        assert!(encoded.target_url.contains("idem%2Fslash"));
        assert_eq!(
            project_history_retrieval_path_id("/v1/project-histories/idem%2Fslash")
                .expect("decoded slash"),
            "idem/slash"
        );
    }

    #[test]
    fn retrieval_path_and_payload_fail_closed() {
        assert_eq!(
            project_history_retrieval_path_id("/v1/project-histories"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            project_history_retrieval_path_id("/v1/project-histories/"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            project_history_retrieval_path_id("/v1/project-histories/idem-a/extra"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            project_history_retrieval_path_id("/v1/analysis-runs/idem-a"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            project_history_retrieval_path_id(&format!(
                "/v1/project-histories/{}",
                "a".repeat(PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN + 1)
            )),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            lineageweave_project_history_retrieval_exchange(
                "http://insecure.example",
                "tenant-a",
                "idem-a",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            lineageweave_project_history_retrieval_exchange(
                "https://tepp.example.test",
                "tenant-a",
                "",
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_retrieval_payload(""),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_project_history_retrieval_payload(r#"{"findings":[]}"#),
            Ok(())
        );
        assert_eq!(
            refuse_metrics_on_project_history_retrieval_payload(r#"{"rmse":1.0}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_retrieval_payload(r#"{"causal_score":1}"#),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_retrieval_payload(
                r#"{"schema_version":"tepp.scientific_acceptance.v1"}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_retrieval_payload("[]"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_metrics_on_project_history_retrieval_payload(r#"{"nested":[1]}"#),
            Ok(())
        );
        assert_eq!(
            project_history_retrieval_path_id("/v1/project-histories/%zz"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            lineageweave_project_history_retrieval_exchange(
                "https://tepp.example.test",
                "\ntenant",
                "idem-a",
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn retrieval_path_decoding_covers_hostile_and_encoded_bytes() {
        for path in [
            "/v1/project-histories/%",
            "/v1/project-histories/idem!",
            "/v1/project-histories/%00",
            "/v1/project-histories/%FF",
        ] {
            assert_eq!(
                project_history_retrieval_path_id(path),
                Err(ApiError::InvalidWirePayload)
            );
        }
        assert_eq!(
            project_history_retrieval_path_id("/v1/project-histories/idem%2fslash")
                .expect("lowercase hex"),
            "idem/slash"
        );
    }
}
