//! Durable content identity for scientific-recovery payload and execution provenance.
//!
//! The numerical recovery gate lives in [`crate::scientific_recovery`]. This module
//! wraps that gate so exported scientific authority commits both the exact represented
//! grouped truth/recovery payload and the ordered per-replication RNG-state/execution
//! receipts presented by the trusted simulation adapter. These digests are content and
//! lineage identities only; they are not signatures, seed-manifest membership proofs,
//! execution attestations, or chronology proofs.

use crate::scientific_recovery;
use crate::{PromotedClaim, ValidationError};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

const RECOVERY_EVIDENCE_SCHEMA: &str = "tepp.scientific_recovery_evidence.v1";
const REPLICATION_PAYLOAD_SCHEMA: &str = "tepp.scientific_recovery_replication_payload.v1";
const REPLICATION_RECEIPT_SCHEMA: &str = "tepp.scientific_recovery_replication_receipt.v1";
const REPLICATION_PROVENANCE_SCHEMA: &str = "tepp.scientific_recovery_replication_provenance.v1";

/// Trusted-adapter identity for one planned simulation repetition and its execution artifact.
///
/// The receipt binds one zero-based repetition index to the recovery profile, the profile's
/// seed-manifest identity, one planned RNG-state/seed-entry identity, one immutable execution
/// artifact identity, and the exact represented truth/recovery payload identity for that
/// repetition. The receipt digest is derived inside this value object; supplied dependency
/// digests remain trusted-adapter inputs and are not cryptographic proof of execution or
/// seed-manifest membership.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScientificRecoveryReplicationReceiptV1 {
    replication_index: usize,
    profile_sha256: String,
    seed_manifest_sha256: String,
    seed_state_sha256: String,
    execution_artifact_sha256: String,
    payload_sha256: String,
    receipt_sha256: String,
}

impl ScientificRecoveryReplicationReceiptV1 {
    /// Construct one versioned per-replication execution-provenance receipt.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when any supplied SHA-256 is not
    /// canonical lowercase hexadecimal.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        replication_index: usize,
        profile_sha256: &str,
        seed_manifest_sha256: &str,
        seed_state_sha256: &str,
        execution_artifact_sha256: &str,
        payload_sha256: &str,
    ) -> Result<Self, ValidationError> {
        for value in [
            profile_sha256,
            seed_manifest_sha256,
            seed_state_sha256,
            execution_artifact_sha256,
            payload_sha256,
        ] {
            if !is_canonical_sha256(value) {
                return Err(ValidationError::InvalidInput);
            }
        }

        let mut digest = Sha256::new();
        update_digest_field(&mut digest, REPLICATION_RECEIPT_SCHEMA.as_bytes());
        update_digest_usize(&mut digest, replication_index);
        update_digest_field(&mut digest, profile_sha256.as_bytes());
        update_digest_field(&mut digest, seed_manifest_sha256.as_bytes());
        update_digest_field(&mut digest, seed_state_sha256.as_bytes());
        update_digest_field(&mut digest, execution_artifact_sha256.as_bytes());
        update_digest_field(&mut digest, payload_sha256.as_bytes());
        let receipt_sha256 = hex_encode(&digest.finalize());

        Ok(Self {
            replication_index,
            profile_sha256: profile_sha256.to_owned(),
            seed_manifest_sha256: seed_manifest_sha256.to_owned(),
            seed_state_sha256: seed_state_sha256.to_owned(),
            execution_artifact_sha256: execution_artifact_sha256.to_owned(),
            payload_sha256: payload_sha256.to_owned(),
            receipt_sha256,
        })
    }

    /// Zero-based independent simulation repetition index.
    #[must_use]
    pub const fn replication_index(&self) -> usize {
        self.replication_index
    }

    /// Exact versioned recovery-profile identity declared for this execution.
    #[must_use]
    pub fn profile_sha256(&self) -> &str {
        &self.profile_sha256
    }

    /// Exact seed-manifest identity declared by the recovery profile.
    #[must_use]
    pub fn seed_manifest_sha256(&self) -> &str {
        &self.seed_manifest_sha256
    }

    /// Planned RNG-state or seed-entry identity for this repetition.
    #[must_use]
    pub fn seed_state_sha256(&self) -> &str {
        &self.seed_state_sha256
    }

    /// Immutable execution-artifact identity for this repetition.
    #[must_use]
    pub fn execution_artifact_sha256(&self) -> &str {
        &self.execution_artifact_sha256
    }

    /// Exact represented truth/recovery payload identity for this repetition.
    #[must_use]
    pub fn payload_sha256(&self) -> &str {
        &self.payload_sha256
    }

    /// Domain-separated identity of the complete per-replication receipt binding.
    #[must_use]
    pub fn receipt_sha256(&self) -> &str {
        &self.receipt_sha256
    }
}

/// Scientific authority bound to recovery-profile, exact-head receipt, payload, and execution identities.
#[derive(Clone, Debug, PartialEq)]
pub struct ScientificRecoveryPromotionV1 {
    inner: scientific_recovery::ScientificRecoveryPromotionV1,
    recovery_evidence_sha256: String,
    replication_provenance_sha256: String,
}

impl ScientificRecoveryPromotionV1 {
    /// Promoted scientific claim.
    #[must_use]
    pub const fn claim(&self) -> PromotedClaim {
        self.inner.claim()
    }

    /// Exact SHA-256 of the versioned recovery profile used for promotion.
    #[must_use]
    pub fn profile_sha256(&self) -> &str {
        self.inner.profile_sha256()
    }

    /// Exact SHA-256 of the head/artifact/status receipt binding used for promotion.
    #[must_use]
    pub fn exact_head_receipt_sha256(&self) -> &str {
        self.inner.exact_head_receipt_sha256()
    }

    /// Domain-separated SHA-256 of the exact represented grouped recovery payload.
    #[must_use]
    pub fn recovery_evidence_sha256(&self) -> &str {
        &self.recovery_evidence_sha256
    }

    /// Domain-separated SHA-256 of the ordered per-replication execution receipts.
    #[must_use]
    pub fn replication_provenance_sha256(&self) -> &str {
        &self.replication_provenance_sha256
    }
}

/// Derive the canonical identity of one represented truth/recovery repetition payload.
///
/// Signed zero is canonicalized because `+0.0` and `-0.0` have the same scientific
/// represented value for this evidence contract. Coordinate order and cardinality remain
/// committed because the state-composition contract gives those positions meaning.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when either row is empty, cardinalities differ,
/// or any coordinate is non-finite.
pub fn scientific_recovery_replication_payload_sha256(
    truth: &[f64],
    recovered: &[f64],
) -> Result<String, ValidationError> {
    if truth.is_empty()
        || truth.len() != recovered.len()
        || truth.iter().chain(recovered).any(|value| !value.is_finite())
    {
        return Err(ValidationError::InvalidInput);
    }

    let mut digest = Sha256::new();
    update_digest_field(&mut digest, REPLICATION_PAYLOAD_SCHEMA.as_bytes());
    update_digest_usize(&mut digest, truth.len());
    update_digest_field(&mut digest, b"truth");
    for value in truth {
        update_digest_field(&mut digest, &canonical_f64_bits(*value).to_le_bytes());
    }
    update_digest_field(&mut digest, b"recovered");
    for value in recovered {
        update_digest_field(&mut digest, &canonical_f64_bits(*value).to_le_bytes());
    }
    Ok(hex_encode(&digest.finalize()))
}

/// Promote a scientific claim and retain evaluated payload plus per-replication execution identities.
///
/// `replication_receipts[i]` must describe outer scientific repetition `i`, bind the exact
/// profile and seed-manifest identities, and carry the exact payload digest recomputed by this
/// owner from `truth_replications[i]` and `recovered_replications[i]`. Planned RNG-state/seed
/// entry identities must be unique across independent repetitions so accidental seed reuse
/// cannot masquerade as additional Monte Carlo evidence.
///
/// The underlying Validation Evidence owner then performs the existing profile, replication,
/// numerical, exact-head receipt, and ADR 0014 authority checks. Successful authority retains
/// separate digests for the grouped scientific payload and ordered execution provenance.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for receipt count/order/profile/manifest/payload
/// mismatch, duplicate planned seed-state identity, malformed represented replication payload,
/// or any canonical scientific-recovery input refusal. Other fail-closed errors propagate from
/// the numerical/profile/exact-head/ADR 0014 gate.
pub fn promote_scientific_recovery(
    candidate_head: &str,
    protected_head: &str,
    truth_replications: &[&[f64]],
    recovered_replications: &[&[f64]],
    profile: &scientific_recovery::ScientificRecoveryProfileV1,
    exact_head_receipt: &scientific_recovery::ScientificRecoveryExactHeadReceiptV1,
    replication_receipts: &[ScientificRecoveryReplicationReceiptV1],
) -> Result<ScientificRecoveryPromotionV1, ValidationError> {
    let replication_provenance_sha256 = validate_replication_receipts(
        profile,
        truth_replications,
        recovered_replications,
        replication_receipts,
    )?;

    let inner = scientific_recovery::promote_scientific_recovery(
        candidate_head,
        protected_head,
        truth_replications,
        recovered_replications,
        profile,
        exact_head_receipt,
    )?;
    let recovery_evidence_sha256 =
        recovery_evidence_sha256(profile, truth_replications, recovered_replications);

    Ok(ScientificRecoveryPromotionV1 {
        inner,
        recovery_evidence_sha256,
        replication_provenance_sha256,
    })
}

fn validate_replication_receipts(
    profile: &scientific_recovery::ScientificRecoveryProfileV1,
    truth_replications: &[&[f64]],
    recovered_replications: &[&[f64]],
    replication_receipts: &[ScientificRecoveryReplicationReceiptV1],
) -> Result<String, ValidationError> {
    if replication_receipts.len() != profile.planned_replications()
        || replication_receipts.len() != truth_replications.len()
        || replication_receipts.len() != recovered_replications.len()
    {
        return Err(ValidationError::InvalidInput);
    }

    let profile_sha256 = profile.sha256();
    let mut seed_states = HashSet::with_capacity(replication_receipts.len());
    let mut digest = Sha256::new();
    update_digest_field(&mut digest, REPLICATION_PROVENANCE_SCHEMA.as_bytes());
    update_digest_field(&mut digest, profile_sha256.as_bytes());
    update_digest_usize(&mut digest, replication_receipts.len());

    for (index, ((truth, recovered), receipt)) in truth_replications
        .iter()
        .zip(recovered_replications.iter())
        .zip(replication_receipts.iter())
        .enumerate()
    {
        let expected_payload = scientific_recovery_replication_payload_sha256(truth, recovered)?;
        if receipt.replication_index() != index
            || receipt.profile_sha256() != profile_sha256
            || receipt.seed_manifest_sha256() != profile.seed_manifest_sha256()
            || receipt.payload_sha256() != expected_payload
            || !seed_states.insert(receipt.seed_state_sha256())
        {
            return Err(ValidationError::InvalidInput);
        }
        update_digest_usize(&mut digest, index);
        update_digest_field(&mut digest, receipt.receipt_sha256().as_bytes());
    }

    Ok(hex_encode(&digest.finalize()))
}

fn recovery_evidence_sha256(
    profile: &scientific_recovery::ScientificRecoveryProfileV1,
    truth_replications: &[&[f64]],
    recovered_replications: &[&[f64]],
) -> String {
    let mut digest = Sha256::new();
    update_digest_field(&mut digest, RECOVERY_EVIDENCE_SCHEMA.as_bytes());
    let profile_sha256 = profile.sha256();
    update_digest_field(&mut digest, profile_sha256.as_bytes());
    update_digest_usize(&mut digest, truth_replications.len());

    for (index, (truth, recovered)) in truth_replications
        .iter()
        .zip(recovered_replications.iter())
        .enumerate()
    {
        update_digest_usize(&mut digest, index);
        update_digest_field(&mut digest, b"truth");
        update_digest_usize(&mut digest, truth.len());
        for value in *truth {
            update_digest_field(&mut digest, &canonical_f64_bits(*value).to_le_bytes());
        }
        update_digest_field(&mut digest, b"recovered");
        update_digest_usize(&mut digest, recovered.len());
        for value in *recovered {
            update_digest_field(&mut digest, &canonical_f64_bits(*value).to_le_bytes());
        }
    }

    hex_encode(&digest.finalize())
}

fn canonical_f64_bits(value: f64) -> u64 {
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn update_digest_usize(digest: &mut Sha256, value: usize) {
    update_digest_field(digest, &value.to_le_bytes());
}

fn update_digest_field(digest: &mut Sha256, field: &[u8]) {
    digest.update(field.len().to_le_bytes());
    digest.update(field);
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
