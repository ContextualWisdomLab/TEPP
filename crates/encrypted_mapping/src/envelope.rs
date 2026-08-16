//! Keyed SHA-256 HMAC envelope for identity mappings.

use crate::EncryptedMappingError;
use sha2::{Digest, Sha256};

const ENC_CONTEXT: &[u8] = b"tepp-encrypted-mapping-enc";
const MAC_CONTEXT: &[u8] = b"tepp-encrypted-mapping-mac";

/// Caller-held mapping key identity and bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MappingKey {
    key_id: u128,
    key_bytes: [u8; 32],
}

impl MappingKey {
    /// Bind a key identity to 32 nonzero key bytes.
    ///
    /// # Errors
    ///
    /// Returns [`EncryptedMappingError::EmptyIdentity`] when every key byte is zero.
    pub const fn new(key_id: u128, key_bytes: [u8; 32]) -> Result<Self, EncryptedMappingError> {
        let mut index = 0;
        let mut nonzero = false;
        while index < key_bytes.len() {
            if key_bytes[index] != 0 {
                nonzero = true;
                break;
            }
            index += 1;
        }
        if !nonzero {
            return Err(EncryptedMappingError::EmptyIdentity);
        }
        Ok(Self { key_id, key_bytes })
    }

    /// Derive a 32-byte mapping key from caller-held material of any length.
    ///
    /// Material that is already 32 nonzero bytes is used directly. Longer or
    /// shorter nonzero material is reduced with HMAC-SHA-256.
    ///
    /// # Errors
    ///
    /// Returns [`EncryptedMappingError::EmptyIdentity`] when the material is
    /// empty or every byte is zero.
    pub fn from_material(key_id: u128, material: &[u8]) -> Result<Self, EncryptedMappingError> {
        if material.is_empty() || material.iter().all(|byte| *byte == 0) {
            return Err(EncryptedMappingError::EmptyIdentity);
        }
        let key_bytes = if material.len() == 32 {
            let mut bytes = [0_u8; 32];
            bytes.copy_from_slice(material);
            bytes
        } else {
            hmac_sha256(material, b"tepp-encrypted-mapping-key")
        };
        Self::new(key_id, key_bytes)
    }

    /// Return the key identity copied into sealed envelopes.
    #[must_use]
    pub const fn key_id(self) -> u128 {
        self.key_id
    }
}

/// Closed purpose vocabulary for opening a sealed mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingPurpose {
    /// Ordinary psychometric or longitudinal computation.
    AnalyticalComputation,
    /// Explicit re-identification of the protected mapping.
    ReidentificationExport,
    /// Ordinary operational logs.
    OperationalLog,
    /// Model artifacts, prompts, or provider payloads.
    ModelArtifact,
}

/// Sealed analytical-id to source-identity envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncryptedIdentityMapping {
    analytical_id: u128,
    key_id: u128,
    nonce: [u8; 16],
    ciphertext: Vec<u8>,
    tag: [u8; 32],
}

impl EncryptedIdentityMapping {
    /// Opaque identifier used in ordinary compute artifacts.
    #[must_use]
    pub const fn analytical_id(&self) -> u128 {
        self.analytical_id
    }

    /// Key identity required to authenticate the envelope.
    #[must_use]
    pub const fn key_id(&self) -> u128 {
        self.key_id
    }
}

/// Seal a source identity so analytical artifacts cannot read it.
///
/// # Errors
///
/// Returns [`EncryptedMappingError::EmptyIdentity`] when the identity is empty.
pub fn seal_identity(
    analytical_id: u128,
    source_identity: &[u8],
    key: &MappingKey,
    nonce: [u8; 16],
) -> Result<EncryptedIdentityMapping, EncryptedMappingError> {
    if source_identity.is_empty() {
        return Err(EncryptedMappingError::EmptyIdentity);
    }
    let ciphertext = xor_keystream(source_identity, &derive_key(key, ENC_CONTEXT), &nonce);
    let tag = authenticate(key, analytical_id, &nonce, &ciphertext);
    Ok(EncryptedIdentityMapping {
        analytical_id,
        key_id: key.key_id,
        nonce,
        ciphertext,
        tag,
    })
}

/// Open a sealed mapping only under re-identification purpose.
///
/// # Errors
///
/// Returns purpose, key-identity, or authentication errors when opening is
/// unauthorized or the envelope does not authenticate.
pub fn open_identity(
    envelope: &EncryptedIdentityMapping,
    key: &MappingKey,
    purpose: MappingPurpose,
) -> Result<Vec<u8>, EncryptedMappingError> {
    if !matches!(purpose, MappingPurpose::ReidentificationExport) {
        return Err(EncryptedMappingError::UnauthorizedPurpose);
    }
    if envelope.key_id != key.key_id {
        return Err(EncryptedMappingError::KeyIdentityMismatch);
    }
    let expected = authenticate(
        key,
        envelope.analytical_id,
        &envelope.nonce,
        &envelope.ciphertext,
    );
    if !tags_equal(&expected, &envelope.tag) {
        return Err(EncryptedMappingError::AuthenticationFailed);
    }
    Ok(xor_keystream(
        &envelope.ciphertext,
        &derive_key(key, ENC_CONTEXT),
        &envelope.nonce,
    ))
}

/// Refuse to treat a blanket PII mask as encryption.
///
/// # Errors
///
/// Always returns [`EncryptedMappingError::BlanketMaskIsNotEncryption`].
pub fn refuse_blanket_mask_as_encryption() -> Result<(), EncryptedMappingError> {
    Err(EncryptedMappingError::BlanketMaskIsNotEncryption)
}

/// Refuse to persist the mapping until a later migration exists.
///
/// # Errors
///
/// Always returns [`EncryptedMappingError::PersistenceRequiresLaterMigration`].
pub fn refuse_persistence_without_later_migration() -> Result<(), EncryptedMappingError> {
    Err(EncryptedMappingError::PersistenceRequiresLaterMigration)
}

/// Fraction of recovered identities that match known truth.
///
/// # Errors
///
/// Returns [`EncryptedMappingError::InvalidMappingPayload`] when either slice
/// is empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[&str],
    decided: &[String],
) -> Result<f64, EncryptedMappingError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(EncryptedMappingError::InvalidMappingPayload);
    }
    let mut matches = 0_u32;
    for (truth_identity, decided_identity) in truth.iter().zip(decided) {
        if *truth_identity == decided_identity {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

fn derive_key(key: &MappingKey, context: &[u8]) -> [u8; 32] {
    hmac_sha256(&key.key_bytes, context)
}

fn authenticate(
    key: &MappingKey,
    analytical_id: u128,
    nonce: &[u8; 16],
    ciphertext: &[u8],
) -> [u8; 32] {
    let mut message = Vec::with_capacity(16 + 16 + ciphertext.len());
    message.extend_from_slice(&analytical_id.to_be_bytes());
    message.extend_from_slice(nonce);
    message.extend_from_slice(ciphertext);
    hmac_sha256(&derive_key(key, MAC_CONTEXT), &message)
}

fn xor_keystream(payload: &[u8], enc_key: &[u8; 32], nonce: &[u8; 16]) -> Vec<u8> {
    payload
        .chunks(32)
        .enumerate()
        .flat_map(|(block_index, chunk)| {
            let mut block_input = [0_u8; 24];
            block_input[..16].copy_from_slice(nonce);
            block_input[16..].copy_from_slice(&(block_index as u64).to_be_bytes());
            let block = hmac_sha256(enc_key, &block_input);
            chunk
                .iter()
                .zip(block)
                .map(|(byte, mask)| byte ^ mask)
                .collect::<Vec<u8>>()
        })
        .collect()
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let key_block = keyed_block(key);
    let mut ipad = [0x36_u8; 64];
    let mut opad = [0x5c_u8; 64];
    for index in 0..64 {
        ipad[index] ^= key_block[index];
        opad[index] ^= key_block[index];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(data);
    let inner_hash = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    let digest = outer.finalize();
    let mut tag = [0_u8; 32];
    tag.copy_from_slice(&digest);
    tag
}

fn keyed_block(key: &[u8]) -> [u8; 64] {
    let mut block = [0_u8; 64];
    if key.len() > 64 {
        let hashed = Sha256::digest(key);
        block[..32].copy_from_slice(&hashed);
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    block
}

fn tags_equal(left: &[u8; 32], right: &[u8; 32]) -> bool {
    let mut acc = 0_u8;
    for index in 0..32 {
        acc |= left[index] ^ right[index];
    }
    acc == 0
}

#[cfg(test)]
mod tests {
    use super::{
        MappingKey, MappingPurpose, hmac_sha256, identity_recovery_rate, open_identity,
        seal_identity, tags_equal,
    };
    use crate::EncryptedMappingError;

    #[test]
    fn hmac_sha256_matches_rfc_4231_case_one() {
        let tag = hmac_sha256(&[0x0b; 20], b"Hi There");
        let expected = [
            0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
            0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
            0x2e, 0x32, 0xcf, 0xf7,
        ];
        assert_eq!(tag, expected);
    }

    #[test]
    fn hmac_hashes_keys_longer_than_one_block() {
        let long_key = [0xaa_u8; 131];
        let tag = hmac_sha256(
            &long_key,
            b"Test Using Larger Than Block-Size Key - Hash Key First",
        );
        let expected = [
            0x60, 0xe4, 0x31, 0x59, 0x1e, 0xe0, 0xb6, 0x7f, 0x0d, 0x8a, 0x26, 0xaa, 0xcb, 0xf5,
            0xb7, 0x7f, 0x8e, 0x0b, 0xc6, 0x21, 0x37, 0x28, 0xc5, 0x14, 0x05, 0x46, 0x04, 0x0f,
            0x0e, 0xe3, 0x7f, 0x54,
        ];
        assert_eq!(tag, expected);
    }

    #[test]
    fn tampered_ciphertext_or_tag_fails_authentication() {
        let mut delayed = [0_u8; 32];
        delayed[5] = 0x33;
        let key = MappingKey::new(1, delayed).expect("key");
        let sealed = seal_identity(9, b"department-north", &key, [9; 16]).expect("seal");
        let mut body = sealed.clone();
        body.ciphertext[0] ^= 0x01;
        assert_eq!(
            open_identity(&body, &key, MappingPurpose::ReidentificationExport),
            Err(EncryptedMappingError::AuthenticationFailed)
        );
        let mut tagged = sealed;
        tagged.tag[0] ^= 0x01;
        assert_eq!(
            open_identity(&tagged, &key, MappingPurpose::ReidentificationExport),
            Err(EncryptedMappingError::AuthenticationFailed)
        );
        assert!(!tags_equal(&[0; 32], &[1; 32]));
    }

    #[test]
    fn long_identity_uses_more_than_one_keystream_block() {
        let key = MappingKey::new(2, [0x44; 32]).expect("key");
        let identity = vec![0x55_u8; 40];
        let sealed = seal_identity(10, &identity, &key, [10; 16]).expect("seal");
        let opened =
            open_identity(&sealed, &key, MappingPurpose::ReidentificationExport).expect("open");
        assert_eq!(opened, identity);
        assert_ne!(sealed.ciphertext, identity);
        assert!(
            !sealed
                .ciphertext
                .windows(identity.len())
                .any(|window| window == identity)
        );
    }

    #[test]
    fn collapsed_identities_recover_worse_than_the_sealed_path() {
        let truth = ["alice", "bob"];
        let recovered = identity_recovery_rate(&truth, &["alice".to_owned(), "bob".to_owned()])
            .expect("recovered");
        let collapsed = identity_recovery_rate(&truth, &["alice".to_owned(), "alice".to_owned()])
            .expect("collapsed");
        assert!(recovered > collapsed);
    }

    #[test]
    fn debug_does_not_print_the_source_identity() {
        let key = MappingKey::new(3, [0x55; 32]).expect("key");
        let sealed = seal_identity(11, b"secret-name", &key, [11; 16]).expect("seal");
        let rendered = format!("{sealed:?}");
        assert!(!rendered.contains("secret-name"));
        assert_eq!(sealed.analytical_id(), 11);
        assert_eq!(sealed.key_id(), 3);
    }
}
