//! Versioned HTTP interchange naruon may emit toward TEPP (ADR 0011).

use crate::authorization::{
    AnalyticalPurpose, ExportAuthorizationRequest, authorize_export, require_export_allowed,
};
use crate::wire::require_nonempty;
use crate::{AnalysisRunRequest, ApiError};

/// Versioned analysis-run create path naruon may call.
pub const NARUON_ANALYSIS_RUN_PATH: &str = "/v1/analysis-runs";

/// Versioned export-authorization path naruon may call.
pub const NARUON_EXPORT_PATH: &str = "/v1/exports";

/// Allowed naruon claim that a result came from TEPP inference.
pub const NARUON_TEPP_INFERENCE_METHOD: &str = "tepp_topic_measurement";

/// One fail-closed HTTP exchange naruon may send to a TEPP origin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NaruonHttpExchange {
    /// HTTP method (`POST` for both create and export-authorize).
    pub method: &'static str,
    /// Absolute `https` URL on the versioned TEPP path.
    pub target_url: String,
    /// Ordered header pairs; never includes review or Copilot credentials.
    pub headers: Vec<(String, String)>,
    /// JSON body already validated by the owning DTO.
    pub body: String,
}

/// Build a naruon → TEPP analysis-run create exchange.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-`https` origin, a
/// table-access URL, or an invalid analysis-run body.
pub fn naruon_analysis_run_exchange(
    origin: &str,
    request: &AnalysisRunRequest,
) -> Result<NaruonHttpExchange, ApiError> {
    naruon_analysis_run_exchange_with_headers(origin, request, &[])
}

/// Build an analysis-run exchange and refuse credential-bearing extra headers.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] when an extra header names a
/// review, Copilot, or bearer credential. Other failures match
/// [`naruon_analysis_run_exchange`].
pub fn naruon_analysis_run_exchange_with_headers(
    origin: &str,
    request: &AnalysisRunRequest,
    extra_headers: &[(&str, &str)],
) -> Result<NaruonHttpExchange, ApiError> {
    let target_url = compose_https_target(origin, NARUON_ANALYSIS_RUN_PATH)?;
    refuse_credential_headers(extra_headers)?;
    let body = request.to_json()?;
    let mut headers = standard_headers(&request.idempotency_key);
    for (name, value) in extra_headers {
        headers.push(((*name).to_owned(), (*value).to_owned()));
    }
    Ok(NaruonHttpExchange {
        method: "POST",
        target_url,
        headers,
        body,
    })
}

/// Build a naruon → TEPP export-authorization exchange.
///
/// Only [`AnalyticalPurpose::ModularServiceConsumer`] is accepted. TEPP remains
/// the purpose-bound disclosure authority. The `idempotency-key` header is the
/// caller-supplied per-export operation key and is never derived from
/// `principal_id` alone.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] when the purpose is not modular
/// consumption or [`authorize_export`] denies the request.
/// Returns [`ApiError::InvalidWirePayload`] for a hostile origin or empty
/// idempotency key.
pub fn naruon_export_exchange(
    origin: &str,
    request: &ExportAuthorizationRequest,
    idempotency_key: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    if request.purpose != AnalyticalPurpose::ModularServiceConsumer {
        return Err(ApiError::AuthorizationDenied);
    }
    require_nonempty(idempotency_key)?;
    require_export_allowed(&authorize_export(request)?)?;
    let target_url = compose_https_target(origin, NARUON_EXPORT_PATH)?;
    let body = crate::wire::to_json(request)?;
    Ok(NaruonHttpExchange {
        method: "POST",
        target_url,
        headers: standard_headers(idempotency_key),
        body,
    })
}

/// Refuse lexical or empty method codes as TEPP inference claims.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] unless the method is exactly
/// [`NARUON_TEPP_INFERENCE_METHOD`].
pub fn naruon_may_claim_tepp_inference(method_code: &str) -> Result<(), ApiError> {
    require_nonempty(method_code)?;
    if method_code == NARUON_TEPP_INFERENCE_METHOD {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn compose_https_target(origin: &str, path: &str) -> Result<String, ApiError> {
    require_nonempty(origin)?;
    if !origin.starts_with("https://") {
        return Err(ApiError::InvalidWirePayload);
    }
    let host = &origin["https://".len()..];
    if host.is_empty()
        || host.starts_with('/')
        || host.contains('@')
        || host.contains('/')
        || host.contains('?')
        || host.contains('#')
        || host.chars().any(|ch| matches!(ch, '\'' | ';' | '\\' | ' '))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    let lowered = origin.to_ascii_lowercase();
    // Host-only origins cannot embed path segments; refuse DB-access host labels.
    if lowered.contains("postgres") || lowered.contains("jdbc") {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(format!("{origin}{path}"))
}

/// Return whether `name` is a reserved naruon interchange header.
pub(crate) fn header_is_reserved_standard(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "content-type" | "tepp-consumer" | "tepp-contract-version" | "idempotency-key"
    )
}

/// Return whether `name` is a review, model, proxy, or bearer credential header.
pub(crate) fn header_is_credential(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    lowered == "authorization"
        || lowered == "proxy-authorization"
        || lowered == "cookie"
        || lowered == "x-api-key"
        || lowered.contains("api-key")
        || lowered.contains("api_key")
        || lowered.contains("apikey")
        || lowered.contains("token")
        || lowered.contains("copilot")
        || lowered.contains("github")
        || lowered.contains("nim")
        || lowered.contains("nvidia")
}

fn refuse_credential_headers(extra_headers: &[(&str, &str)]) -> Result<(), ApiError> {
    for (name, _) in extra_headers {
        if header_is_reserved_standard(name) {
            return Err(ApiError::InvalidWirePayload);
        }
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
    }
    Ok(())
}

fn standard_headers(idempotency_key: &str) -> Vec<(String, String)> {
    vec![
        ("content-type".into(), "application/json".into()),
        ("tepp-consumer".into(), "naruon".into()),
        ("tepp-contract-version".into(), "1".into()),
        ("idempotency-key".into(), idempotency_key.to_owned()),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        NARUON_TEPP_INFERENCE_METHOD, compose_https_target, naruon_export_exchange,
        naruon_may_claim_tepp_inference, refuse_credential_headers,
    };
    use crate::{AnalyticalPurpose, ApiError, ExportAuthorizationRequest};

    #[test]
    fn compose_https_target_accepts_clean_origin_and_rejects_hostile_forms() {
        assert!(compose_https_target("https://tepp.example.test", "/v1/x").is_ok());
        assert_eq!(
            compose_https_target("", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("http://insecure.example", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https:///leading", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://user@host", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://host/path", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://host?q=1", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://host#frag", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://ho st", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://ho\\st", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://ho'st", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://ho;st", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://ho\u{0001}st", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://db.postgres.example", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://jdbc.example", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://api.example/sql", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            compose_https_target("https://api.example/tables/x", "/v1/x"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn refuse_credential_headers_covers_reserved_and_secret_names() {
        assert!(refuse_credential_headers(&[("x-trace", "1")]).is_ok());
        assert!(refuse_credential_headers(&[]).is_ok());
        assert_eq!(
            refuse_credential_headers(&[("Content-Type", "text/plain")]),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_credential_headers(&[("IDEMPOTENCY-KEY", "override")]),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_credential_headers(&[("tepp-consumer", "hostile")]),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_credential_headers(&[("tepp-contract-version", "0")]),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            refuse_credential_headers(&[("Authorization", "Bearer x")]),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            refuse_credential_headers(&[("cookie", "a=b")]),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            refuse_credential_headers(&[("x-api-key", "k")]),
            Err(ApiError::AuthorizationDenied)
        );
        for name in ["x-apikey", "x-api_key", "X-ApiKey"] {
            assert_eq!(
                refuse_credential_headers(&[(name, "k")]),
                Err(ApiError::AuthorizationDenied),
                "header={name}"
            );
        }
        assert_eq!(
            refuse_credential_headers(&[("x-github-token", "t")]),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            refuse_credential_headers(&[("x-github-actor", "agent")]),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            refuse_credential_headers(&[("x-copilot-session", "t")]),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            refuse_credential_headers(&[("Proxy-Authorization", "Basic x")]),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            refuse_credential_headers(&[("x-nvidia-nim-key", "nvapi-x")]),
            Err(ApiError::AuthorizationDenied)
        );
    }

    #[test]
    fn naruon_export_exchange_covers_unit_test_purpose_gate() {
        let allowed = ExportAuthorizationRequest {
            tenant_workspace_id: "tenant-a".into(),
            principal_id: "naruon-service".into(),
            purpose: AnalyticalPurpose::ModularServiceConsumer,
            artifact_id: "artifact-a".into(),
            includes_source_text: false,
        };
        assert!(
            naruon_export_exchange("https://tepp.example.test", &allowed, "export-idem-a").is_ok()
        );

        let denied = ExportAuthorizationRequest {
            purpose: AnalyticalPurpose::OperationalMonitoring,
            ..allowed
        };
        assert_eq!(
            naruon_export_exchange("https://tepp.example.test", &denied, "export-idem-b"),
            Err(ApiError::AuthorizationDenied)
        );
    }

    #[test]
    fn naruon_may_claim_tepp_inference_covers_accept_and_reject_arms() {
        assert!(naruon_may_claim_tepp_inference(NARUON_TEPP_INFERENCE_METHOD).is_ok());
        assert_eq!(
            naruon_may_claim_tepp_inference(""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            naruon_may_claim_tepp_inference("other_method"),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
