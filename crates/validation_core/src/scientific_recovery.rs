//! Scientific recovery promotion over independent simulation replications.
//!
//! The Validation Evidence boundary treats one outer element as one independent
//! simulation repetition. Temporal states, clustered units, cross-classified
//! memberships, and other correlated coordinates stay inside that repetition and
//! are reduced to one replication-level RMSE before Monte Carlo uncertainty is
//! evaluated. This prevents within-replication rows from masquerading as `n_sim`.

use crate::claim;
use crate::{
    ClaimAuthority, ClaimEvidence, ClaimEvidenceKind, PromotedClaim, PromotionRequest,
    ValidationError, root_mean_square_error,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const PROFILE_SCHEMA: &str = "tepp.scientific_recovery_profile.v1";

/// Failure policy bound into a scientific recovery profile.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificRecoveryFailurePolicyV1 {
    /// Every planned independent replication must have a valid recovery result.
    RequireAllPlannedRecovered,
}

impl ScientificRecoveryFailurePolicyV1 {
    /// Stable wire name used by the profile digest contract.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::RequireAllPlannedRecovered => "require_all_planned_recovered",
        }
    }
}

/// Terminal state of one exact-head test receipt offered for scientific promotion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScientificRecoveryExactHeadReceiptStatusV1 {
    /// The exact-head test receipt is terminal and passing.
    Passed,
    /// The exact-head test receipt is terminal and failing.
    Failed,
    /// The exact-head test run has not reached a terminal result.
    Queued,
    /// A required exact-head test was skipped or ignored.
    Skipped,
}

/// Immutable identity for the exact-head test evidence used by scientific recovery.
///
/// The receipt SHA-256 is an opaque artifact identity supplied by a trusted
/// repository/CI adapter. This value object does not infer test truth from the
/// digest contents; it binds the adapter's terminal state and immutable receipt
/// identity to the exact Git commit that was tested.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScientificRecoveryExactHeadReceiptV1 {
    head: [u8; 20],
    receipt_sha256: String,
    status: ScientificRecoveryExactHeadReceiptStatusV1,
}

impl ScientificRecoveryExactHeadReceiptV1 {
    /// Construct one exact-head test receipt identity.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when `head` is not an exact Git
    /// commit SHA or `receipt_sha256` is not canonical lowercase SHA-256 hex.
    pub fn new(
        head: &str,
        receipt_sha256: &str,
        status: ScientificRecoveryExactHeadReceiptStatusV1,
    ) -> Result<Self, ValidationError> {
        if !is_canonical_sha256(receipt_sha256) {
            return Err(ValidationError::InvalidInput);
        }
        Ok(Self {
            head: claim::parse_commit_head(head)?,
            receipt_sha256: receipt_sha256.to_owned(),
            status,
        })
    }

    /// Exact Git commit tested by this receipt.
    #[must_use]
    pub const fn head(&self) -> [u8; 20] {
        self.head
    }

    /// Canonical SHA-256 identity of the immutable CI/test receipt artifact.
    #[must_use]
    pub fn receipt_sha256(&self) -> &str {
        &self.receipt_sha256
    }

    /// Terminal receipt state supplied by the trusted adapter.
    #[must_use]
    pub const fn status(&self) -> ScientificRecoveryExactHeadReceiptStatusV1 {
        self.status
    }
}

/// Versioned scientific recovery design and acceptance policy.
///
/// The value object binds the simulation denominator and practical acceptance
/// settings to immutable digests for the DGP/configuration, seed stream, estimand,
/// and within-replication state composition. Its SHA-256 is derived from canonical
/// represented fields; callers cannot provide a detached profile digest.
///
/// This object proves content identity only. Publication chronology remains an
/// integration responsibility: a buyer path must separately prove that this exact
/// profile digest was persisted/approved before the simulation execution began.
#[derive(Clone, Debug, PartialEq)]
pub struct ScientificRecoveryProfileV1 {
    planned_replications: usize,
    max_rmse: f64,
    se_multiplier: f64,
    dgp_sha256: String,
    seed_manifest_sha256: String,
    estimand_sha256: String,
    state_composition_sha256: String,
    failure_policy: ScientificRecoveryFailurePolicyV1,
}

impl ScientificRecoveryProfileV1 {
    /// Construct a validated recovery profile.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidConfiguration`] when the planned
    /// replication count, practical RMSE target, or uncertainty multiplier is
    /// invalid. The uncertainty multiplier accepts canonical positive zero but
    /// rejects IEEE 754 negative zero so one scientific policy cannot acquire two
    /// profile identities solely from the zero sign bit. Returns
    /// [`ValidationError::InvalidInput`] when any dependency digest is not
    /// canonical lowercase SHA-256 hexadecimal.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        planned_replications: usize,
        max_rmse: f64,
        se_multiplier: f64,
        dgp_sha256: &str,
        seed_manifest_sha256: &str,
        estimand_sha256: &str,
        state_composition_sha256: &str,
        failure_policy: ScientificRecoveryFailurePolicyV1,
    ) -> Result<Self, ValidationError> {
        if planned_replications < 2
            || !max_rmse.is_finite()
            || max_rmse <= 0.0
            || !se_multiplier.is_finite()
            || se_multiplier < 0.0
            || (se_multiplier == 0.0 && se_multiplier.is_sign_negative())
        {
            return Err(ValidationError::InvalidConfiguration);
        }
        for digest in [
            dgp_sha256,
            seed_manifest_sha256,
            estimand_sha256,
            state_composition_sha256,
        ] {
            if !is_canonical_sha256(digest) {
                return Err(ValidationError::InvalidInput);
            }
        }
        Ok(Self {
            planned_replications,
            max_rmse,
            se_multiplier,
            dgp_sha256: dgp_sha256.to_owned(),
            seed_manifest_sha256: seed_manifest_sha256.to_owned(),
            estimand_sha256: estimand_sha256.to_owned(),
            state_composition_sha256: state_composition_sha256.to_owned(),
            failure_policy,
        })
    }

    /// Planned independent simulation denominator.
    #[must_use]
    pub const fn planned_replications(&self) -> usize {
        self.planned_replications
    }

    /// Claim-specific practical RMSE target.
    #[must_use]
    pub const fn max_rmse(&self) -> f64 {
        self.max_rmse
    }

    /// Monte Carlo uncertainty multiplier.
    #[must_use]
    pub const fn se_multiplier(&self) -> f64 {
        self.se_multiplier
    }

    /// Immutable DGP/configuration digest.
    #[must_use]
    pub fn dgp_sha256(&self) -> &str {
        &self.dgp_sha256
    }

    /// Immutable seed-stream or seed-manifest digest.
    #[must_use]
    pub fn seed_manifest_sha256(&self) -> &str {
        &self.seed_manifest_sha256
    }

    /// Immutable estimand-contract digest.
    #[must_use]
    pub fn estimand_sha256(&self) -> &str {
        &self.estimand_sha256
    }

    /// Immutable within-replication state-composition digest.
    #[must_use]
    pub fn state_composition_sha256(&self) -> &str {
        &self.state_composition_sha256
    }

    /// Explicit estimator-failure policy.
    #[must_use]
    pub const fn failure_policy(&self) -> ScientificRecoveryFailurePolicyV1 {
        self.failure_policy
    }

    /// Deterministic domain-separated profile identity.
    #[must_use]
    pub fn sha256(&self) -> String {
        let mut digest = Sha256::new();
        update_digest_field(&mut digest, PROFILE_SCHEMA.as_bytes());
        update_digest_field(&mut digest, &self.planned_replications.to_le_bytes());
        update_digest_field(&mut digest, &self.max_rmse.to_bits().to_le_bytes());
        update_digest_field(&mut digest, &self.se_multiplier.to_bits().to_le_bytes());
        update_digest_field(&mut digest, self.dgp_sha256.as_bytes());
        update_digest_field(&mut digest, self.seed_manifest_sha256.as_bytes());
        update_digest_field(&mut digest, self.estimand_sha256.as_bytes());
        update_digest_field(&mut digest, self.state_composition_sha256.as_bytes());
        update_digest_field(&mut digest, self.failure_policy.wire_name().as_bytes());
        hex_encode(&digest.finalize())
    }

    /// Serialize the validated profile to versioned JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when serialization fails.
    pub fn to_json(&self) -> Result<String, ValidationError> {
        serde_json::to_string(self).map_err(|_| ValidationError::InvalidInput)
    }

    /// Parse and validate one versioned recovery profile.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] for malformed, unsupported, or
    /// non-canonical profile JSON, and propagates invalid profile configuration.
    pub fn from_json(value: &str) -> Result<Self, ValidationError> {
        serde_json::from_str(value).map_err(|_| ValidationError::InvalidInput)
    }
}

impl Serialize for ScientificRecoveryProfileV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("ScientificRecoveryProfileV1", 9)?;
        state.serialize_field("schema", PROFILE_SCHEMA)?;
        state.serialize_field("planned_replications", &self.planned_replications)?;
        state.serialize_field("max_rmse", &self.max_rmse)?;
        state.serialize_field("se_multiplier", &self.se_multiplier)?;
        state.serialize_field("dgp_sha256", &self.dgp_sha256)?;
        state.serialize_field("seed_manifest_sha256", &self.seed_manifest_sha256)?;
        state.serialize_field("estimand_sha256", &self.estimand_sha256)?;
        state.serialize_field(
            "state_composition_sha256",
            &self.state_composition_sha256,
        )?;
        state.serialize_field("failure_policy", &self.failure_policy)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ScientificRecoveryProfileV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            schema: String,
            planned_replications: usize,
            max_rmse: f64,
            se_multiplier: f64,
            dgp_sha256: String,
            seed_manifest_sha256: String,
            estimand_sha256: String,
            state_composition_sha256: String,
            failure_policy: ScientificRecoveryFailurePolicyV1,
        }

        let raw = Raw::deserialize(deserializer)?;
        if raw.schema != PROFILE_SCHEMA {
            return Err(serde::de::Error::custom(
                "unsupported scientific recovery profile schema",
            ));
        }
        Self::new(
            raw.planned_replications,
            raw.max_rmse,
            raw.se_multiplier,
            &raw.dgp_sha256,
            &raw.seed_manifest_sha256,
            &raw.estimand_sha256,
            &raw.state_composition_sha256,
            raw.failure_policy,
        )
        .map_err(serde::de::Error::custom)
    }
}

/// Scientific authority bound to one exact recovery-profile and test-receipt identity.
#[derive(Clone, Debug, PartialEq)]
pub struct ScientificRecoveryPromotionV1 {
    claim: PromotedClaim,
    profile_sha256: String,
    exact_head_receipt_sha256: String,
}

impl ScientificRecoveryPromotionV1 {
    /// Promoted scientific claim.
    #[must_use]
    pub const fn claim(&self) -> PromotedClaim {
        self.claim
    }

    /// Exact SHA-256 of the versioned recovery profile used for promotion.
    #[must_use]
    pub fn profile_sha256(&self) -> &str {
        &self.profile_sha256
    }

    /// Exact SHA-256 of the immutable exact-head test receipt used for promotion.
    #[must_use]
    pub fn exact_head_receipt_sha256(&self) -> &str {
        &self.exact_head_receipt_sha256
    }
}

/// Promote a scientific claim from grouped recovery replications under one profile.
///
/// The outer elements are independent simulation repetitions. Coordinates within
/// each repetition are reduced to one replication-level RMSE before Monte Carlo
/// uncertainty is evaluated. The profile owns the denominator, practical target,
/// uncertainty multiplier, dependency provenance, and explicit failure policy.
///
/// `exact_head_receipt` binds exact-head test evidence to the tested Git commit and
/// an immutable receipt identity. Only a passing receipt for `candidate_head` is
/// converted into internal [`ClaimEvidenceKind::ExactHeadTests`] evidence. A
/// passing [`ClaimEvidenceKind::ScientificRecovery`] item is appended only after
/// the numerical/profile gate succeeds, and final authority is minted through the
/// canonical ADR 0014 claim gate.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when the presented outer denominator
/// differs from the profile or an inner truth/recovery pair is invalid. Recovery
/// rejection, head mismatch, predecessor/failed/queued/skipped exact-head receipt,
/// and other canonical claim-gate errors fail closed.
pub fn promote_scientific_recovery(
    candidate_head: &str,
    protected_head: &str,
    truth_replications: &[&[f64]],
    recovered_replications: &[&[f64]],
    profile: &ScientificRecoveryProfileV1,
    exact_head_receipt: &ScientificRecoveryExactHeadReceiptV1,
) -> Result<ScientificRecoveryPromotionV1, ValidationError> {
    let planned_replications = profile.planned_replications();
    if truth_replications.len() != planned_replications
        || recovered_replications.len() != planned_replications
        || truth_replications.len() != recovered_replications.len()
    {
        return Err(ValidationError::InvalidInput);
    }

    match profile.failure_policy() {
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered => {}
    }

    let replication_rmse: Result<Vec<_>, _> = truth_replications
        .iter()
        .zip(recovered_replications)
        .map(|(truth, recovered)| root_mean_square_error(truth, recovered))
        .collect();
    let replication_rmse = replication_rmse?;
    let zero_truth = vec![0.0; replication_rmse.len()];

    // This private numerical gate proves the recovery criterion only. Its
    // historical return value is deliberately not exposed as final authority;
    // ADR 0014 authority below is recomposed through `promote_claim` with the
    // exact-head receipt plus the computed recovery item.
    claim::promote_scientific_recovery(
        candidate_head,
        protected_head,
        &zero_truth,
        &replication_rmse,
        profile.max_rmse(),
        profile.se_multiplier(),
    )?;

    let candidate = claim::parse_commit_head(candidate_head)?;
    if exact_head_receipt.head() != candidate {
        return Err(ValidationError::ClaimPredecessorHead);
    }
    match exact_head_receipt.status() {
        ScientificRecoveryExactHeadReceiptStatusV1::Passed => {}
        ScientificRecoveryExactHeadReceiptStatusV1::Failed => {
            return Err(ValidationError::ClaimEvidenceFailed);
        }
        ScientificRecoveryExactHeadReceiptStatusV1::Queued => {
            return Err(ValidationError::ClaimQueuedEvidence);
        }
        ScientificRecoveryExactHeadReceiptStatusV1::Skipped => {
            return Err(ValidationError::ClaimSkippedRequired);
        }
    }

    let evidence = [
        ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true),
        ClaimEvidence::new(ClaimEvidenceKind::ScientificRecovery, true),
    ];
    let request = PromotionRequest::new(
        ClaimAuthority::ScientificallySupported,
        candidate_head,
        protected_head,
        &evidence,
    )?;
    let claim = claim::promote_claim(&request)?;

    Ok(ScientificRecoveryPromotionV1 {
        claim,
        profile_sha256: profile.sha256(),
        exact_head_receipt_sha256: exact_head_receipt.receipt_sha256().to_owned(),
    })
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
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
