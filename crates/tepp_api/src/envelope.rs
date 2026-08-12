//! Machine-readable API error envelopes that never echo secrets or source text.

use crate::ApiError;
use crate::wire::{require_nonempty, to_json};
use serde::{Deserialize, Serialize};

/// Stable wire error envelope for service responses.
///
/// Fields are private so callers cannot emit empty codes or unredacted free-form
/// struct literals. Prefer [`ErrorEnvelope::from_api_error`] for typed mapping.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorEnvelope {
    /// Stable machine-readable error code (`snake_case`).
    error_code: String,
    /// Content-redacting human message.
    message: String,
    /// Opaque request correlation identifier.
    request_id: String,
    /// Whether a client may retry the same operation.
    retryable: bool,
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
    /// Fixed payload/edge limits map to non-retryable `limit_exceeded`. Transient
    /// capacity limits should use a separate future error variant if introduced.
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
            ApiError::LimitExceeded => ("limit_exceeded", false),
            ApiError::AuthorizationDenied => ("authorization_denied", false),
        };
        Self::new(error_code, error.to_string(), request_id, retryable)
    }

    /// Stable machine-readable error code.
    #[must_use]
    pub fn error_code(&self) -> &str {
        &self.error_code
    }

    /// Content-redacting human message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Opaque request correlation identifier.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Whether a client may retry the same operation.
    #[must_use]
    pub const fn retryable(&self) -> bool {
        self.retryable
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_nonempty(&self.error_code)?;
        require_nonempty(&self.message)?;
        require_nonempty(&self.request_id)?;
        Ok(())
    }

    /// Serialize to JSON after re-validating required fields.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when fields are empty or
    /// serialization fails.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
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
        assert!(!envelope.retryable());
        assert_eq!(envelope.error_code(), "limit_exceeded");
        assert!(!envelope.message().is_empty());
        assert_eq!(envelope.request_id(), "req-1");
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
        // Deserialized empty fields must fail closed on serialize.
        let forged: ErrorEnvelope = serde_json::from_str(
            r#"{"error_code":"","message":"m","request_id":"r","retryable":false}"#,
        )
        .expect("deser");
        assert_eq!(forged.to_json(), Err(ApiError::InvalidWirePayload));
        for error in [
            ApiError::InvalidWirePayload,
            ApiError::UnsupportedContractVersion,
            ApiError::AuthorizationDenied,
            ApiError::LimitExceeded,
        ] {
            let mapped = ErrorEnvelope::from_api_error(error, "req-x").expect("map");
            assert!(!mapped.retryable());
        }
    }
}
