//! Versioned HTTPS interchange for the contextual-orchestrator interpretation port.

use crate::ApiError;
use crate::wire::require_nonempty;

/// Versioned interpretation-run path on the orchestrator origin.
pub const ORCHESTRATOR_INTERPRETATION_PATH: &str = "/v1/interpretation-runs";

/// Fail-closed HTTPS POST that TEPP may send to contextual-orchestrator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestratorHttpExchange {
    method: String,
    target_url: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl OrchestratorHttpExchange {
    /// HTTP method; always `POST`.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Absolute `https` target URL.
    #[must_use]
    pub fn target_url(&self) -> &str {
        &self.target_url
    }

    /// Request headers; never includes credentials or review-agent tokens.
    #[must_use]
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }

    /// JSON body; never includes repository-write secrets.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

/// Build a credential-free interpretation request for contextual-orchestrator.
///
/// The origin is a DNS host only. Table-access hosts and non-`https` schemes
/// fail closed. The orchestrator does not become scientific authority.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for empty idempotency or body.
/// Returns [`ApiError::AuthorizationDenied`] for a hostile or non-`https` host.
pub fn orchestrator_interpretation_exchange(
    origin_host: &str,
    idempotency_key: &str,
    body: &str,
) -> Result<OrchestratorHttpExchange, ApiError> {
    require_nonempty(idempotency_key)?;
    require_nonempty(body)?;
    require_safe_https_host(origin_host)?;
    if body.contains("COPILOT_GITHUB_TOKEN") {
        return Err(ApiError::AuthorizationDenied);
    }
    Ok(OrchestratorHttpExchange {
        method: "POST".into(),
        target_url: format!("https://{origin_host}{ORCHESTRATOR_INTERPRETATION_PATH}"),
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "contextual-orchestrator".into()),
            ("tepp-contract-version".into(), "1".into()),
            ("idempotency-key".into(), idempotency_key.into()),
        ],
        body: body.into(),
    })
}

/// Orchestrator output never replaces deterministic/statistical acceptance.
///
/// # Errors
///
/// Always returns [`ApiError::AuthorizationDenied`].
pub fn refuse_orchestrator_as_scientific_acceptance() -> Result<(), ApiError> {
    Err(ApiError::AuthorizationDenied)
}

/// Refuse repository-write or review-agent secret names on this port.
///
/// `NVIDIA_NIM_API_KEY` is the only allowed model-credential name.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for an empty name and
/// [`ApiError::AuthorizationDenied`] for Copilot, GitHub, or review-agent names.
pub fn refuse_repository_write_secret(secret_name: &str) -> Result<(), ApiError> {
    require_nonempty(secret_name)?;
    let folded: String = secret_name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    if folded == "nvidianimapikey" {
        return Ok(());
    }
    if folded.contains("copilot") || folded.contains("github") || folded.contains("reviewagent") {
        return Err(ApiError::AuthorizationDenied);
    }
    Err(ApiError::AuthorizationDenied)
}

fn require_safe_https_host(host: &str) -> Result<(), ApiError> {
    if host.is_empty()
        || host.chars().any(|ch| {
            ch.is_control()
                || ch.is_whitespace()
                || matches!(ch, '@' | '/' | '?' | '#' | '\'' | ';' | '\\')
        })
    {
        return Err(ApiError::AuthorizationDenied);
    }
    let lowered = host.to_ascii_lowercase();
    if ["postgres", "jdbc", "sql", "tables"]
        .iter()
        .any(|needle| lowered.contains(needle))
    {
        return Err(ApiError::AuthorizationDenied);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        OrchestratorHttpExchange, refuse_repository_write_secret, require_safe_https_host,
    };
    use crate::ApiError;

    #[test]
    fn unknown_secret_names_and_accessors_are_covered() {
        assert_eq!(
            refuse_repository_write_secret("AWS_SECRET"),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            refuse_repository_write_secret(""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(require_safe_https_host("ok.example"), Ok(()));
        let exchange = OrchestratorHttpExchange {
            method: "POST".into(),
            target_url: "https://ok.example/v1/interpretation-runs".into(),
            headers: Vec::new(),
            body: "{}".into(),
        };
        assert_eq!(exchange.method(), "POST");
        assert!(exchange.headers().is_empty());
    }
}
