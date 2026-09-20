//! Durable content identity for the exact recovery payload admitted to scientific authority.
//!
//! The numerical recovery gate lives in [`crate::scientific_recovery`]. This module
//! wraps that gate so the exported promotion result also commits the exact represented
//! grouped truth/recovery payload that was evaluated. The digest is content identity
//! only; it is not an execution signature, seed-execution proof, or chronology proof.

use crate::scientific_recovery;
use crate::{PromotedClaim, ValidationError};
use sha2::{Digest, Sha256};

const RECOVERY_EVIDENCE_SCHEMA: &str = "tepp.scientific_recovery_evidence.v1";

/// Scientific authority bound to recovery-profile, exact-head receipt, and payload identities.
#[derive(Clone, Debug, PartialEq)]
pub struct ScientificRecoveryPromotionV1 {
    inner: scientific_recovery::ScientificRecoveryPromotionV1,
    recovery_evidence_sha256: String,
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
}

/// Promote a scientific claim and retain the exact evaluated recovery-payload identity.
///
/// The underlying Validation Evidence owner first performs the existing profile,
/// replication, numerical, exact-head receipt, and ADR 0014 authority checks. Only a
/// successful promotion receives a recovery evidence digest. That digest commits the
/// profile identity plus outer/inner cardinality and ordered canonical binary64 values
/// for every truth/recovery coordinate; signed zero is canonicalized to positive zero.
///
/// # Errors
///
/// Propagates every fail-closed error from the canonical scientific recovery gate and
/// returns [`ValidationError::InvalidInput`] if a payload cardinality cannot be encoded
/// in the versioned evidence identity.
pub fn promote_scientific_recovery(
    candidate_head: &str,
    protected_head: &str,
    truth_replications: &[&[f64]],
    recovered_replications: &[&[f64]],
    profile: &scientific_recovery::ScientificRecoveryProfileV1,
    exact_head_receipt: &scientific_recovery::ScientificRecoveryExactHeadReceiptV1,
) -> Result<ScientificRecoveryPromotionV1, ValidationError> {
    let inner = scientific_recovery::promote_scientific_recovery(
        candidate_head,
        protected_head,
        truth_replications,
        recovered_replications,
        profile,
        exact_head_receipt,
    )?;
    let recovery_evidence_sha256 = recovery_evidence_sha256(
        profile,
        truth_replications,
        recovered_replications,
    )?;

    Ok(ScientificRecoveryPromotionV1 {
        inner,
        recovery_evidence_sha256,
    })
}

fn recovery_evidence_sha256(
    profile: &scientific_recovery::ScientificRecoveryProfileV1,
    truth_replications: &[&[f64]],
    recovered_replications: &[&[f64]],
) -> Result<String, ValidationError> {
    let mut digest = Sha256::new();
    update_digest_field(&mut digest, RECOVERY_EVIDENCE_SCHEMA.as_bytes())?;
    let profile_sha256 = profile.sha256();
    update_digest_field(&mut digest, profile_sha256.as_bytes())?;
    update_digest_u64(&mut digest, truth_replications.len())?;

    for (index, (truth, recovered)) in truth_replications
        .iter()
        .zip(recovered_replications.iter())
        .enumerate()
    {
        update_digest_u64(&mut digest, index)?;
        update_digest_field(&mut digest, b"truth")?;
        update_digest_u64(&mut digest, truth.len())?;
        for value in *truth {
            update_digest_field(&mut digest, &canonical_f64_bits(*value).to_le_bytes())?;
        }
        update_digest_field(&mut digest, b"recovered")?;
        update_digest_u64(&mut digest, recovered.len())?;
        for value in *recovered {
            update_digest_field(&mut digest, &canonical_f64_bits(*value).to_le_bytes())?;
        }
    }

    Ok(hex_encode(&digest.finalize()))
}

fn canonical_f64_bits(value: f64) -> u64 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

fn update_digest_u64(digest: &mut Sha256, value: usize) -> Result<(), ValidationError> {
    let value = u64::try_from(value).map_err(|_| ValidationError::InvalidInput)?;
    update_digest_field(digest, &value.to_le_bytes())
}

fn update_digest_field(digest: &mut Sha256, field: &[u8]) -> Result<(), ValidationError> {
    let len = u64::try_from(field.len()).map_err(|_| ValidationError::InvalidInput)?;
    digest.update(len.to_le_bytes());
    digest.update(field);
    Ok(())
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
