//! Strict JSON serialization helpers for versioned API contracts.

use crate::ApiError;
use serde::{Deserialize, Serialize};

/// Serialize a wire DTO to canonical JSON.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when serialization fails.
pub fn to_json<T: Serialize>(value: &T) -> Result<String, ApiError> {
    serde_json::to_string(value).map_err(|_| ApiError::InvalidWirePayload)
}

/// Deserialize a strict wire DTO from JSON text.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for malformed JSON or unknown fields.
pub fn from_json<'de, T: Deserialize<'de>>(payload: &'de str) -> Result<T, ApiError> {
    serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)
}

/// Reject empty, whitespace-only, or control-containing identity/key strings.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when the value is empty after trim
/// or contains a Unicode control character that could corrupt a wire boundary.
pub fn require_nonempty(value: &str) -> Result<(), ApiError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

/// Reject payloads larger than `maximum_bytes` (UTF-8 byte length).
///
/// # Errors
///
/// Returns [`ApiError::LimitExceeded`] when the payload is too large.
pub fn require_byte_limit(payload: &str, maximum_bytes: usize) -> Result<(), ApiError> {
    if payload.len() > maximum_bytes {
        return Err(ApiError::LimitExceeded);
    }
    Ok(())
}

/// Accept only the single supported contract version.
///
/// # Errors
///
/// Returns [`ApiError::UnsupportedContractVersion`] for any other version.
pub fn require_contract_version(version: u16, expected: u16) -> Result<(), ApiError> {
    if version == expected {
        Ok(())
    } else {
        Err(ApiError::UnsupportedContractVersion)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
    };
    use crate::ApiError;
    use serde::Serialize;
    use serde::ser::Serializer;

    struct SerializationFailure;

    impl Serialize for SerializationFailure {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("intentional"))
        }
    }

    #[test]
    fn wire_helpers_cover_success_and_failure_arms() {
        assert_eq!(
            to_json(&SerializationFailure),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            from_json::<u8>("not-json"),
            Err(ApiError::InvalidWirePayload)
        );
        require_nonempty("tenant-a").expect("ok");
        require_nonempty("line one two").expect("spaced text stays valid");
        assert_eq!(require_nonempty("   "), Err(ApiError::InvalidWirePayload));
        assert_eq!(require_nonempty(""), Err(ApiError::InvalidWirePayload));
        assert_eq!(
            require_nonempty("tenant\u{1f}workspace"),
            Err(ApiError::InvalidWirePayload)
        );
        require_byte_limit("abc", 3).expect("ok");
        assert_eq!(require_byte_limit("abcd", 3), Err(ApiError::LimitExceeded));
        require_contract_version(1, 1).expect("ok");
        assert_eq!(
            require_contract_version(2, 1),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            require_contract_version(0, 1),
            Err(ApiError::UnsupportedContractVersion)
        );
    }
}
