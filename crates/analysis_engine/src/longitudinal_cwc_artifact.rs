//! Digest-bound CWC within/between composition as an analysis-run profile.

use psychometric_core::{
    CausalHeuristic, ClusteredScore, PsychometricError, claim_causal_effect,
    recover_cluster_mean_within_between_slopes,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, format_digest, require_receipt_identity,
    valid_identifier,
};

/// Versioned schema for a completed longitudinal CWC artifact.
pub const LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION: &str = "tepp.longitudinal_cwc.v1";
/// Model contract required by the CWC composition execution path.
pub const LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION: &str = "longitudinal_cwc_v1";
/// Analysis-run output profile required for a longitudinal CWC artifact.
pub const LONGITUDINAL_CWC_OUTPUT_PROFILE: &str = "longitudinal_cwc_v1";
/// Maximum canonical artifact JSON size.
pub const LONGITUDINAL_CWC_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const LONGITUDINAL_CWC_INFERENCE_STATUS: &str = "composed_cwc_slopes_not_causal";

/// One already-mapped clustered score offered to a cutoff-safe CWC run.
#[derive(Clone, Debug, PartialEq)]
pub struct LongitudinalClusterScore {
    evidence_id: String,
    snapshot_id: String,
    cluster_key: u64,
    predictor: f64,
    outcome: f64,
    available_time: AvailableTime,
}

impl LongitudinalClusterScore {
    /// Bind one clustered predictor–outcome pair to immutable evidence, snapshot, and availability provenance.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when either identity is invalid or either
    /// coordinate is non-finite.
    pub fn new(
        evidence_id: impl Into<String>,
        snapshot_id: impl Into<String>,
        cluster_key: u64,
        predictor: f64,
        outcome: f64,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let evidence_id = evidence_id.into();
        let snapshot_id = snapshot_id.into();
        if !valid_identifier(&evidence_id)
            || !valid_identifier(&snapshot_id)
            || !predictor.is_finite()
            || !outcome.is_finite()
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            evidence_id,
            snapshot_id,
            cluster_key,
            predictor,
            outcome,
            available_time,
        })
    }

    /// Return the opaque immutable evidence identity.
    #[must_use]
    pub fn evidence_id(&self) -> &str {
        &self.evidence_id
    }

    /// Return the immutable source snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return the cluster identity.
    #[must_use]
    pub const fn cluster_key(&self) -> u64 {
        self.cluster_key
    }

    /// Return the already-mapped predictor.
    #[must_use]
    pub const fn predictor(&self) -> f64 {
        self.predictor
    }

    /// Return the already-mapped outcome.
    #[must_use]
    pub const fn outcome(&self) -> f64 {
        self.outcome
    }

    /// Return the availability clock used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded CWC composition consumed by analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LongitudinalCwcArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the composition.
    pub knowledge_cutoff: String,
    /// Eligible clustered rows after cutoff.
    pub row_count: u64,
    /// Distinct clusters among eligible rows.
    pub cluster_count: u64,
    /// Rows excluded because availability was after the cutoff.
    pub excluded_after_cutoff_count: u64,
    /// Within-cluster OLS slope after CWC.
    pub within_slope: f64,
    /// Between-cluster OLS slope of cluster means.
    pub between_slope: f64,
    /// CWC contextual effect `between − within`.
    pub contextual_effect: f64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl LongitudinalCwcArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidLongitudinalCwcArtifact`] when the
    /// schema, identifiers, counts, slopes, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > LONGITUDINAL_CWC_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidLongitudinalCwcArtifact)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize canonical validated artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation or serialization failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        Ok(payload)
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
        let max_rows = u64::try_from(MAX_EVIDENCE_UNITS)
            .map_err(|_| AnalysisEngineError::InvalidLongitudinalCwcArtifact)?;
        let total_rows = self
            .row_count
            .checked_add(self.excluded_after_cutoff_count)
            .ok_or(AnalysisEngineError::InvalidLongitudinalCwcArtifact)?;
        let expected_contextual_effect = self.between_slope - self.within_slope;
        if self.schema_version != LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.row_count < 2
            || self.row_count > max_rows
            || self.cluster_count < 2
            || self.cluster_count > self.row_count
            || total_rows > max_rows
            || !self.within_slope.is_finite()
            || !self.between_slope.is_finite()
            || !self.contextual_effect.is_finite()
            || !expected_contextual_effect.is_finite()
            || self.contextual_effect.to_bits() != expected_contextual_effect.to_bits()
            || self.inference_status != LONGITUDINAL_CWC_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidLongitudinalCwcArtifact);
        }
        Ok(())
    }
}

/// One completed CWC artifact and its request-bound terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct LongitudinalCwcExecution {
    /// Digest-bound completed composition artifact.
    pub artifact: LongitudinalCwcArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

struct EligibleCwcRows {
    scores: Vec<ClusteredScore>,
    excluded_after_cutoff_count: u64,
}

fn admit_scores_at_cutoff(
    scores: &[LongitudinalClusterScore],
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<EligibleCwcRows, AnalysisEngineError> {
    if scores.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    let mut evidence_ids = BTreeSet::new();
    let mut eligible = Vec::new();
    let mut excluded_after_cutoff_count = 0_u64;
    for score in scores {
        if score.snapshot_id != snapshot_id {
            return Err(AnalysisEngineError::SnapshotMismatch);
        }
        if score.available_time.instant() <= knowledge_cutoff.instant() {
            if !evidence_ids.insert(score.evidence_id.as_str()) {
                return Err(AnalysisEngineError::DuplicateEvidence);
            }
            eligible.push(ClusteredScore {
                cluster_key: score.cluster_key,
                predictor: score.predictor,
                outcome: score.outcome,
            });
        } else {
            excluded_after_cutoff_count += 1;
        }
    }
    if eligible.is_empty() {
        return Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput,
        ));
    }
    Ok(EligibleCwcRows {
        scores: eligible,
        excluded_after_cutoff_count,
    })
}

fn require_causal_refusal(
    result: Result<(), PsychometricError>,
) -> Result<(), AnalysisEngineError> {
    match result {
        Err(PsychometricError::CausalUnderidentified) => Ok(()),
        Ok(()) | Err(_) => Err(AnalysisEngineError::InvalidEvidence),
    }
}

#[expect(
    clippy::missing_panics_doc,
    reason = "bounded summary constants cannot fail"
)]
/// Execute cutoff-safe CWC within/between composition as one analysis-run profile.
///
/// The caller supplies already-mapped clustered coordinates. Each row carries its immutable
/// evidence identity, source snapshot, and availability provenance. Future-unavailable rows are
/// censored before evidence-identity admission, so they cannot alter a historical replay;
/// duplicate identities among cutoff-visible evidence fail closed before scientific composition.
/// This executor does not invent an ESEM/DSEM estimator, persist rows, or treat the recovered
/// slopes as a causal effect.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, duplicate-visible-evidence refusal,
/// psychometric recovery failure, or invalid artifact error.
pub fn execute_longitudinal_cwc_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    scores: &[LongitudinalClusterScore],
    completed_at: impl Into<String>,
) -> Result<LongitudinalCwcExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION
        || request.output_profile != LONGITUDINAL_CWC_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let eligible = admit_scores_at_cutoff(scores, snapshot_id, knowledge_cutoff)?;
    let slopes = recover_cluster_mean_within_between_slopes(&eligible.scores)?;
    require_causal_refusal(claim_causal_effect(CausalHeuristic::TemporalPrecedence))?;

    let mut clusters = BTreeSet::new();
    for score in &eligible.scores {
        clusters.insert(score.cluster_key);
    }
    let row_count = u64::try_from(eligible.scores.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let cluster_count =
        u64::try_from(clusters.len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let artifact = LongitudinalCwcArtifact {
        schema_version: LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        row_count,
        cluster_count,
        excluded_after_cutoff_count: eligible.excluded_after_cutoff_count,
        within_slope: slopes.within_slope,
        between_slope: slopes.between_slope,
        contextual_effect: slopes.contextual_effect,
        inference_status: LONGITUDINAL_CWC_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new("longitudinal_cwc", row_count, 3, "validated")
        .expect("bounded longitudinal CWC summary constants are valid");
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("longitudinal_cwc_artifact_{}", &digest[..16]),
        digest,
        LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(LongitudinalCwcExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        LONGITUDINAL_CWC_ARTIFACT_BYTE_LIMIT, LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION,
        LONGITUDINAL_CWC_INFERENCE_STATUS, LongitudinalCwcArtifact, require_causal_refusal,
    };
    use crate::AnalysisEngineError;
    use psychometric_core::PsychometricError;

    fn artifact() -> LongitudinalCwcArtifact {
        LongitudinalCwcArtifact {
            schema_version: LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            row_count: 4,
            cluster_count: 2,
            excluded_after_cutoff_count: 0,
            within_slope: 0.5,
            between_slope: 2.0,
            contextual_effect: 1.5,
            inference_status: LONGITUDINAL_CWC_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &LongitudinalCwcArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidLongitudinalCwcArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            LongitudinalCwcArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            LongitudinalCwcArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidLongitudinalCwcArtifact)
        );
        assert_eq!(
            LongitudinalCwcArtifact::from_json(
                &"x".repeat(LONGITUDINAL_CWC_ARTIFACT_BYTE_LIMIT + 1)
            ),
            Err(AnalysisEngineError::LimitExceeded)
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
                value.row_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.cluster_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.cluster_count = 5;
                value
            },
            {
                let mut value = artifact.clone();
                value.within_slope = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.between_slope = f64::INFINITY;
                value
            },
            {
                let mut value = artifact.clone();
                value.contextual_effect = f64::NEG_INFINITY;
                value
            },
            {
                let mut value = artifact.clone();
                value.contextual_effect = 0.0;
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
    fn causal_refusal_contract_fails_closed_on_provider_drift() {
        assert_eq!(
            require_causal_refusal(Err(PsychometricError::CausalUnderidentified)),
            Ok(())
        );
        assert_eq!(
            require_causal_refusal(Ok(())),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            require_causal_refusal(Err(PsychometricError::InvalidNumericInput)),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
