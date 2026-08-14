//! Versioned HTTPS interchange for the contextual-orchestrator interpretation port.

use crate::ApiError;
use crate::wire::require_nonempty;
use serde_json::Value;

/// Versioned interpretation-run path on the orchestrator origin.
pub const ORCHESTRATOR_INTERPRETATION_PATH: &str = "/v1/interpretation-runs";
/// Maximum UTF-8 bytes accepted in one orchestrator JSON body.
pub const MAX_ORCHESTRATOR_BODY_BYTES: usize = 1_048_576;
/// Maximum UTF-8 bytes accepted in one idempotency-key header value.
pub const MAX_ORCHESTRATOR_IDEMPOTENCY_KEY_BYTES: usize = 256;

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

    /// JSON body; never includes repository-write secret names.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

/// Build a credential-free interpretation request for contextual-orchestrator.
///
/// The origin is a DNS host only. Table-access hosts and non-`https` schemes
/// fail closed. The body must be a bounded JSON object and cannot carry known
/// repository-write or review-agent secret names. The orchestrator does not
/// become scientific authority.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for an empty, unsafe, or oversized
/// idempotency key, malformed/non-object JSON, or an empty body. Returns
/// [`ApiError::LimitExceeded`] for body or idempotency-key size limits. Returns
/// [`ApiError::AuthorizationDenied`] for a hostile host or forbidden secret-name
/// channel.
pub fn orchestrator_interpretation_exchange(
    origin_host: &str,
    idempotency_key: &str,
    body: &str,
) -> Result<OrchestratorHttpExchange, ApiError> {
    require_safe_idempotency_key(idempotency_key)?;
    require_bounded_json_object(body)?;
    require_safe_https_host(origin_host)?;
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
/// [`ApiError::AuthorizationDenied`] for every name except the NVIDIA NIM key.
pub fn refuse_repository_write_secret(secret_name: &str) -> Result<(), ApiError> {
    require_nonempty(secret_name)?;
    let folded = normalize_secret_name(secret_name);
    if folded == "nvidianimapikey" {
        Ok(())
    } else {
        Err(ApiError::AuthorizationDenied)
    }
}

fn require_safe_idempotency_key(value: &str) -> Result<(), ApiError> {
    require_nonempty(value)?;
    if value.len() > MAX_ORCHESTRATOR_IDEMPOTENCY_KEY_BYTES {
        return Err(ApiError::LimitExceeded);
    }
    if value.chars().any(char::is_whitespace) || value.chars().any(char::is_control) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

fn require_bounded_json_object(body: &str) -> Result<(), ApiError> {
    require_nonempty(body)?;
    if body.len() > MAX_ORCHESTRATOR_BODY_BYTES {
        return Err(ApiError::LimitExceeded);
    }
    let value: Value = serde_json::from_str(body).map_err(|_| ApiError::InvalidWirePayload)?;
    if !value.is_object() {
        return Err(ApiError::InvalidWirePayload);
    }

    let mut pending = vec![&value];
    while let Some(current) = pending.pop() {
        match current {
            Value::Object(entries) => {
                for (key, nested) in entries {
                    if is_forbidden_secret_name(key) {
                        return Err(ApiError::AuthorizationDenied);
                    }
                    pending.push(nested);
                }
            }
            Value::Array(entries) => pending.extend(entries),
            Value::String(text) => {
                if is_forbidden_secret_name(text) {
                    return Err(ApiError::AuthorizationDenied);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }
    Ok(())
}

fn normalize_secret_name(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_forbidden_secret_name(value: &str) -> bool {
    let folded = normalize_secret_name(value);
    if folded == "nvidianimapikey" {
        return false;
    }
    folded == "githubtoken"
        || folded == "copilotgithubtoken"
        || folded == "reviewagentgithubtoken"
        || folded == "opencodegithubtoken"
        || (folded.contains("copilot") && folded.contains("token"))
        || (folded.contains("reviewagent") && folded.contains("token"))
        || (folded.starts_with("github") && folded.ends_with("token"))
}

fn require_safe_https_host(host: &str) -> Result<(), ApiError> {
    if host.is_empty()
        || host.len() > 253
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
