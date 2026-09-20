//! Durable content identity for scientific-recovery payload and execution provenance.
//!
//! The numerical recovery gate lives in [`crate::scientific_recovery`]. This module
//! wraps that gate so exported scientific authority commits the exact represented
//! grouped truth/recovery payload, ordered per-replication RNG-state/execution receipts,
//! and owner-ledger evidence that the recovery profile was approved before execution
//! entries were issued. These digests are content and lineage identities; trusted ledger
//! inputs remain distinct from cryptographic attestations.

use crate::scientific_recovery;
use crate::{PromotedClaim, ValidationError};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

const RECOVERY_EVIDENCE_SCHEMA: &str = "tepp.scientific_recovery_evidence.v1";
const REPLICATION_PAYLOAD_SCHEMA: &str = "tepp.scientific_recovery_replication_payload.v1";
const REPLICATION_RECEIPT_SCHEMA: &str = "tepp.scientific_recovery_replication_receipt.v1";
const REPLICATION_PROVENANCE_SCHEMA: &str = "tepp.scientific_recovery_replication_provenance.v1";
const SEED_MANIFEST_SCHEMA: &str = "tepp.scientific_recovery_seed_manifest.v1";
const PROFILE_CHRONOLOGY_SCHEMA: &str = "tepp.scientific_recovery_profile_chronology.v1";

/// Canonical ordered manifest of planned RNG-state or seed-entry identities.
///
/// The manifest identity is derived from the ordered unique canonical SHA-256 entries.
/// Scientific recovery reconstructs this value from the presented per-replication receipts
/// and requires its digest to equal the manifest identity already committed by the recovery
/// profile. This proves membership and order against the represented manifest content; it
/// does not prove when the manifest was approved or that an execution actually used a seed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScientificRecoverySeedManifestV1 {
    seed_state_sha256: Vec<String>,
    sha256: String,
}

impl ScientificRecoverySeedManifestV1 {
    /// Construct a canonical ordered seed manifest.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when fewer than two entries are supplied,
    /// an entry is not canonical lowercase SHA-256 hexadecimal, or any entry is duplicated.
    pub fn new(seed_state_sha256: &[&str]) -> Result<Self, ValidationError> {
        if seed_state_sha256.len() < 2 {
            return Err(ValidationError::InvalidInput);
        }
        let mut unique = HashSet::with_capacity(seed_state_sha256.len());
        for value in seed_state_sha256 {
            if !is_canonical_sha256(value) || !unique.insert(*value) {
                return Err(ValidationError::InvalidInput);
            }
        }

        let mut digest = Sha256::new();
        update_digest_field(&mut digest, SEED_MANIFEST_SCHEMA.as_bytes());
        update_digest_usize(&mut digest, seed_state_sha256.len());
        for (index, value) in seed_state_sha256.iter().enumerate() {
            update_digest_usize(&mut digest, index);
            update_digest_field(&mut digest, value.as_bytes());
        }

        Ok(Self {
            seed_state_sha256: seed_state_sha256
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
            sha256: hex_encode(&digest.finalize()),
        })
    }

    /// Number of planned independent seed-state entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.seed_state_sha256.len()
    }

    /// Whether the manifest contains no planned entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.seed_state_sha256.is_empty()
    }

    /// Planned seed-state identity at one zero-based repetition index.
    #[must_use]
    pub fn seed_state_sha256(&self, index: usize) -> Option<&str> {
        self.seed_state_sha256.get(index).map(String::as_str)
    }

    /// Domain-separated SHA-256 identity of the ordered manifest content.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Trusted-adapter identity for one planned simulation repetition and its execution artifact.
///
/// The receipt binds one zero-based repetition index to the recovery profile, the profile's
/// seed-manifest identity, one planned RNG-state/seed-entry identity, one immutable execution
/// artifact identity, and the exact represented truth/recovery payload identity for that
/// repetition. The receipt digest is derived inside this value object; supplied dependency
/// digests remain trusted-adapter inputs and are not cryptographic proof of execution.
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

/// Approval state recorded for one recovery-profile registration ledger entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScientificRecoveryProfileRegistrationStatusV1 {
    /// The exact recovery profile was approved before execution evidence was issued.
    Approved,
    /// Approval has not reached a terminal decision.
    Pending,
    /// The proposed recovery profile was explicitly rejected.
    Rejected,
}

impl ScientificRecoveryProfileRegistrationStatusV1 {
    /// Stable wire name committed into chronology identity.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Pending => "pending",
            Self::Rejected => "rejected",
        }
    }
}

/// One immutable owner-ledger entry for a scientific-recovery execution artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScientificRecoveryExecutionLedgerEntryV1 {
    execution_artifact_sha256: String,
    ledger_entry_sha256: String,
    sequence: u64,
}

impl ScientificRecoveryExecutionLedgerEntryV1 {
    /// Construct one execution ledger entry.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when either digest is not canonical
    /// lowercase SHA-256 hexadecimal.
    pub fn new(
        execution_artifact_sha256: &str,
        ledger_entry_sha256: &str,
        sequence: u64,
    ) -> Result<Self, ValidationError> {
        if !is_canonical_sha256(execution_artifact_sha256)
            || !is_canonical_sha256(ledger_entry_sha256)
        {
            return Err(ValidationError::InvalidInput);
        }
        Ok(Self {
            execution_artifact_sha256: execution_artifact_sha256.to_owned(),
            ledger_entry_sha256: ledger_entry_sha256.to_owned(),
            sequence,
        })
    }

    /// Immutable execution-artifact identity recorded by this ledger entry.
    #[must_use]
    pub fn execution_artifact_sha256(&self) -> &str {
        &self.execution_artifact_sha256
    }

    /// Immutable owner-ledger entry identity.
    #[must_use]
    pub fn ledger_entry_sha256(&self) -> &str {
        &self.ledger_entry_sha256
    }

    /// Monotonic owner-ledger position for this execution entry.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}

/// Owner-ledger chronology binding profile approval before all represented executions.
///
/// The chronology commits an exact recovery profile to one immutable ledger and registration
/// entry, registration approval state and position, and one ledger entry for each planned
/// execution artifact. Execution positions may appear in any repetition order for parallel
/// work, but each position and ledger-entry identity must be unique and strictly later than
/// the registration position. The ledger identities and positions are trusted-adapter inputs;
/// this value object is not a signature, trusted timestamp, or external attestation verifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScientificRecoveryProfileChronologyV1 {
    profile_sha256: String,
    ledger_sha256: String,
    registration_entry_sha256: String,
    registration_sequence: u64,
    registration_status: ScientificRecoveryProfileRegistrationStatusV1,
    execution_entries: Vec<ScientificRecoveryExecutionLedgerEntryV1>,
    sha256: String,
}

impl ScientificRecoveryProfileChronologyV1 {
    /// Construct a versioned profile-registration and execution chronology.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when ledger identities are malformed,
    /// execution cardinality differs from the planned simulation denominator, an execution
    /// entry is at or before registration, or execution ledger positions/entry identities
    /// are duplicated.
    pub fn new(
        profile: &scientific_recovery::ScientificRecoveryProfileV1,
        ledger_sha256: &str,
        registration_entry_sha256: &str,
        registration_sequence: u64,
        registration_status: ScientificRecoveryProfileRegistrationStatusV1,
        execution_entries: &[ScientificRecoveryExecutionLedgerEntryV1],
    ) -> Result<Self, ValidationError> {
        if !is_canonical_sha256(ledger_sha256)
            || !is_canonical_sha256(registration_entry_sha256)
            || execution_entries.len() != profile.planned_replications()
        {
            return Err(ValidationError::InvalidInput);
        }

        let mut sequences = HashSet::with_capacity(execution_entries.len());
        let mut entry_ids = HashSet::with_capacity(execution_entries.len());
        for entry in execution_entries {
            if entry.sequence() <= registration_sequence
                || !sequences.insert(entry.sequence())
                || entry.ledger_entry_sha256() == registration_entry_sha256
                || !entry_ids.insert(entry.ledger_entry_sha256())
            {
                return Err(ValidationError::InvalidInput);
            }
        }

        let profile_sha256 = profile.sha256();
        let mut digest = Sha256::new();
        update_digest_field(&mut digest, PROFILE_CHRONOLOGY_SCHEMA.as_bytes());
        update_digest_field(&mut digest, profile_sha256.as_bytes());
        update_digest_field(&mut digest, ledger_sha256.as_bytes());
        update_digest_field(&mut digest, registration_entry_sha256.as_bytes());
        update_digest_field(&mut digest, &registration_sequence.to_le_bytes());
        update_digest_field(&mut digest, registration_status.wire_name().as_bytes());
        update_digest_usize(&mut digest, execution_entries.len());
        for (index, entry) in execution_entries.iter().enumerate() {
            update_digest_usize(&mut digest, index);
            update_digest_field(&mut digest, entry.execution_artifact_sha256().as_bytes());
            update_digest_field(&mut digest, entry.ledger_entry_sha256().as_bytes());
            update_digest_field(&mut digest, &entry.sequence().to_le_bytes());
        }

        Ok(Self {
            profile_sha256,
            ledger_sha256: ledger_sha256.to_owned(),
            registration_entry_sha256: registration_entry_sha256.to_owned(),
            registration_sequence,
            registration_status,
            execution_entries: execution_entries.to_vec(),
            sha256: hex_encode(&digest.finalize()),
        })
    }

    /// Exact recovery-profile identity registered in the owner ledger.
    #[must_use]
    pub fn profile_sha256(&self) -> &str {
        &self.profile_sha256
    }

    /// Immutable identity of the owner ledger used for chronology.
    #[must_use]
    pub fn ledger_sha256(&self) -> &str {
        &self.ledger_sha256
    }

    /// Immutable ledger-entry identity for profile registration.
    #[must_use]
    pub fn registration_entry_sha256(&self) -> &str {
        &self.registration_entry_sha256
    }

    /// Monotonic owner-ledger position of profile registration.
    #[must_use]
    pub const fn registration_sequence(&self) -> u64 {
        self.registration_sequence
    }

    /// Registration approval state.
    #[must_use]
    pub const fn registration_status(&self) -> ScientificRecoveryProfileRegistrationStatusV1 {
        self.registration_status
    }

    /// Ordered execution ledger entries aligned to scientific repetition indices.
    #[must_use]
    pub fn execution_entries(&self) -> &[ScientificRecoveryExecutionLedgerEntryV1] {
        &self.execution_entries
    }

    /// Domain-separated identity of the complete represented chronology.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Scientific authority bound to profile, chronology, exact-head receipt, payload, and execution identities.
#[derive(Clone, Debug, PartialEq)]
pub struct ScientificRecoveryPromotionV1 {
    inner: scientific_recovery::ScientificRecoveryPromotionV1,
    recovery_evidence_sha256: String,
    replication_provenance_sha256: String,
    profile_chronology_sha256: String,
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

    /// Domain-separated SHA-256 of profile registration and execution ledger chronology.
    #[must_use]
    pub fn profile_chronology_sha256(&self) -> &str {
        &self.profile_chronology_sha256
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

/// Promote a scientific claim and retain evaluated payload, chronology, and execution identities.
///
/// `replication_receipts[i]` must describe outer scientific repetition `i`, bind the exact
/// profile and seed-manifest identities, and carry the exact payload digest recomputed by this
/// owner from `truth_replications[i]` and `recovered_replications[i]`. The ordered seed-state
/// identities are reconstructed as a canonical manifest and must hash to the manifest identity
/// already committed by the profile; arbitrary unique seed-state substitutions therefore fail
/// closed rather than masquerading as planned Monte Carlo evidence.
///
/// `profile_chronology` must bind this exact profile to an approved immutable owner-ledger
/// registration entry whose sequence precedes every aligned execution-artifact ledger entry.
/// Caller wall-clock timestamps are not accepted as chronology authority.
///
/// The underlying Validation Evidence owner then performs the existing profile, replication,
/// numerical, exact-head receipt, and ADR 0014 authority checks. Successful authority retains
/// separate digests for the grouped scientific payload, ordered execution provenance, and
/// represented owner-ledger chronology.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for receipt count/order/profile/manifest/payload
/// mismatch, chronology/profile/execution mismatch, malformed represented inputs, or any
/// canonical scientific-recovery input refusal. Pending or rejected registration fails through
/// canonical claim-evidence errors. Other fail-closed errors propagate from the numerical,
/// profile, exact-head, and ADR 0014 gates.
#[allow(clippy::too_many_arguments)]
pub fn promote_scientific_recovery(
    candidate_head: &str,
    protected_head: &str,
    truth_replications: &[&[f64]],
    recovered_replications: &[&[f64]],
    profile: &scientific_recovery::ScientificRecoveryProfileV1,
    exact_head_receipt: &scientific_recovery::ScientificRecoveryExactHeadReceiptV1,
    replication_receipts: &[ScientificRecoveryReplicationReceiptV1],
    profile_chronology: &ScientificRecoveryProfileChronologyV1,
) -> Result<ScientificRecoveryPromotionV1, ValidationError> {
    let replication_provenance_sha256 = validate_replication_receipts(
        profile,
        truth_replications,
        recovered_replications,
        replication_receipts,
    )?;
    let profile_chronology_sha256 =
        validate_profile_chronology(profile, replication_receipts, profile_chronology)?;

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
        profile_chronology_sha256,
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

    let seed_states: Vec<&str> = replication_receipts
        .iter()
        .map(ScientificRecoveryReplicationReceiptV1::seed_state_sha256)
        .collect();
    let seed_manifest = ScientificRecoverySeedManifestV1::new(&seed_states)?;
    if seed_manifest.sha256() != profile.seed_manifest_sha256() {
        return Err(ValidationError::InvalidInput);
    }

    let profile_sha256 = profile.sha256();
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
            || receipt.seed_state_sha256() != seed_manifest.seed_state_sha256(index).unwrap_or("")
            || receipt.payload_sha256() != expected_payload
        {
            return Err(ValidationError::InvalidInput);
        }
        update_digest_usize(&mut digest, index);
        update_digest_field(&mut digest, receipt.receipt_sha256().as_bytes());
    }

    Ok(hex_encode(&digest.finalize()))
}

fn validate_profile_chronology(
    profile: &scientific_recovery::ScientificRecoveryProfileV1,
    replication_receipts: &[ScientificRecoveryReplicationReceiptV1],
    chronology: &ScientificRecoveryProfileChronologyV1,
) -> Result<String, ValidationError> {
    let profile_sha256 = profile.sha256();
    if chronology.profile_sha256() != profile_sha256.as_str() {
        return Err(ValidationError::InvalidInput);
    }

    match chronology.registration_status() {
        ScientificRecoveryProfileRegistrationStatusV1::Approved => {}
        ScientificRecoveryProfileRegistrationStatusV1::Pending => {
            return Err(ValidationError::ClaimQueuedEvidence);
        }
        ScientificRecoveryProfileRegistrationStatusV1::Rejected => {
            return Err(ValidationError::ClaimEvidenceFailed);
        }
    }

    if chronology
        .execution_entries()
        .iter()
        .zip(replication_receipts)
        .any(|(entry, receipt)| {
            entry.execution_artifact_sha256() != receipt.execution_artifact_sha256()
        })
    {
        return Err(ValidationError::InvalidInput);
    }

    Ok(chronology.sha256().to_owned())
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
