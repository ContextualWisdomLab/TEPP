//! Digest-bound Rubin loading uncertainty as an analysis-run profile.

use psychometric_core::{IndicatorKind, PsychometricError, combine_draw_level_ols_loadings};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, format_digest, require_receipt_identity,
    valid_identifier,
};

/// Versioned schema for a completed Rubin loading-uncertainty artifact.
pub const RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION: &str = "tepp.rubin_loading_uncertainty.v1";
/// Model contract required by the Rubin loading-uncertainty execution path.
pub const RUBIN_LOADING_MODEL_CONTRACT_VERSION: &str = "rubin_loading_uncertainty_v1";
/// Analysis-run output profile required for a Rubin loading-uncertainty artifact.
pub const RUBIN_LOADING_OUTPUT_PROFILE: &str = "rubin_loading_uncertainty_v1";
/// Maximum canonical artifact JSON size.
pub const RUBIN_LOADING_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const RUBIN_LOADING_INFERENCE_STATUS: &str = "rubin_combined_ols_loadings_not_mislevy_pv";
const RUBIN_LOADING_STATISTIC_COUNT: u64 = 5;

/// One already-mapped factor score with complete-data indicator draws.
#[derive(Clone, Debug, PartialEq)]
pub struct RubinLoadingObservation {
    factor_score: f64,
    indicator_draws: Vec<f64>,
    available_time: AvailableTime,
}

impl RubinLoadingObservation {
    /// Bind one factor score and its complete-data indicator draws to an
    /// availability clock.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the factor score is
    /// non-finite, no draws are supplied, or any draw is non-finite.
    pub fn new(
        factor_score: f64,
        indicator_draws: Vec<f64>,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        if !factor_score.is_finite()
            || indicator_draws.is_empty()
            || indicator_draws.iter().any(|value| !value.is_finite())
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            factor_score,
            indicator_draws,
            available_time,
        })
    }

    /// Return the already-mapped factor score.
    #[must_use]
    pub const fn factor_score(&self) -> f64 {
        self.factor_score
    }

    /// Return the complete-data indicator draws in source order.
    #[must_use]
    pub fn indicator_draws(&self) -> &[f64] {
        &self.indicator_draws
    }

    /// Return the availability clock used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded Rubin loading-uncertainty result.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RubinLoadingUncertaintyArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the combination.
    pub knowledge_cutoff: String,
    /// Eligible observations after cutoff.
    pub observation_count: u64,
    /// Complete-data draws combined by Rubin `T`.
    pub draw_count: u64,
    /// Observations excluded because availability was after the cutoff.
    pub excluded_after_cutoff_count: u64,
    /// Admitted indicator-kind wire name.
    pub indicator_kind: String,
    /// Arithmetic mean of per-draw OLS loadings. Not Rubin `T`.
    pub point_estimate_mean: f64,
    /// Rubin mean complete-data loading `Q̄`.
    pub mean_loading: f64,
    /// Mean complete-data sampling variance `Ū`.
    pub within_variance: f64,
    /// Between-draw variance `B`.
    pub between_variance: f64,
    /// Total variance `T = Ū + (1 + 1/m) B`.
    pub total_variance: f64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl RubinLoadingUncertaintyArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact`]
    /// when the schema, identifiers, counts, variances, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > RUBIN_LOADING_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize canonical validated artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation or serialization failure.
    #[expect(clippy::needless_return, reason = "LLVM success region")]
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        return Ok(payload);
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
        if self.schema_version != RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.observation_count < 2
            || self.draw_count < 2
            || !admitted_indicator_kind(&self.indicator_kind)
            || !self.point_estimate_mean.is_finite()
            || !self.mean_loading.is_finite()
            || !self.within_variance.is_finite()
            || self.within_variance < 0.0
            || !self.between_variance.is_finite()
            || self.between_variance < 0.0
            || !self.total_variance.is_finite()
            || self.total_variance < 0.0
            || self.inference_status != RUBIN_LOADING_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact);
        }
        Ok(())
    }
}

/// One completed Rubin loading-uncertainty artifact and terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct RubinLoadingUncertaintyExecution {
    /// Digest-bound completed combination artifact.
    pub artifact: RubinLoadingUncertaintyArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

struct EligibleRubinRows {
    factor_scores: Vec<f64>,
    indicator_draws: Vec<Vec<f64>>,
    excluded_after_cutoff_count: u64,
}

fn admitted_indicator_kind(label: &str) -> bool {
    matches!(label, "alr" | "ilr" | "logistic_normal")
}

fn admit_observations_at_cutoff(
    observations: &[RubinLoadingObservation],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<EligibleRubinRows, AnalysisEngineError> {
    if observations.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    let mut eligible = Vec::new();
    let mut excluded_after_cutoff_count = 0_u64;
    for observation in observations {
        if observation.available_time.instant() <= knowledge_cutoff.instant() {
            eligible.push(observation);
        } else {
            excluded_after_cutoff_count += 1;
        }
    }
    if eligible.is_empty() {
        return Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput,
        ));
    }
    let draw_count = eligible[0].indicator_draws.len();
    let mut factor_scores = Vec::with_capacity(eligible.len());
    let mut indicator_draws = vec![Vec::with_capacity(eligible.len()); draw_count];
    for observation in eligible {
        if observation.indicator_draws.len() != draw_count {
            return Err(AnalysisEngineError::Psychometric(
                PsychometricError::InvalidNumericInput,
            ));
        }
        factor_scores.push(observation.factor_score);
        for (draw_index, value) in observation.indicator_draws.iter().enumerate() {
            indicator_draws[draw_index].push(*value);
        }
    }
    #[rustfmt::skip]
    let rows = EligibleRubinRows { factor_scores, indicator_draws, excluded_after_cutoff_count };
    Ok(rows)
}

/// Execute cutoff-safe Rubin loading uncertainty as one analysis-run profile.
///
/// The caller supplies already-mapped factor scores and complete-data indicator
/// draws. The Rubin combination's mean loading is also the draw-mean point
/// estimate. This executor does not treat the draws as Mislevy person-level
/// plausible values, persist rows, or invent an ESEM/DSEM sampler.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, psychometric
/// recovery failure, or invalid artifact error.
#[rustfmt::skip]
#[expect(clippy::missing_panics_doc, reason = "validated local artifacts and bounded constants cannot fail")]
pub fn execute_rubin_loading_uncertainty_run(request: &AnalysisRunRequest, accepted: &AnalysisRunAccepted, snapshot_id: &str, knowledge_cutoff: KnowledgeCutoff, kind: IndicatorKind, observations: &[RubinLoadingObservation], completed_at: impl Into<String>) -> Result<RubinLoadingUncertaintyExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != RUBIN_LOADING_MODEL_CONTRACT_VERSION
        || request.output_profile != RUBIN_LOADING_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let eligible = admit_observations_at_cutoff(observations, knowledge_cutoff)?;
    let combined =
        combine_draw_level_ols_loadings(&eligible.factor_scores, &eligible.indicator_draws, kind)?;
    let point_estimate_mean = combined.mean_loading;
    let observation_count = u64::try_from(eligible.factor_scores.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let combined_draw_count = combined.draw_count;
    let draw_count = u64::try_from(combined_draw_count)
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    #[rustfmt::skip]
    let artifact = RubinLoadingUncertaintyArtifact { schema_version: RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION.into(), run_id: accepted.run_id.clone(), snapshot_id: snapshot_id.to_owned(), knowledge_cutoff: knowledge_cutoff.to_rfc3339(), observation_count, draw_count, excluded_after_cutoff_count: eligible.excluded_after_cutoff_count, indicator_kind: kind.as_str().to_owned(), point_estimate_mean, mean_loading: combined.mean_loading, within_variance: combined.within_variance, between_variance: combined.between_variance, total_variance: combined.total_variance, inference_status: RUBIN_LOADING_INFERENCE_STATUS.into() };
    let digest = artifact
        .sha256()
        .expect("constructed Rubin artifact is valid and serializable");
    let family = "rubin_loading_uncertainty";
    let statistic_count = RUBIN_LOADING_STATISTIC_COUNT;
    let status = RUBIN_LOADING_INFERENCE_STATUS;
    let summary = AnalysisResultSummary::new(family, observation_count, statistic_count, status)
        .expect("bounded Rubin summary constants are valid");
    let artifact_id = format!("rubin_loading_uncertainty_artifact_{}", &digest[..16]);
    let schema = RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION;
    let succeed = AnalysisRunTerminalResult::succeeded;
    let req = request;
    let receipt = accepted;
    let id = artifact_id;
    let hash = digest;
    let time = completed_at;
    let result_summary = summary;
    let terminal_result = succeed(req, receipt, id, hash, schema, time, result_summary)?;
    Ok(RubinLoadingUncertaintyExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        RUBIN_LOADING_ARTIFACT_BYTE_LIMIT, RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION,
        RUBIN_LOADING_INFERENCE_STATUS, RubinLoadingUncertaintyArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> RubinLoadingUncertaintyArtifact {
        RubinLoadingUncertaintyArtifact {
            schema_version: RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            observation_count: 3,
            draw_count: 2,
            excluded_after_cutoff_count: 0,
            indicator_kind: "alr".into(),
            point_estimate_mean: 0.8,
            mean_loading: 0.8,
            within_variance: 0.0,
            between_variance: 0.02,
            total_variance: 0.03,
            inference_status: RUBIN_LOADING_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &RubinLoadingUncertaintyArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            RubinLoadingUncertaintyArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            RubinLoadingUncertaintyArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)
        );
        assert_eq!(
            RubinLoadingUncertaintyArtifact::from_json(
                &"x".repeat(RUBIN_LOADING_ARTIFACT_BYTE_LIMIT + 1)
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
                value.observation_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.draw_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.indicator_kind = "raw_proportion".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.point_estimate_mean = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.mean_loading = f64::INFINITY;
                value
            },
            {
                let mut value = artifact.clone();
                value.within_variance = -0.1;
                value
            },
            {
                let mut value = artifact.clone();
                value.within_variance = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.between_variance = f64::NEG_INFINITY;
                value
            },
            {
                let mut value = artifact.clone();
                value.between_variance = -0.1;
                value
            },
            {
                let mut value = artifact.clone();
                value.total_variance = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.total_variance = -0.1;
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
}
