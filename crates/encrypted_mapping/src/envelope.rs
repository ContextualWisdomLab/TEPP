//! AES-256-GCM envelope for purpose-bound identity mappings.

use crate::EncryptedMappingError;
use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, Nonce as AeadNonce, Payload},
};
use sha2::{Digest, Sha256};

const NONCE_LENGTH: usize = 12;
const TAG_LENGTH: usize = 16;
const MAX_SOURCE_IDENTITY_LENGTH: usize = 1 << 20;
const AAD_CONTEXT: &[u8] = b"tepp-encrypted-mapping-aes256gcm-v1";

/// Caller-held mapping key identity and bytes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct MappingKey {
    key_id: u128,
    key_bytes: [u8; 32],
}

#[allow(clippy::missing_fields_in_debug)] // key_bytes is intentionally redacted.
impl std::fmt::Debug for MappingKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MappingKey")
            .field("key_id", &self.key_id)
            .finish()
    }
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
    nonce: [u8; NONCE_LENGTH],
    ciphertext: Vec<u8>,
    tag: [u8; TAG_LENGTH],
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

/// Seal a source identity with AES-256-GCM and an operating-system nonce.
///
/// # Errors
///
/// Returns [`EncryptedMappingError::EmptyIdentity`] when the identity is empty,
/// [`EncryptedMappingError::InvalidMappingPayload`] when it exceeds the
/// bounded identity size, or [`EncryptedMappingError::RandomnessUnavailable`]
/// when the operating system cannot provide a nonce.
///
/// # Panics
///
/// The fixed-size key, nonce, and bounded payload invariants make the internal
/// AES-GCM conversions infallible for values accepted by this API.
pub fn seal_identity(
    analytical_id: u128,
    source_identity: &[u8],
    key: &MappingKey,
) -> Result<EncryptedIdentityMapping, EncryptedMappingError> {
    seal_identity_with_fill(analytical_id, source_identity, key, getrandom::fill)
}

fn seal_identity_with_fill(
    analytical_id: u128,
    source_identity: &[u8],
    key: &MappingKey,
    fill: impl FnOnce(&mut [u8]) -> Result<(), getrandom::Error>,
) -> Result<EncryptedIdentityMapping, EncryptedMappingError> {
    if source_identity.is_empty() {
        return Err(EncryptedMappingError::EmptyIdentity);
    }
    if source_identity.len() > MAX_SOURCE_IDENTITY_LENGTH {
        return Err(EncryptedMappingError::InvalidMappingPayload);
    }
    let mut nonce = [0_u8; NONCE_LENGTH];
    fill_nonce(&mut nonce, fill)?;
    let cipher = Aes256Gcm::new_from_slice(&key.key_bytes).expect("AES-256 key has fixed length");
    let nonce_array =
        AeadNonce::<Aes256Gcm>::try_from(nonce.as_slice()).expect("nonce has fixed length");
    let aad = associated_data(analytical_id, key.key_id);
    let mut sealed = cipher
        .encrypt(
            &nonce_array,
            Payload {
                msg: source_identity,
                aad: &aad,
            },
        )
        .expect("bounded identity fits AES-GCM payload limit");
    let tag_start = sealed.len() - TAG_LENGTH;
    let tag_bytes = sealed.split_off(tag_start);
    let mut tag = [0_u8; TAG_LENGTH];
    tag.copy_from_slice(&tag_bytes);
    Ok(EncryptedIdentityMapping {
        analytical_id,
        key_id: key.key_id,
        nonce,
        ciphertext: sealed,
        tag,
    })
}

/// Open a sealed mapping only under re-identification purpose.
///
/// # Errors
///
/// Returns purpose, key-identity, or authentication errors when opening is
/// unauthorized or the envelope does not authenticate.
///
/// # Panics
///
/// The envelope stores a fixed-size nonce and the key is a fixed-size
/// AES-256 key, so the internal conversions are infallible.
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
    let cipher = Aes256Gcm::new_from_slice(&key.key_bytes).expect("AES-256 key has fixed length");
    let aad = associated_data(envelope.analytical_id, envelope.key_id);
    let mut sealed = envelope.ciphertext.clone();
    sealed.extend_from_slice(&envelope.tag);
    let nonce = AeadNonce::<Aes256Gcm>::try_from(envelope.nonce.as_slice())
        .expect("nonce has fixed length");
    cipher
        .decrypt(
            &nonce,
            Payload {
                msg: &sealed,
                aad: &aad,
            },
        )
        .map_err(|_| EncryptedMappingError::AuthenticationFailed)
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

fn associated_data(analytical_id: u128, key_id: u128) -> Vec<u8> {
    let mut data = Vec::with_capacity(AAD_CONTEXT.len() + 32);
    data.extend_from_slice(AAD_CONTEXT);
    data.extend_from_slice(&analytical_id.to_be_bytes());
    data.extend_from_slice(&key_id.to_be_bytes());
    data
}

fn fill_nonce(
    nonce: &mut [u8; NONCE_LENGTH],
    fill: impl FnOnce(&mut [u8]) -> Result<(), getrandom::Error>,
) -> Result<(), EncryptedMappingError> {
    fill(nonce).map_err(|_| EncryptedMappingError::RandomnessUnavailable)
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

#[cfg(test)]
mod tests {
    use super::{
        MAX_SOURCE_IDENTITY_LENGTH, MappingKey, MappingPurpose, NONCE_LENGTH, fill_nonce,
        hmac_sha256, identity_recovery_rate, keyed_block, open_identity, seal_identity,
        seal_identity_with_fill,
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
    fn keyed_block_copies_keys_at_the_block_boundary() {
        let key_length = std::hint::black_box(64_usize);
        let key = vec![0xa5_u8; key_length];

        assert_eq!(&keyed_block(&key)[..key_length], key.as_slice());
    }

    #[test]
    fn tampered_ciphertext_or_tag_fails_authentication() {
        let mut delayed = [0_u8; 32];
        delayed[5] = 0x33;
        let key = MappingKey::new(1, delayed).expect("key");
        let sealed = seal_identity(9, b"department-north", &key).expect("seal");
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
    }

    #[test]
    fn randomness_failure_fails_closed() {
        let mut nonce = [0_u8; NONCE_LENGTH];
        assert_eq!(
            fill_nonce(&mut nonce, |_| Err(getrandom::Error::new_custom(1))),
            Err(EncryptedMappingError::RandomnessUnavailable)
        );
        let key = MappingKey::new(15, [0x99; 32]).expect("key");
        assert_eq!(
            seal_identity_with_fill(15, b"secret-name", &key, |_| {
                Err(getrandom::Error::new_custom(1))
            }),
            Err(EncryptedMappingError::RandomnessUnavailable)
        );
    }

    #[test]
    fn long_identity_round_trips_through_aead() {
        let key = MappingKey::new(2, [0x44; 32]).expect("key");
        let identity = vec![0x55_u8; 40];
        let sealed = seal_identity(10, &identity, &key).expect("seal");
        let opened =
            open_identity(&sealed, &key, MappingPurpose::ReidentificationExport).expect("open");
        assert_eq!(opened, identity);
        assert_ne!(sealed.ciphertext, identity);
    }

    #[test]
    fn oversized_identity_fails_closed_before_encryption() {
        let key = MappingKey::new(14, [0x88; 32]).expect("key");
        let oversized = vec![0_u8; MAX_SOURCE_IDENTITY_LENGTH + 1];

        assert_eq!(
            seal_identity(14, &oversized, &key),
            Err(EncryptedMappingError::InvalidMappingPayload)
        );
    }

    #[test]
    fn empty_payloads_fail_closed() {
        let key = MappingKey::new(15, [0x88; 32]).expect("key");

        assert_eq!(
            seal_identity(15, b"", &key),
            Err(EncryptedMappingError::EmptyIdentity)
        );
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(EncryptedMappingError::InvalidMappingPayload)
        );
        assert_eq!(
            identity_recovery_rate(&["a"], &[]),
            Err(EncryptedMappingError::InvalidMappingPayload)
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
        let sealed = seal_identity(11, b"secret-name", &key).expect("seal");
        let rendered = format!("{sealed:?}");
        assert!(!rendered.contains("secret-name"));
        assert_eq!(sealed.analytical_id(), 11);
        assert_eq!(sealed.key_id(), 3);
    }

    #[test]
    fn analytical_id_is_authenticated_as_associated_data() {
        let key = MappingKey::new(4, [0x77; 32]).expect("key");
        let mut altered = seal_identity(12, b"secret-name", &key).expect("seal");
        altered.analytical_id = 13;

        assert_eq!(
            open_identity(&altered, &key, MappingPurpose::ReidentificationExport),
            Err(EncryptedMappingError::AuthenticationFailed)
        );
    }

    #[test]
    fn ordinary_purposes_cannot_open_a_mapping() {
        let key = MappingKey::new(5, [0x66; 32]).expect("key");
        let sealed = seal_identity(13, b"secret-name", &key).expect("seal");

        for purpose in [
            MappingPurpose::AnalyticalComputation,
            MappingPurpose::OperationalLog,
            MappingPurpose::ModelArtifact,
        ] {
            assert_eq!(
                open_identity(&sealed, &key, purpose),
                Err(EncryptedMappingError::UnauthorizedPurpose)
            );
        }

        let other_key = MappingKey::new(6, [0x66; 32]).expect("other key");
        assert_eq!(
            open_identity(&sealed, &other_key, MappingPurpose::ReidentificationExport),
            Err(EncryptedMappingError::KeyIdentityMismatch)
        );
    }

    #[test]
    fn mapping_key_debug_does_not_print_key_bytes() {
        let key = MappingKey::new(12, [0xa5; 32]).expect("key");

        assert_eq!(format!("{key:?}"), "MappingKey { key_id: 12 }");
    }

    #[test]
    fn mapping_key_rejects_runtime_all_zero_bytes() {
        let key_bytes = std::hint::black_box([0_u8; 32]);

        assert_eq!(
            MappingKey::new(13, key_bytes),
            Err(EncryptedMappingError::EmptyIdentity)
        );
    }
}
