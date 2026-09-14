//! Digest-bound independent TDT link-criterion fitting as an analysis-run profile.

use corpus_split::cutoff_eligible;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, LineageCriterionFitError, LineageCriterionObservation, MAX_EVIDENCE_UNITS,
    fit_lineage_criterion_posteriors, format_digest, require_receipt_identity, valid_identifier,
};

/// Versioned schema for a completed lineage-criterion artifact.
pub const LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION: &str = "tepp.lineage_criterion.v1";
/// Model contract required by the lineage-criterion execution path.
pub const LINEAGE_CRITERION_MODEL_CONTRACT_VERSION: &str = "lineage_criterion_v1";
/// Analysis-run output profile required for a lineage-criterion artifact.
pub const LINEAGE_CRITERION_OUTPUT_PROFILE: &str = "lineage_criterion_v1";
/// Maximum accepted lineage-criterion artifact JSON size.
pub const LINEAGE_CRITERION_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const MAX_LINEAGE_CRITERION_DRAW_VALUES: usize = 1_000_000;
const LINEAGE_CRITERION_INFERENCE_STATUS: &str =
    "independent_tdt_criterion_not_date_from_record_order";

/// Cutoff-safe independent TDT link-criterion payload bound to the existing fitter.
///
/// Provenance is carried in parallel slices because the scientific
/// [`LineageCriterionObservation`] remains the estimator-owned counts/draws
/// contract. Alignment is checked before any observation is admitted.
#[derive(Clone, Debug)]
pub struct LineageCriterionInput<'a> {
    observations: &'a [LineageCriterionObservation],
    snapshot_ids: &'a [String],
    available_times: &'a [AvailableTime],
    draw_count: usize,
}

impl<'a> LineageCriterionInput<'a> {
    /// Construct a lineage-criterion payload with explicit per-pair provenance.
    #[must_use]
    pub const fn new(
        observations: &'a [LineageCriterionObservation],
        snapshot_ids: &'a [String],
        available_times: &'a [AvailableTime],
        draw_count: usize,
    ) -> Self {
        Self {
            observations,
            snapshot_ids,
            available_times,
            draw_count,
        }
    }

    /// Borrow the independently observed exact pairs.
    #[must_use]
    pub const fn observations(&self) -> &'a [LineageCriterionObservation] {
        self.observations
    }

    /// Borrow immutable source snapshot identities aligned to observations.
    #[must_use]
    pub const fn snapshot_ids(&self) -> &'a [String] {
        self.snapshot_ids
    }

    /// Borrow pair-observation availability times aligned to observations.
    #[must_use]
    pub const fn available_times(&self) -> &'a [AvailableTime] {
        self.available_times
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
    /// Number of independently observed exact pairs admitted at the cutoff.
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
    /// the schema, identifiers, counts, resource budget, or claim boundary fail.
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
    /// Valid identifiers, strict timestamp syntax, and bounded pair/draw counts
    /// make canonical output smaller than [`LINEAGE_CRITERION_ARTIFACT_BYTE_LIMIT`].
    /// Untrusted input remains capped by [`Self::from_json`].
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
        let pair_count = usize::try_from(self.pair_count).ok();
        let draw_count = usize::try_from(self.draw_count).ok();
        if self.schema_version != LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || pair_count.is_none()
            || draw_count.is_none()
            || validate_resource_budget(
                pair_count.unwrap_or_default(),
                draw_count.unwrap_or_default(),
            )
            .is_err()
            || pair_count == Some(0)
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
    /// Digest-bound completed lineage-criterion census.
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
/// draw counts. Event-time draws remain producer evidence and are distinct
/// from the evidence-availability clock used for historical admission. This is
/// not a Bayesian sampler, GPU execution, or topic birth/split/merge.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, malformed or
/// unavailable provenance, resource limit, fitter refusal, or invalid artifact
/// error.
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
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != LINEAGE_CRITERION_MODEL_CONTRACT_VERSION
        || request.output_profile != LINEAGE_CRITERION_OUTPUT_PROFILE
        || input.observations().len() != input.snapshot_ids().len()
        || input.observations().len() != input.available_times().len()
        || input.draw_count() < 2
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if input.observations().len() > MAX_EVIDENCE_UNITS
        || input.draw_count() > MAX_LINEAGE_CRITERION_DRAW_VALUES
    {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let mut admitted_observations = Vec::with_capacity(input.observations().len());
    for ((observation, observation_snapshot_id), available_time) in input
        .observations()
        .iter()
        .zip(input.snapshot_ids())
        .zip(input.available_times())
    {
        if observation_snapshot_id != snapshot_id {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        if !cutoff_eligible(available_time, &knowledge_cutoff) {
            continue;
        }
        let next_pair_count = admitted_observations
            .len()
            .checked_add(1)
            .ok_or(AnalysisEngineError::LimitExceeded)?;
        validate_resource_budget(next_pair_count, input.draw_count())?;
        validate_event_time_draws(observation, input.draw_count())?;
        admitted_observations.push(observation.clone());
    }

    let fits = fit_lineage_criterion_posteriors(&admitted_observations, input.draw_count()).map_err(
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
    let summary = AnalysisResultSummary::new("lineage_criterion", pair_count, 2, "validated")?;
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

fn validate_event_time_draws(
    observation: &LineageCriterionObservation,
    draw_count: usize,
) -> Result<(), AnalysisEngineError> {
    if observation.predecessor_event_time_draws.len() != draw_count
        || observation.successor_event_time_draws.len() != draw_count
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    for (predecessor, successor) in observation
        .predecessor_event_time_draws
        .iter()
        .zip(&observation.successor_event_time_draws)
    {
        let predecessor = EventTime::parse_rfc3339(predecessor)
            .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
        let successor = EventTime::parse_rfc3339(successor)
            .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
        if predecessor > successor {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
    }
    Ok(())
}

fn validate_resource_budget(
    pair_count: usize,
    draw_count: usize,
) -> Result<(), AnalysisEngineError> {
    if pair_count > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    if draw_count < 2 {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    let draw_values = pair_count
        .checked_mul(draw_count)
        .ok_or(AnalysisEngineError::LimitExceeded)?;
    if draw_values > MAX_LINEAGE_CRITERION_DRAW_VALUES {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        LINEAGE_CRITERION_ARTIFACT_BYTE_LIMIT, LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION,
        LINEAGE_CRITERION_INFERENCE_STATUS, LineageCriterionArtifact, LineageCriterionInput,
        MAX_LINEAGE_CRITERION_DRAW_VALUES, validate_event_time_draws, validate_resource_budget,
    };
    use crate::{AnalysisEngineError, LineageCriterionObservation, MAX_EVIDENCE_UNITS};
    use temporal_core::AvailableTime;

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

    fn observation() -> LineageCriterionObservation {
        LineageCriterionObservation {
            pair_id: "pair-a".into(),
            successes: 1,
            trials: 2,
            predecessor_event_time_draws: vec!["2026-01-01T00:00:00Z".into(); 2],
            successor_event_time_draws: vec!["2026-01-02T00:00:00Z".into(); 2],
        }
    }

    fn assert_invalid(artifact: &LineageCriterionArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidLineageCriterionArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_input_size_bounds_fail_closed() {
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
    fn maximal_valid_artifact_stays_below_the_input_wire_cap() {
        let artifact = LineageCriterionArtifact {
            schema_version: LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "\\".repeat(256),
            snapshot_id: "\\".repeat(256),
            knowledge_cutoff: "2026-08-01T23:59:59.999999999+23:59".into(),
            pair_count: u64::try_from(MAX_EVIDENCE_UNITS).expect("pair bound"),
            draw_count: 10,
            inference_status: LINEAGE_CRITERION_INFERENCE_STATUS.into(),
        };
        let payload = artifact.to_json().expect("bounded output");
        assert!(payload.len() < LINEAGE_CRITERION_ARTIFACT_BYTE_LIMIT);
        assert_eq!(LineageCriterionArtifact::from_json(&payload), Ok(artifact));
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
    fn event_time_draw_shape_rejects_mismatch_malformed_and_reverse_order() {
        let valid = observation();
        assert_eq!(validate_event_time_draws(&valid, 2), Ok(()));

        let mut mismatched = valid.clone();
        mismatched.successor_event_time_draws.pop();
        assert_eq!(
            validate_event_time_draws(&mismatched, 2),
            Err(AnalysisEngineError::InvalidEvidence)
        );

        let mut malformed = valid.clone();
        malformed.predecessor_event_time_draws[0] = "not-a-time".into();
        assert_eq!(
            validate_event_time_draws(&malformed, 2),
            Err(AnalysisEngineError::InvalidEvidence)
        );

        let mut reversed = valid;
        reversed.predecessor_event_time_draws[0] = "2026-01-03T00:00:00Z".into();
        assert_eq!(
            validate_event_time_draws(&reversed, 2),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }

    #[test]
    fn resource_budget_bounds_pairs_and_materialized_draw_values() {
        assert_eq!(
            validate_resource_budget(MAX_EVIDENCE_UNITS, 10),
            Ok(())
        );
        assert_eq!(
            validate_resource_budget(MAX_EVIDENCE_UNITS + 1, 2),
            Err(AnalysisEngineError::LimitExceeded)
        );
        assert_eq!(
            validate_resource_budget(1, MAX_LINEAGE_CRITERION_DRAW_VALUES + 1),
            Err(AnalysisEngineError::LimitExceeded)
        );
        assert_eq!(
            validate_resource_budget(1, 1),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }

    #[test]
    fn input_accessors_expose_observations_provenance_and_draw_count() {
        let observations = [LineageCriterionObservation {
            pair_id: "pair-a".into(),
            successes: 1,
            trials: 2,
            predecessor_event_time_draws: vec!["2026-01-01T00:00:00Z".into(); 32],
            successor_event_time_draws: vec!["2026-01-02T00:00:00Z".into(); 32],
        }];
        let snapshot_ids = ["snapshot-1".to_owned()];
        let available_times = [
            AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available time"),
        ];
        let input = LineageCriterionInput::new(&observations, &snapshot_ids, &available_times, 32);
        assert_eq!(input.observations(), &observations);
        assert_eq!(input.snapshot_ids(), &snapshot_ids);
        assert_eq!(input.available_times(), &available_times);
        assert_eq!(input.draw_count(), 32);
    }
}
