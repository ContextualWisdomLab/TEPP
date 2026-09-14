//! Digest-bound two-group OLS invariance as an analysis-run profile.

use psychometric_core::{
    GroupIndicatorSeries, IndicatorKind, PsychometricError, classify_two_group_ols_invariance,
    recover_strong_gated_latent_mean_difference,
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

/// Versioned schema for a completed two-group OLS invariance artifact.
pub const TWO_GROUP_OLS_INVARIANCE_ARTIFACT_SCHEMA_VERSION: &str =
    "tepp.two_group_ols_invariance.v1";
/// Model contract required by the two-group OLS invariance execution path.
pub const TWO_GROUP_OLS_INVARIANCE_MODEL_CONTRACT_VERSION: &str = "two_group_ols_invariance_v1";
/// Analysis-run output profile required for a two-group OLS invariance artifact.
pub const TWO_GROUP_OLS_INVARIANCE_OUTPUT_PROFILE: &str = "two_group_ols_invariance_v1";
/// Maximum canonical artifact JSON size.
pub const TWO_GROUP_OLS_INVARIANCE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const TWO_GROUP_OLS_INVARIANCE_INFERENCE_STATUS: &str = "two_group_ols_invariance_not_mgcfa";
const TWO_GROUP_OLS_INVARIANCE_STATISTIC_COUNT: u64 = 7;
const TWO_GROUP_OLS_INVARIANCE_TOLERANCE: f64 = 1e-9;

/// One already-mapped factor score and indicator bound to availability.
#[derive(Clone, Debug, PartialEq)]
pub struct InvarianceObservation {
    factor_score: f64,
    indicator: f64,
    available_time: AvailableTime,
}

impl InvarianceObservation {
    /// Bind one factor score and indicator to an availability clock.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when either coordinate
    /// is non-finite.
    pub fn new(
        factor_score: f64,
        indicator: f64,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        if !factor_score.is_finite() || !indicator.is_finite() {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            factor_score,
            indicator,
            available_time,
        })
    }

    /// Return the already-mapped factor score.
    #[must_use]
    pub const fn factor_score(&self) -> f64 {
        self.factor_score
    }

    /// Return the already-mapped indicator coordinate.
    #[must_use]
    pub const fn indicator(&self) -> f64 {
        self.indicator
    }

    /// Return the availability clock used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded two-group OLS invariance result.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TwoGroupOlsInvarianceArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the classification.
    pub knowledge_cutoff: String,
    /// Eligible reference-group observations after cutoff.
    pub reference_observation_count: u64,
    /// Eligible comparison-group observations after cutoff.
    pub comparison_observation_count: u64,
    /// Reference observations excluded because availability was after the cutoff.
    pub excluded_after_cutoff_reference_count: u64,
    /// Comparison observations excluded because availability was after the cutoff.
    pub excluded_after_cutoff_comparison_count: u64,
    /// Admitted indicator-kind wire name.
    pub indicator_kind: String,
    /// Local Meredith-style status (`strong` / `strict`).
    pub invariance_status: String,
    /// `#84` wire name (`scalar`) or `null` when local strict has no `#84` name.
    pub measurement_invariance_wire_name: Option<String>,
    /// Whether the classified status licenses latent-mean comparison.
    pub licenses_latent_mean_comparison: bool,
    /// Strong/strict-gated `(ȳ_c − ȳ_r) / λ`.
    pub latent_mean_difference: f64,
    /// Reference-group OLS intercept.
    pub reference_intercept: f64,
    /// Reference-group OLS loading.
    pub reference_loading: f64,
    /// Comparison-group OLS intercept.
    pub comparison_intercept: f64,
    /// Comparison-group OLS loading.
    pub comparison_loading: f64,
    /// Reference residual variance.
    pub reference_residual_variance: f64,
    /// Comparison residual variance.
    pub comparison_residual_variance: f64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl TwoGroupOlsInvarianceArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidTwoGroupOlsInvarianceArtifact`]
    /// when the schema, identifiers, counts, OLS values, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > TWO_GROUP_OLS_INVARIANCE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidTwoGroupOlsInvarianceArtifact)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize canonical validated artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation, serialization, or size failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        if payload.len() > TWO_GROUP_OLS_INVARIANCE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
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
        if self.schema_version != TWO_GROUP_OLS_INVARIANCE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.reference_observation_count < 2
            || self.comparison_observation_count < 2
            || !admitted_indicator_kind(&self.indicator_kind)
            || !valid_invariance_status_and_wire(
                &self.invariance_status,
                self.measurement_invariance_wire_name.as_deref(),
            )
            || !self.licenses_latent_mean_comparison
            || !self.latent_mean_difference.is_finite()
            || !self.reference_intercept.is_finite()
            || !self.reference_loading.is_finite()
            || !self.comparison_intercept.is_finite()
            || !self.comparison_loading.is_finite()
            || !self.reference_residual_variance.is_finite()
            || self.reference_residual_variance < 0.0
            || !self.comparison_residual_variance.is_finite()
            || self.comparison_residual_variance < 0.0
            || self.inference_status != TWO_GROUP_OLS_INVARIANCE_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidTwoGroupOlsInvarianceArtifact);
        }
        Ok(())
    }
}

/// One completed two-group OLS invariance artifact and terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct TwoGroupOlsInvarianceExecution {
    /// Digest-bound completed invariance artifact.
    pub artifact: TwoGroupOlsInvarianceArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

struct EligibleGroup {
    series: GroupIndicatorSeries,
    excluded_after_cutoff_count: u64,
}

fn admitted_indicator_kind(label: &str) -> bool {
    matches!(label, "alr" | "ilr" | "logistic_normal")
}

fn valid_invariance_status_and_wire(status: &str, wire: Option<&str>) -> bool {
    matches!(
        (status, wire),
        ("strong", Some("scalar")) | ("strict", None)
    )
}

fn admit_group_at_cutoff(
    observations: &[InvarianceObservation],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<EligibleGroup, AnalysisEngineError> {
    let mut factor_scores = Vec::new();
    let mut indicators = Vec::new();
    let mut excluded_after_cutoff_count = 0_u64;
    for observation in observations {
        if observation.available_time.instant() <= knowledge_cutoff.instant() {
            factor_scores.push(observation.factor_score);
            indicators.push(observation.indicator);
        } else {
            excluded_after_cutoff_count += 1;
        }
    }
    if factor_scores.is_empty() {
        return Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput,
        ));
    }
    Ok(EligibleGroup {
        series: GroupIndicatorSeries {
            factor_scores,
            indicators,
        },
        excluded_after_cutoff_count,
    })
}

/// Execute cutoff-safe two-group OLS invariance as one analysis-run profile.
///
/// The caller supplies already-mapped factor scores and indicators for a
/// reference group and a comparison group. This executor jointly invokes
/// [`classify_two_group_ols_invariance`] and
/// [`recover_strong_gated_latent_mean_difference`]. Metric/configural status
/// fails closed: metric does not license latent-mean comparison. It does not
/// invent an MGCFA sampler, persist rows, or restore a Driver p.16 `std` map.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, psychometric
/// recovery failure, or invalid artifact error.
#[allow(clippy::too_many_arguments)]
pub fn execute_two_group_ols_invariance_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    kind: IndicatorKind,
    reference: &[InvarianceObservation],
    comparison: &[InvarianceObservation],
    completed_at: impl Into<String>,
) -> Result<TwoGroupOlsInvarianceExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != TWO_GROUP_OLS_INVARIANCE_MODEL_CONTRACT_VERSION
        || request.output_profile != TWO_GROUP_OLS_INVARIANCE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if reference.len().saturating_add(comparison.len()) > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let eligible_reference = admit_group_at_cutoff(reference, knowledge_cutoff)?;
    let eligible_comparison = admit_group_at_cutoff(comparison, knowledge_cutoff)?;
    let measurement = classify_two_group_ols_invariance(
        &eligible_reference.series,
        &eligible_comparison.series,
        kind,
        TWO_GROUP_OLS_INVARIANCE_TOLERANCE,
        TWO_GROUP_OLS_INVARIANCE_TOLERANCE,
        TWO_GROUP_OLS_INVARIANCE_TOLERANCE,
    )?;
    let latent_mean_difference = recover_strong_gated_latent_mean_difference(
        &eligible_reference.series,
        &eligible_comparison.series,
        kind,
        TWO_GROUP_OLS_INVARIANCE_TOLERANCE,
        TWO_GROUP_OLS_INVARIANCE_TOLERANCE,
        TWO_GROUP_OLS_INVARIANCE_TOLERANCE,
    )?;
    let reference_observation_count = u64::try_from(eligible_reference.series.factor_scores.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let comparison_observation_count =
        u64::try_from(eligible_comparison.series.factor_scores.len())
            .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let evidence_count = reference_observation_count
        .checked_add(comparison_observation_count)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    let artifact = TwoGroupOlsInvarianceArtifact {
        schema_version: TWO_GROUP_OLS_INVARIANCE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        reference_observation_count,
        comparison_observation_count,
        excluded_after_cutoff_reference_count: eligible_reference.excluded_after_cutoff_count,
        excluded_after_cutoff_comparison_count: eligible_comparison.excluded_after_cutoff_count,
        indicator_kind: kind.as_str().to_owned(),
        invariance_status: measurement.status.as_str().to_owned(),
        measurement_invariance_wire_name: measurement
            .status
            .as_measurement_invariance_wire_name()
            .map(str::to_owned),
        licenses_latent_mean_comparison: measurement.status.licenses_latent_mean_comparison(),
        latent_mean_difference,
        reference_intercept: measurement.reference_intercept,
        reference_loading: measurement.reference_loading,
        comparison_intercept: measurement.comparison_intercept,
        comparison_loading: measurement.comparison_loading,
        reference_residual_variance: measurement.reference_residual_variance,
        comparison_residual_variance: measurement.comparison_residual_variance,
        inference_status: TWO_GROUP_OLS_INVARIANCE_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "two_group_ols_invariance",
        evidence_count,
        TWO_GROUP_OLS_INVARIANCE_STATISTIC_COUNT,
        TWO_GROUP_OLS_INVARIANCE_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("two_group_ols_invariance_artifact_{}", &digest[..16]),
        digest,
        TWO_GROUP_OLS_INVARIANCE_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(TwoGroupOlsInvarianceExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        TWO_GROUP_OLS_INVARIANCE_ARTIFACT_BYTE_LIMIT,
        TWO_GROUP_OLS_INVARIANCE_ARTIFACT_SCHEMA_VERSION,
        TWO_GROUP_OLS_INVARIANCE_INFERENCE_STATUS, TwoGroupOlsInvarianceArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> TwoGroupOlsInvarianceArtifact {
        TwoGroupOlsInvarianceArtifact {
            schema_version: TWO_GROUP_OLS_INVARIANCE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            reference_observation_count: 3,
            comparison_observation_count: 3,
            excluded_after_cutoff_reference_count: 0,
            excluded_after_cutoff_comparison_count: 0,
            indicator_kind: "alr".into(),
            invariance_status: "strict".into(),
            measurement_invariance_wire_name: None,
            licenses_latent_mean_comparison: true,
            latent_mean_difference: 2.0,
            reference_intercept: 0.5,
            reference_loading: 1.2,
            comparison_intercept: 0.5,
            comparison_loading: 1.2,
            reference_residual_variance: 0.0,
            comparison_residual_variance: 0.0,
            inference_status: TWO_GROUP_OLS_INVARIANCE_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &TwoGroupOlsInvarianceArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidTwoGroupOlsInvarianceArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            TwoGroupOlsInvarianceArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            TwoGroupOlsInvarianceArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidTwoGroupOlsInvarianceArtifact)
        );
        assert_eq!(
            TwoGroupOlsInvarianceArtifact::from_json(
                &"x".repeat(TWO_GROUP_OLS_INVARIANCE_ARTIFACT_BYTE_LIMIT + 1)
            ),
            Err(AnalysisEngineError::LimitExceeded)
        );
    }

    #[test]
    fn artifact_identity_tampering_fails_closed() {
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
                value.reference_observation_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.comparison_observation_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.indicator_kind = "raw_proportion".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.invariance_status = "metric".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.invariance_status = "strict".into();
                value.measurement_invariance_wire_name = Some("scalar".into());
                value
            },
            {
                let mut value = artifact.clone();
                value.invariance_status = "strong".into();
                value.measurement_invariance_wire_name = None;
                value
            },
        ];
        for invalid in invalid_artifacts {
            assert_invalid(&invalid);
        }
    }

    #[test]
    fn artifact_numeric_tampering_fails_closed() {
        let artifact = artifact();
        let invalid_artifacts = [
            {
                let mut value = artifact.clone();
                value.licenses_latent_mean_comparison = false;
                value
            },
            {
                let mut value = artifact.clone();
                value.latent_mean_difference = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.reference_intercept = f64::INFINITY;
                value
            },
            {
                let mut value = artifact.clone();
                value.reference_loading = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.comparison_intercept = f64::NEG_INFINITY;
                value
            },
            {
                let mut value = artifact.clone();
                value.comparison_loading = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.reference_residual_variance = -0.1;
                value
            },
            {
                let mut value = artifact.clone();
                value.comparison_residual_variance = f64::NAN;
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
    fn strong_scalar_wire_name_round_trips() {
        let mut artifact = artifact();
        artifact.invariance_status = "strong".into();
        artifact.measurement_invariance_wire_name = Some("scalar".into());
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            TwoGroupOlsInvarianceArtifact::from_json(&payload),
            Ok(artifact)
        );
    }
}
