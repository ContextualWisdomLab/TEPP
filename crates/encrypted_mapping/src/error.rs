//! Fail-closed encrypted-mapping errors.

use std::fmt;

/// A fail-closed encrypted-mapping error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EncryptedMappingError {
    /// Opening was requested without a re-identification purpose.
    UnauthorizedPurpose,
    /// The supplied key identity does not match the sealed envelope.
    KeyIdentityMismatch,
    /// AEAD authentication of the sealed envelope failed.
    AuthenticationFailed,
    /// A source identity was empty or a key was all zeros.
    EmptyIdentity,
    /// A recovery or seal payload was empty or length-mismatched.
    InvalidMappingPayload,
    /// Persistence was requested before a later migration exists.
    PersistenceRequiresLaterMigration,
    /// A blanket PII mask was treated as an encryption control.
    BlanketMaskIsNotEncryption,
    /// The operating-system randomness source could not provide a nonce.
    RandomnessUnavailable,
}

impl fmt::Display for EncryptedMappingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnauthorizedPurpose => {
                "encrypted identity mappings open only under re-identification purpose"
            }
            Self::KeyIdentityMismatch => "mapping key identity does not match the sealed envelope",
            Self::AuthenticationFailed => "encrypted identity mapping authentication failed",
            Self::EmptyIdentity => "encrypted identity mapping rejected an empty identity or key",
            Self::InvalidMappingPayload => "invalid encrypted identity mapping payload",
            Self::PersistenceRequiresLaterMigration => {
                "encrypted identity mapping persistence waits for a later migration"
            }
            Self::BlanketMaskIsNotEncryption => {
                "a blanket PII mask is not encrypted identity mapping"
            }
            Self::RandomnessUnavailable => {
                "operating-system randomness was unavailable for the encryption nonce"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for EncryptedMappingError {}

#[cfg(test)]
mod tests {
    use super::EncryptedMappingError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                EncryptedMappingError::UnauthorizedPurpose,
                "encrypted identity mappings open only under re-identification purpose",
            ),
            (
                EncryptedMappingError::KeyIdentityMismatch,
                "mapping key identity does not match the sealed envelope",
            ),
            (
                EncryptedMappingError::AuthenticationFailed,
                "encrypted identity mapping authentication failed",
            ),
            (
                EncryptedMappingError::EmptyIdentity,
                "encrypted identity mapping rejected an empty identity or key",
            ),
            (
                EncryptedMappingError::InvalidMappingPayload,
                "invalid encrypted identity mapping payload",
            ),
            (
                EncryptedMappingError::PersistenceRequiresLaterMigration,
                "encrypted identity mapping persistence waits for a later migration",
            ),
            (
                EncryptedMappingError::BlanketMaskIsNotEncryption,
                "a blanket PII mask is not encrypted identity mapping",
            ),
            (
                EncryptedMappingError::RandomnessUnavailable,
                "operating-system randomness was unavailable for the encryption nonce",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
