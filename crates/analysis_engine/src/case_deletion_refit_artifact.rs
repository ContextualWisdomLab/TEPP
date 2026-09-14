//! Digest-bound exhaustive case-deletion refit as an analysis-run profile.

use corpus_split::cutoff_eligible;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, CaseDeletionDocument, CaseDeletionRefitter, ExhaustiveCaseDeletionError,
    MAX_EVIDENCE_UNITS, fit_exhaustive_case_deletion, format_digest, require_receipt_identity,
    valid_identifier,
};

/// Versioned schema for a completed exhaustive case-deletion artifact.
pub const CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION: &str = "tepp.case_deletion_refit.v1";
/// Model contract required by the exhaustive case-deletion execution path.
pub const CASE_DELETION_REFIT_MODEL_CONTRACT_VERSION: &str = "case_deletion_refit_v1";
/// Analysis-run output profile required for an exhaustive case-deletion artifact.
pub const CASE_DELETION_REFIT_OUTPUT_PROFILE: &str = "case_deletion_refit_v1";
/// Maximum accepted case-deletion artifact JSON size.
pub const CASE_DELETION_REFIT_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const MAX_CASE_DELETION_RETAINED_IDENTITIES: usize = 256 * 255;
const CASE_DELETION_REFIT_INFERENCE_STATUS: &str =
    "exhaustive_actual_deletion_not_reweighting_approx";

/// Cutoff-safe exhaustive case-deletion payload bound to an existing fitter.
///
/// Snapshot and availability provenance remain parallel to the scientific
/// [`CaseDeletionDocument`] so the reusable fitter contract does not acquire
/// application-layer historical-admission fields.
#[derive(Clone, Debug)]
pub struct CaseDeletionRefitInput<'a, D, F> {
    documents: &'a [CaseDeletionDocument<D>],
    snapshot_ids: &'a [String],
    available_times: &'a [AvailableTime],
    seed_domain_base: &'a str,
    fitter: &'a F,
}

impl<'a, D, F> CaseDeletionRefitInput<'a, D, F> {
    /// Construct a case-deletion payload with explicit per-document provenance.
    #[must_use]
    pub const fn new(
        documents: &'a [CaseDeletionDocument<D>],
        snapshot_ids: &'a [String],
        available_times: &'a [AvailableTime],
        seed_domain_base: &'a str,
        fitter: &'a F,
    ) -> Self {
        Self {
            documents,
            snapshot_ids,
            available_times,
            seed_domain_base,
            fitter,
        }
    }

    /// Borrow the candidate scientific documents.
    #[must_use]
    pub const fn documents(&self) -> &'a [CaseDeletionDocument<D>] {
        self.documents
    }

    /// Borrow immutable source snapshot identities aligned to documents.
    #[must_use]
    pub const fn snapshot_ids(&self) -> &'a [String] {
        self.snapshot_ids
    }

    /// Borrow evidence-availability times aligned to documents.
    #[must_use]
    pub const fn available_times(&self) -> &'a [AvailableTime] {
        self.available_times
    }

    /// Return the seed-domain base used to separate full and deleted fits.
    #[must_use]
    pub const fn seed_domain_base(&self) -> &'a str {
        self.seed_domain_base
    }

    /// Borrow the scientific fitter invoked on each actual corpus.
    #[must_use]
    pub const fn fitter(&self) -> &'a F {
        self.fitter
    }
}

/// Completed, bounded exhaustive case-deletion counts for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseDeletionRefitArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the run.
    pub knowledge_cutoff: String,
    /// Number of admitted documents.
    pub document_count: u64,
    /// Number of actual one-document deletion refits.
    pub deletion_refit_count: u64,
    /// Number of independent seed domains (full fit plus each deletion).
    pub independent_seed_domain_count: u64,
    /// Domain-separated randomness identity for the full-data fit.
    pub full_seed_domain: String,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl CaseDeletionRefitArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidCaseDeletionRefitArtifact`] when
    /// the schema, identifiers, counts, resource budget, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > CASE_DELETION_REFIT_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidCaseDeletionRefitArtifact)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize canonical validated artifact JSON.
    ///
    /// Valid identifiers, strict timestamp syntax, and the case-deletion
    /// resource budget make canonical output smaller than
    /// [`CASE_DELETION_REFIT_ARTIFACT_BYTE_LIMIT`]. Untrusted input remains
    /// capped by [`Self::from_json`].
    ///
    /// # Errors
    ///
    /// Returns a typed validation or serialization failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)
    }

    /// Return the lowercase SHA-256 digest of canonical artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation or serialization failure.
    pub fn sha256(&self) -> Result<String, AnalysisEngineError> {
        self.to_json()
            .map(|json| format_digest(Sha256::digest(json.into_bytes())))
    }

    fn validate(&self) -> Result<(), AnalysisEngineError> {
        let document_count = usize::try_from(self.document_count).ok();
        if self.schema_version != CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || document_count.is_none()
            || document_count == Some(0)
            || document_count == Some(1)
            || !within_case_deletion_resource_budget(document_count.unwrap_or_default())
            || self.deletion_refit_count != self.document_count
            || self.independent_seed_domain_count
                != self
                    .document_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::InvalidCaseDeletionRefitArtifact)?
            || !valid_identifier(&self.full_seed_domain)
            || self.inference_status != CASE_DELETION_REFIT_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidCaseDeletionRefitArtifact);
        }
        Ok(())
    }
}

/// One completed exhaustive case-deletion artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct CaseDeletionRefitExecution {
    /// Digest-bound completed case-deletion artifact.
    pub artifact: CaseDeletionRefitArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute exhaustive actual case-deletion as one analysis-run profile.
///
/// The executor invokes [`fit_exhaustive_case_deletion`] and does not
/// reimplement leave-one-out fitting, reweighting, or a diagonal
/// approximation. Evidence availability is admitted before duplicate/scientific
/// fitting, so evidence not yet available at the cutoff cannot alter a
/// historical replay. Raw posteriors stay with the scientific fitter; the
/// operator artifact carries only bounded counts and seed-domain identity.
/// This is not a Bayesian sampler and not GPU execution.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, malformed or
/// unavailable provenance, resource limit, fitter refusal, or invalid artifact
/// error.
pub fn execute_case_deletion_refit_run<D, P, F>(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &CaseDeletionRefitInput<'_, D, F>,
    completed_at: impl Into<String>,
) -> Result<CaseDeletionRefitExecution, AnalysisEngineError>
where
    D: Clone,
    F: CaseDeletionRefitter<D, P>,
{
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != CASE_DELETION_REFIT_MODEL_CONTRACT_VERSION
        || request.output_profile != CASE_DELETION_REFIT_OUTPUT_PROFILE
        || input.documents().len() != input.snapshot_ids().len()
        || input.documents().len() != input.available_times().len()
        || !valid_identifier(input.seed_domain_base())
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if input.documents().len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let mut admitted_documents = Vec::with_capacity(input.documents().len().min(256));
    for ((document, document_snapshot_id), available_time) in input
        .documents()
        .iter()
        .zip(input.snapshot_ids())
        .zip(input.available_times())
    {
        if document_snapshot_id != snapshot_id {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        if !cutoff_eligible(available_time, &knowledge_cutoff) {
            continue;
        }
        let next_document_count = admitted_documents
            .len()
            .checked_add(1)
            .ok_or(AnalysisEngineError::LimitExceeded)?;
        if !within_case_deletion_resource_budget(next_document_count) {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        admitted_documents.push(document.clone());
    }

    let fits = fit_exhaustive_case_deletion(
        &admitted_documents,
        input.seed_domain_base(),
        input.fitter(),
    )
    .map_err(|error| match error {
        ExhaustiveCaseDeletionError::InvalidInput => AnalysisEngineError::InvalidEvidence,
        ExhaustiveCaseDeletionError::Fit(_) => AnalysisEngineError::CaseDeletionFitFailure,
    })?;
    let document_count = u64::try_from(admitted_documents.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let deletion_refit_count = u64::try_from(fits.deletion_refits.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let independent_seed_domain_count = document_count
        .checked_add(1)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    let artifact = CaseDeletionRefitArtifact {
        schema_version: CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        document_count,
        deletion_refit_count,
        independent_seed_domain_count,
        full_seed_domain: fits.full_seed_domain,
        inference_status: CASE_DELETION_REFIT_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new("case_deletion_refit", document_count, 3, "validated")?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("case_deletion_refit_artifact_{}", &digest[..16]),
        digest,
        CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(CaseDeletionRefitExecution {
        artifact,
        terminal_result,
    })
}

fn within_case_deletion_resource_budget(document_count: usize) -> bool {
    document_count
        .checked_mul(document_count.saturating_sub(1))
        .is_some_and(|retained_identities| {
            retained_identities <= MAX_CASE_DELETION_RETAINED_IDENTITIES
        })
}

#[cfg(test)]
mod tests {
    use super::{
        CASE_DELETION_REFIT_ARTIFACT_BYTE_LIMIT, CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION,
        CASE_DELETION_REFIT_INFERENCE_STATUS, CaseDeletionRefitArtifact, CaseDeletionRefitInput,
        within_case_deletion_resource_budget,
    };
    use crate::{AnalysisEngineError, CaseDeletionDocument};
    use temporal_core::AvailableTime;

    struct UnusedFitter;

    fn artifact() -> CaseDeletionRefitArtifact {
        CaseDeletionRefitArtifact {
            schema_version: CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            document_count: 3,
            deletion_refit_count: 3,
            independent_seed_domain_count: 4,
            full_seed_domain: "topic-model-run:full".into(),
            inference_status: CASE_DELETION_REFIT_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &CaseDeletionRefitArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidCaseDeletionRefitArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            CaseDeletionRefitArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            CaseDeletionRefitArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidCaseDeletionRefitArtifact)
        );
        assert_eq!(
            CaseDeletionRefitArtifact::from_json(
                &"x".repeat(CASE_DELETION_REFIT_ARTIFACT_BYTE_LIMIT + 1)
            ),
            Err(AnalysisEngineError::LimitExceeded)
        );
    }

    #[test]
    fn maximal_valid_artifact_stays_below_inbound_wire_cap() {
        let identifier = "\\".repeat(256);
        let mut artifact = artifact();
        artifact.run_id = identifier.clone();
        artifact.snapshot_id = identifier.clone();
        artifact.full_seed_domain = identifier;
        artifact.document_count = 256;
        artifact.deletion_refit_count = 256;
        artifact.independent_seed_domain_count = 257;
        let payload = artifact.to_json().expect("maximal artifact");
        assert!(payload.len() < CASE_DELETION_REFIT_ARTIFACT_BYTE_LIMIT);
        assert_eq!(
            CaseDeletionRefitArtifact::from_json(&payload),
            Ok(artifact)
        );
    }

    #[test]
    fn artifact_metadata_tampering_fails_closed() {
        let artifact = artifact();
        let invalid_artifacts = [
            {
                let mut value = artifact.clone();
                value.schema_version.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.run_id.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.snapshot_id.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.knowledge_cutoff = "invalid".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.document_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.document_count = 257;
                value.deletion_refit_count = 257;
                value.independent_seed_domain_count = 258;
                value
            },
            {
                let mut value = artifact.clone();
                value.deletion_refit_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.independent_seed_domain_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.full_seed_domain.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.inference_status.clear();
                value
            },
        ];
        for invalid in invalid_artifacts {
            assert_invalid(&invalid);
        }
    }

    #[test]
    fn resource_budget_matches_quadratic_retained_identity_storage() {
        assert!(within_case_deletion_resource_budget(256));
        assert!(!within_case_deletion_resource_budget(257));
    }

    #[test]
    fn input_accessors_expose_documents_and_provenance() {
        let documents = [CaseDeletionDocument {
            document_id: "document-a".into(),
            evidence: 1.0,
        }];
        let snapshot_ids = ["snapshot-1".to_owned()];
        let available_times =
            [AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available")];
        let fitter = UnusedFitter;
        let input = CaseDeletionRefitInput::new(
            &documents,
            &snapshot_ids,
            &available_times,
            "topic-model-run",
            &fitter,
        );
        assert_eq!(input.documents(), &documents);
        assert_eq!(input.snapshot_ids(), &snapshot_ids);
        assert_eq!(input.available_times(), &available_times);
        assert_eq!(input.seed_domain_base(), "topic-model-run");
        let _ = input.fitter();
    }
}
