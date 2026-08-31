//! Digest-bound independent TDT link-criterion fitting as an analysis-run profile.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, LineageCriterionFitError, LineageCriterionObservation,
    fit_lineage_criterion_posteriors, format_digest, require_receipt_identity, valid_identifier,
};

/// Versioned schema for a completed lineage-criterion artifact.
pub const LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION: &str = "tepp.lineage_criterion.v1";
/// Model contract required by the lineage-criterion execution path.
pub const LINEAGE_CRITERION_MODEL_CONTRACT_VERSION: &str = "lineage_criterion_v1";
/// Analysis-run output profile required for a lineage-criterion artifact.
pub const LINEAGE_CRITERION_OUTPUT_PROFILE: &str = "lineage_criterion_v1";
/// Maximum canonical artifact JSON size.
pub const LINEAGE_CRITERION_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const LINEAGE_CRITERION_INFERENCE_STATUS: &str =
    "independent_tdt_criterion_not_date_from_record_order";

/// Cutoff-safe independent TDT link-criterion payload bound to the existing fitter.
#[derive(Clone, Debug)]
pub struct LineageCriterionInput<'a> {
    observations: &'a [LineageCriterionObservation],
    draw_count: usize,
}

impl<'a> LineageCriterionInput<'a> {
    /// Construct a lineage-criterion payload from existing fitter inputs.
    #[must_use]
    pub const fn new(observations: &'a [LineageCriterionObservation], draw_count: usize) -> Self {
        Self {
            observations,
            draw_count,
        }
    }

    /// Borrow the independently observed exact pairs.
    #[must_use]
    pub const fn observations(&self) -> &'a [LineageCriterionObservation] {
        self.observations
    }

    /// Return the common temporal-draw count carried without alteration.
    #[must_use]
    pub const fn draw_count(&self) -> usize {
        self.draw_count
    }
}

/// Completed, bounded independent TDT link-criterion counts for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineageCriterionArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the run.
    pub knowledge_cutoff: String,
    /// Number of independently observed exact pairs.
    pub pair_count: u64,
    /// Common temporal-draw count carried without alteration.
    pub draw_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl LineageCriterionArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidLineageCriterionArtifact`] when
    /// the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > LINEAGE_CRITERION_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidLineageCriterionArtifact)?;
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
        if payload.len() > LINEAGE_CRITERION_ARTIFACT_BYTE_LIMIT {
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
        if self.schema_version != LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.pair_count == 0
            || self.draw_count == 0
            || self.inference_status != LINEAGE_CRITERION_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidLineageCriterionArtifact);
        }
        Ok(())
    }
}

/// One completed lineage-criterion artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct LineageCriterionExecution {
    /// Digest-bound completed lineage-criterion artifact.
    pub artifact: LineageCriterionArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute independent TDT link-criterion fitting as one analysis-run profile.
///
/// The executor invokes [`fit_lineage_criterion_posteriors`] and does not
/// reimplement Jeffreys fitting, infer a date from record order, or promote
/// CHRONOS predictions to observed facts. Raw posteriors stay with the
/// scientific fitter; the operator artifact carries only bounded pair and
/// draw counts. Event-time draws remain producer evidence. This is not a
/// Bayesian sampler, not GPU execution, and not topic birth/split/merge.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, invalid
/// observations, fitter refusal, or invalid artifact error.
pub fn execute_lineage_criterion_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &LineageCriterionInput<'_>,
    completed_at: impl Into<String>,
) -> Result<LineageCriterionExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != LINEAGE_CRITERION_MODEL_CONTRACT_VERSION
        || request.output_profile != LINEAGE_CRITERION_OUTPUT_PROFILE
        || input.draw_count() == 0
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let fits = fit_lineage_criterion_posteriors(input.observations(), input.draw_count()).map_err(
        |error| match error {
            LineageCriterionFitError::EmptyInput
            | LineageCriterionFitError::InvalidPairIdentity
            | LineageCriterionFitError::TemporalDrawMismatch => {
                AnalysisEngineError::InvalidEvidence
            }
            LineageCriterionFitError::Criterion(_) => {
                AnalysisEngineError::LineageCriterionFitFailure
            }
        },
    )?;
    let pair_count =
        u64::try_from(fits.len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let draw_count =
        u64::try_from(input.draw_count()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let artifact = LineageCriterionArtifact {
        schema_version: LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        pair_count,
        draw_count,
        inference_status: LINEAGE_CRITERION_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "lineage_criterion",
        pair_count,
        2,
        LINEAGE_CRITERION_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("lineage_criterion_artifact_{}", &digest[..16]),
        digest,
        LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(LineageCriterionExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        LINEAGE_CRITERION_ARTIFACT_BYTE_LIMIT, LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION,
        LINEAGE_CRITERION_INFERENCE_STATUS, LineageCriterionArtifact, LineageCriterionInput,
    };
    use crate::{AnalysisEngineError, LineageCriterionObservation};

    fn artifact() -> LineageCriterionArtifact {
        LineageCriterionArtifact {
            schema_version: LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            pair_count: 2,
            draw_count: 32,
            inference_status: LINEAGE_CRITERION_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &LineageCriterionArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidLineageCriterionArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            LineageCriterionArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            LineageCriterionArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidLineageCriterionArtifact)
        );
        assert_eq!(
            LineageCriterionArtifact::from_json(
                &"x".repeat(LINEAGE_CRITERION_ARTIFACT_BYTE_LIMIT + 1)
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
                value.pair_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.draw_count = 0;
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
    fn input_accessors_expose_observations_and_draw_count() {
        let observations = [LineageCriterionObservation {
            pair_id: "pair-a".into(),
            successes: 1,
            trials: 2,
            predecessor_event_time_draws: vec!["2026-01-01T00:00:00Z".into(); 32],
            successor_event_time_draws: vec!["2026-01-02T00:00:00Z".into(); 32],
        }];
        let input = LineageCriterionInput::new(&observations, 32);
        assert_eq!(input.observations(), &observations);
        assert_eq!(input.draw_count(), 32);
    }
}
