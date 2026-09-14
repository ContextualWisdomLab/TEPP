//! Digest-bound Rubin loading uncertainty as an analysis-run profile.

use psychometric_core::{
    IndicatorKind, PsychometricError, combine_draw_level_ols_loadings,
    recover_loading_point_estimate_mean,
};
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
const RUBIN_LOADING_MAX_DRAWS: usize = 256;
const RUBIN_LOADING_MAX_MATRIX_CELLS: usize = 1_000_000;
const RUBIN_LOADING_INFERENCE_STATUS: &str = "rubin_combined_ols_loadings_not_mislevy_pv";
const RUBIN_LOADING_PROJECTION_STATUS: &str =
    "descriptive_only_unbound_draw_generation_provenance";
const RUBIN_LOADING_STATISTIC_COUNT: u64 = 5;

/// One already-mapped factor score with complete-data indicator draws.
#[derive(Clone, Debug, PartialEq)]
pub struct RubinLoadingObservation {
    snapshot_id: String,
    factor_score: f64,
    indicator_draws: Vec<f64>,
    available_time: AvailableTime,
}

impl RubinLoadingObservation {
    /// Bind one factor score and its complete-data indicator draws to immutable
    /// snapshot and availability provenance.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the snapshot
    /// identity or numeric input is invalid, and [`AnalysisEngineError::LimitExceeded`]
    /// when one observation exceeds the application draw ceiling.
    pub fn new(
        snapshot_id: impl Into<String>,
        factor_score: f64,
        indicator_draws: Vec<f64>,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let snapshot_id = snapshot_id.into();
        if !valid_identifier(&snapshot_id)
            || !factor_score.is_finite()
            || indicator_draws.is_empty()
            || indicator_draws.iter().any(|value| !value.is_finite())
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        if indicator_draws.len() > RUBIN_LOADING_MAX_DRAWS {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        Ok(Self {
            snapshot_id,
            factor_score,
            indicator_draws,
            available_time,
        })
    }

    /// Return the immutable source snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
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
    /// Robust arithmetic mean of per-draw OLS loadings. Not Rubin `T`.
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
    /// Fail-closed projection policy for unbound draw-generation provenance.
    pub projection_status: String,
}

fn require_artifact_byte_limit(payload_len: usize) -> Result<(), AnalysisEngineError> {
    if payload_len > RUBIN_LOADING_ARTIFACT_BYTE_LIMIT {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    Ok(())
}

impl RubinLoadingUncertaintyArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact`]
    /// when the schema, identifiers, counts, variances, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        require_artifact_byte_limit(payload.len())?;
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize canonical validated artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation, size, or serialization failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        require_artifact_byte_limit(payload.len())?;
        Ok(payload)
    }

    /// Return the lowercase SHA-256 digest of canonical artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation, size, or serialization failure.
    pub fn sha256(&self) -> Result<String, AnalysisEngineError> {
        self.to_json()
            .map(|json| format_digest(Sha256::digest(json.into_bytes())))
    }

    fn validate(&self) -> Result<(), AnalysisEngineError> {
        let cutoff = KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff)
            .map_err(|_| AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)?;
        let observation_count = usize::try_from(self.observation_count)
            .map_err(|_| AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)?;
        let draw_count = usize::try_from(self.draw_count)
            .map_err(|_| AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)?;
        let excluded_after_cutoff_count = usize::try_from(self.excluded_after_cutoff_count)
            .map_err(|_| AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)?;
        let total_observation_population = observation_count
            .checked_add(excluded_after_cutoff_count)
            .ok_or(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)?;
        let matrix_cells = observation_count
            .checked_mul(draw_count)
            .ok_or(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)?;

        if self.schema_version != RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || cutoff.to_rfc3339() != self.knowledge_cutoff
            || observation_count < 2
            || total_observation_population > MAX_EVIDENCE_UNITS
            || !(2..=RUBIN_LOADING_MAX_DRAWS).contains(&draw_count)
            || matrix_cells > RUBIN_LOADING_MAX_MATRIX_CELLS
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
            || self.projection_status != RUBIN_LOADING_PROJECTION_STATUS
        {
            return Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact);
        }

        let draw_count_u32 = u32::try_from(draw_count)
            .map_err(|_| AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)?;
        let expected_total = self.within_variance
            + (1.0 + 1.0 / f64::from(draw_count_u32)) * self.between_variance;
        if !expected_total.is_finite() || expected_total != self.total_variance {
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
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<EligibleRubinRows, AnalysisEngineError> {
    if observations.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    let mut eligible = Vec::new();
    let mut excluded_after_cutoff_count = 0_u64;
    for observation in observations {
        if observation.snapshot_id != snapshot_id {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        if observation.available_time.instant() <= knowledge_cutoff.instant() {
            eligible.push(observation);
        } else {
            excluded_after_cutoff_count = excluded_after_cutoff_count
                .checked_add(1)
                .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
        }
    }
    if eligible.is_empty() {
        return Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput,
        ));
    }
    let draw_count = eligible[0].indicator_draws.len();
    if draw_count > RUBIN_LOADING_MAX_DRAWS {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    let matrix_cells = eligible
        .len()
        .checked_mul(draw_count)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if matrix_cells > RUBIN_LOADING_MAX_MATRIX_CELLS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

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
/// draws. The robust point-estimate helper and Rubin combination are invoked as
/// separate protected scientific contracts because their accumulation policies
/// are intentionally not interchangeable. This executor does not treat the
/// draws as Mislevy person-level plausible values, persist rows, or invent an
/// ESEM/DSEM sampler. Until a versioned draw-generation contract is bound to
/// claim-specific Validation Evidence, the artifact is projection-limited to
/// descriptive combination arithmetic.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, resource admission
/// refusal, psychometric recovery failure, or invalid artifact error.
#[rustfmt::skip]
pub fn execute_rubin_loading_uncertainty_run(request: &AnalysisRunRequest, accepted: &AnalysisRunAccepted, snapshot_id: &str, knowledge_cutoff: KnowledgeCutoff, kind: IndicatorKind, observations: &[RubinLoadingObservation], completed_at: impl Into<String>) -> Result<RubinLoadingUncertaintyExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if !valid_identifier(snapshot_id) {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != RUBIN_LOADING_MODEL_CONTRACT_VERSION
        || request.output_profile != RUBIN_LOADING_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let eligible = admit_observations_at_cutoff(observations, snapshot_id, knowledge_cutoff)?;
    let point_estimate_mean = recover_loading_point_estimate_mean(
        &eligible.factor_scores,
        &eligible.indicator_draws,
        kind,
    )?;
    let combined =
        combine_draw_level_ols_loadings(&eligible.factor_scores, &eligible.indicator_draws, kind)?;
    let observation_count = u64::try_from(eligible.factor_scores.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let draw_count = u64::try_from(combined.draw_count)
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    #[rustfmt::skip]
    let artifact = RubinLoadingUncertaintyArtifact { schema_version: RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION.into(), run_id: accepted.run_id.clone(), snapshot_id: snapshot_id.to_owned(), knowledge_cutoff: knowledge_cutoff.to_rfc3339(), observation_count, draw_count, excluded_after_cutoff_count: eligible.excluded_after_cutoff_count, indicator_kind: kind.as_str().to_owned(), point_estimate_mean, mean_loading: combined.mean_loading, within_variance: combined.within_variance, between_variance: combined.between_variance, total_variance: combined.total_variance, inference_status: RUBIN_LOADING_INFERENCE_STATUS.into(), projection_status: RUBIN_LOADING_PROJECTION_STATUS.into() };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "rubin_loading_uncertainty",
        observation_count,
        RUBIN_LOADING_STATISTIC_COUNT,
        "validated",
    )?;
    let artifact_id = format!("rubin_loading_uncertainty_artifact_{}", &digest[..16]);
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        artifact_id,
        digest,
        RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(RubinLoadingUncertaintyExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        RUBIN_LOADING_ARTIFACT_BYTE_LIMIT, RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION,
        RUBIN_LOADING_INFERENCE_STATUS, RUBIN_LOADING_MAX_DRAWS, RUBIN_LOADING_MAX_MATRIX_CELLS,
        RUBIN_LOADING_PROJECTION_STATUS, RubinLoadingUncertaintyArtifact,
        require_artifact_byte_limit,
    };
    use crate::{AnalysisEngineError, MAX_EVIDENCE_UNITS};

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
            projection_status: RUBIN_LOADING_PROJECTION_STATUS.into(),
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
        assert_eq!(
            require_artifact_byte_limit(RUBIN_LOADING_ARTIFACT_BYTE_LIMIT),
            Ok(())
        );
        assert_eq!(
            require_artifact_byte_limit(RUBIN_LOADING_ARTIFACT_BYTE_LIMIT + 1),
            Err(AnalysisEngineError::LimitExceeded)
        );
    }

    #[test]
    fn maximal_valid_artifact_stays_inside_the_wire_envelope() {
        let mut artifact = artifact();
        artifact.run_id = "\\".repeat(256);
        artifact.snapshot_id = "\\".repeat(256);
        artifact.draw_count = u64::try_from(RUBIN_LOADING_MAX_DRAWS).expect("draw bound");
        artifact.observation_count =
            u64::try_from(RUBIN_LOADING_MAX_MATRIX_CELLS / RUBIN_LOADING_MAX_DRAWS)
                .expect("observation bound");
        artifact.excluded_after_cutoff_count = u64::try_from(MAX_EVIDENCE_UNITS)
            .expect("population bound")
            - artifact.observation_count;
        artifact.point_estimate_mean = f64::MAX;
        artifact.mean_loading = -f64::MAX;
        artifact.within_variance = 0.0;
        artifact.between_variance = 0.0;
        artifact.total_variance = 0.0;
        let payload = artifact.to_json().expect("maximal valid json");
        assert!(payload.len() < RUBIN_LOADING_ARTIFACT_BYTE_LIMIT);
        assert_eq!(
            RubinLoadingUncertaintyArtifact::from_json(&payload),
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
                value.knowledge_cutoff = "2026-08-01T01:00:00+01:00".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.observation_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.observation_count = u64::try_from(MAX_EVIDENCE_UNITS).expect("bound") + 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.excluded_after_cutoff_count = u64::MAX;
                value
            },
            {
                let mut value = artifact.clone();
                value.draw_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.draw_count = u64::try_from(RUBIN_LOADING_MAX_DRAWS).expect("bound") + 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.observation_count =
                    u64::try_from(RUBIN_LOADING_MAX_MATRIX_CELLS / RUBIN_LOADING_MAX_DRAWS + 1)
                        .expect("bound");
                value.draw_count = u64::try_from(RUBIN_LOADING_MAX_DRAWS).expect("bound");
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
                value.total_variance = 0.04;
                value
            },
            {
                let mut value = artifact.clone();
                value.inference_status.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.projection_status.clear();
                value
            },
        ];
        for invalid in invalid_artifacts {
            assert_invalid(&invalid);
        }
    }

    #[test]
    fn from_json_rejects_finite_but_inconsistent_rubin_components() {
        let mut artifact = artifact();
        artifact.total_variance = 0.04;
        let payload = serde_json::to_string(&artifact).expect("unchecked json");
        assert_eq!(
            RubinLoadingUncertaintyArtifact::from_json(&payload),
            Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)
        );
    }
}
