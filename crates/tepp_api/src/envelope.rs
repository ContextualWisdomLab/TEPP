//! Machine-readable API error envelopes that never echo secrets or source text.

use crate::ApiError;
use crate::wire::{require_nonempty, to_json};
use serde::{Deserialize, Serialize};

/// Stable wire error envelope for service responses.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorEnvelope {
    /// Stable machine-readable error code (`snake_case`).
    pub error_code: String,
    /// Content-redacting human message.
    pub message: String,
    /// Opaque request correlation identifier.
    pub request_id: String,
    /// Whether a client may retry the same operation.
    pub retryable: bool,
}

impl ErrorEnvelope {
    /// Construct a validated envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when required strings are empty.
    pub fn new(
        error_code: impl Into<String>,
        message: impl Into<String>,
        request_id: impl Into<String>,
        retryable: bool,
    ) -> Result<Self, ApiError> {
        let error_code = error_code.into();
        let message = message.into();
        let request_id = request_id.into();
        require_nonempty(&error_code)?;
        require_nonempty(&message)?;
        require_nonempty(&request_id)?;
        Ok(Self {
            error_code,
            message,
            request_id,
            retryable,
        })
    }

    /// Build an envelope from a typed API error without leaking internals.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when `request_id` is empty.
    pub fn from_api_error(
        error: ApiError,
        request_id: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let (error_code, retryable) = match error {
            ApiError::InvalidWirePayload => ("invalid_wire_payload", false),
            ApiError::UnsupportedContractVersion => ("unsupported_contract_version", false),
            ApiError::LimitExceeded => ("limit_exceeded", true),
            ApiError::AuthorizationDenied => ("authorization_denied", false),
        };
        Self::new(error_code, error.to_string(), request_id, retryable)
    }

    /// Serialize to JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when serialization fails.
    pub fn to_json(&self) -> Result<String, ApiError> {
        to_json(self)
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorEnvelope;
    use crate::ApiError;

    #[test]
    fn envelope_round_trip_and_redaction() {
        let envelope =
            ErrorEnvelope::from_api_error(ApiError::LimitExceeded, "req-1").expect("env");
        assert!(envelope.retryable);
        assert_eq!(envelope.error_code, "limit_exceeded");
        let json = envelope.to_json().expect("json");
        assert!(json.contains("limit_exceeded"));
        assert!(!json.contains("secret"));
        assert_eq!(
            ErrorEnvelope::new("", "m", "r", false),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ErrorEnvelope::new("c", "", "r", false),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ErrorEnvelope::new("c", "m", "", false),
            Err(ApiError::InvalidWirePayload)
        );
        for error in [
            ApiError::InvalidWirePayload,
            ApiError::UnsupportedContractVersion,
            ApiError::AuthorizationDenied,
        ] {
            let mapped = ErrorEnvelope::from_api_error(error, "req-x").expect("map");
            assert!(!mapped.retryable);
        }
    }
}
