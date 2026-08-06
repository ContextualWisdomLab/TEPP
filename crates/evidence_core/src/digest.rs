//! Canonical cryptographic digests for immutable source content.

use crate::EvidenceError;
use sha2::{Digest, Sha256};
use std::fmt;
use std::str::FromStr;

const DIGEST_BYTE_LENGTH: usize = 32;
const DIGEST_HEX_LENGTH: usize = DIGEST_BYTE_LENGTH * 2;
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// A canonical `SHA-256` digest used to detect source-content changes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContentDigest([u8; DIGEST_BYTE_LENGTH]);

impl ContentDigest {
    /// Hash `content` with `SHA-256`.
    #[must_use]
    pub fn sha256(content: impl AsRef<[u8]>) -> Self {
        let computed = Sha256::digest(content.as_ref());
        let mut digest = [0_u8; DIGEST_BYTE_LENGTH];
        digest.copy_from_slice(&computed);
        Self(digest)
    }

    /// Return the exact 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; DIGEST_BYTE_LENGTH] {
        &self.0
    }

    fn canonical_hex(self) -> String {
        let mut encoded = String::with_capacity(DIGEST_HEX_LENGTH);
        for byte in self.0 {
            encoded.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
        }
        encoded
    }
}

impl fmt::Display for ContentDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.canonical_hex())
    }
}

impl FromStr for ContentDigest {
    type Err = EvidenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != DIGEST_HEX_LENGTH {
            return Err(EvidenceError::InvalidContentDigest);
        }
        if !value.is_ascii() {
            return Err(EvidenceError::InvalidContentDigest);
        }

        let mut digest = [0_u8; DIGEST_BYTE_LENGTH];
        for (index, output) in digest.iter_mut().enumerate() {
            let start = index * 2;
            *output = u8::from_str_radix(&value[start..start + 2], 16)
                .map_err(|_| EvidenceError::InvalidContentDigest)?;
        }
        Ok(Self(digest))
    }
}
