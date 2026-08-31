//! Digest-bound irregular event-time log-rate as an analysis-run profile.

use psychometric_core::{
    CausalHeuristic, EventOccasion, LagClock, PsychometricError, claim_causal_effect,
    map_discrete_lag_across_event_intervals, recover_discrete_lag_one,
    recover_event_series_mean_log_rate, refuse_pooled_discrete_lag_across_unequal_intervals,
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

/// Versioned schema for a completed irregular event-time artifact.
pub const IRREGULAR_EVENT_TIME_ARTIFACT_SCHEMA_VERSION: &str = "tepp.irregular_event_time.v1";
/// Model contract required by the irregular event-time execution path.
pub const IRREGULAR_EVENT_TIME_MODEL_CONTRACT_VERSION: &str = "irregular_event_time_v1";
/// Analysis-run output profile required for an irregular event-time artifact.
pub const IRREGULAR_EVENT_TIME_OUTPUT_PROFILE: &str = "irregular_event_time_v1";
/// Maximum canonical artifact JSON size.
pub const IRREGULAR_EVENT_TIME_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const IRREGULAR_EVENT_TIME_INFERENCE_STATUS: &str = "composed_interval_mapped_lags_not_dsem";

/// One already-mapped event-time score offered to a cutoff-safe run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IrregularEventScore {
    event_time: f64,
    score: f64,
    available_time: AvailableTime,
}

impl IrregularEventScore {
    /// Bind one event-time score to an availability clock.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the event time or
    /// score is non-finite.
    pub fn new(
        event_time: f64,
        score: f64,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        if !event_time.is_finite() || !score.is_finite() {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            event_time,
            score,
            available_time,
        })
    }

    /// Return the event / valid time of the observation.
    #[must_use]
    pub const fn event_time(self) -> f64 {
        self.event_time
    }

    /// Return the already-mapped score.
    #[must_use]
    pub const fn score(self) -> f64 {
        self.score
    }

    /// Return the availability clock used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded irregular event-time composition for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IrregularEventTimeArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the composition.
    pub knowledge_cutoff: String,
    /// Eligible occasions after cutoff.
    pub occasion_count: u64,
    /// Consecutive event intervals among eligible occasions.
    pub interval_count: u64,
    /// Occasions excluded because availability was after the cutoff.
    pub excluded_after_cutoff_count: u64,
    /// Mean local log-rate `a` across eligible irregular intervals.
    pub mean_log_rate: f64,
    /// Discrete lag mapped onto the requested reference interval through `a`.
    pub mapped_reference_lag: f64,
    /// Positive reference event interval used for the mapped lag.
    pub reference_delta: f64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl IrregularEventTimeArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidIrregularEventTimeArtifact`] when
    /// the schema, identifiers, counts, rates, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > IRREGULAR_EVENT_TIME_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidIrregularEventTimeArtifact)?;
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
        if payload.len() > IRREGULAR_EVENT_TIME_ARTIFACT_BYTE_LIMIT {
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
        if self.schema_version != IRREGULAR_EVENT_TIME_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.occasion_count < 2
            || self.interval_count == 0
            || self.interval_count != self.occasion_count.saturating_sub(1)
            || !self.mean_log_rate.is_finite()
            || !self.mapped_reference_lag.is_finite()
            || self.mapped_reference_lag <= 0.0
            || !self.reference_delta.is_finite()
            || self.reference_delta <= 0.0
            || self.inference_status != IRREGULAR_EVENT_TIME_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidIrregularEventTimeArtifact);
        }
        Ok(())
    }
}

/// One completed irregular event-time artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct IrregularEventTimeExecution {
    /// Digest-bound completed composition artifact.
    pub artifact: IrregularEventTimeArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

struct EligibleOccasions {
    occasions: Vec<EventOccasion>,
    excluded_after_cutoff_count: u64,
}

fn admit_scores_at_cutoff(
    scores: &[IrregularEventScore],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<EligibleOccasions, AnalysisEngineError> {
    if scores.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    let mut eligible = Vec::new();
    let mut excluded_after_cutoff_count = 0_u64;
    for score in scores {
        if score.available_time.instant() <= knowledge_cutoff.instant() {
            eligible.push(EventOccasion {
                event_time: score.event_time,
                score: score.score,
            });
        } else {
            excluded_after_cutoff_count += 1;
        }
    }
    if eligible.len() < 2 {
        return Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput,
        ));
    }
    Ok(EligibleOccasions {
        occasions: eligible,
        excluded_after_cutoff_count,
    })
}

fn first_source_lag(occasions: &[EventOccasion]) -> Result<(f64, f64), AnalysisEngineError> {
    let mut ordered = occasions.to_vec();
    ordered.sort_by(|left, right| {
        left.event_time
            .partial_cmp(&right.event_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let earlier = ordered[0];
    let later = ordered[1];
    let source_delta = later.event_time - earlier.event_time;
    let discrete_lag = recover_discrete_lag_one(earlier.score, later.score)?;
    Ok((discrete_lag, source_delta))
}

/// Execute cutoff-safe irregular event-time composition as one analysis-run profile.
///
/// The caller supplies already-mapped event-time scores. This executor maps
/// discrete lags through the local log-rate `a` and refuses pooled discrete
/// coefficients across unequal intervals. It is not DSEM.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, psychometric
/// recovery failure, or invalid artifact error.
#[allow(
    clippy::too_many_arguments,
    reason = "audited cutoff, clock, pooling, and reference-interval gates"
)]
pub fn execute_irregular_event_time_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    scores: &[IrregularEventScore],
    lag_clock: LagClock,
    pool_unequal_intervals: bool,
    reference_delta: f64,
    completed_at: impl Into<String>,
) -> Result<IrregularEventTimeExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != IRREGULAR_EVENT_TIME_MODEL_CONTRACT_VERSION
        || request.output_profile != IRREGULAR_EVENT_TIME_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if !reference_delta.is_finite() || reference_delta <= 0.0 {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if pool_unequal_intervals {
        refuse_pooled_discrete_lag_across_unequal_intervals(1.0, 2.0)?;
    }

    let eligible = admit_scores_at_cutoff(scores, knowledge_cutoff)?;
    let mean_log_rate = recover_event_series_mean_log_rate(&eligible.occasions, lag_clock)?;
    let (source_lag, source_delta) = first_source_lag(&eligible.occasions)?;
    let mapped_reference_lag = map_discrete_lag_across_event_intervals(
        source_lag,
        source_delta,
        reference_delta,
        lag_clock,
    )?;
    let _ = claim_causal_effect(CausalHeuristic::TemporalPrecedence);

    let occasion_count = u64::try_from(eligible.occasions.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let interval_count = occasion_count
        .checked_sub(1)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    let artifact = IrregularEventTimeArtifact {
        schema_version: IRREGULAR_EVENT_TIME_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        occasion_count,
        interval_count,
        excluded_after_cutoff_count: eligible.excluded_after_cutoff_count,
        mean_log_rate,
        mapped_reference_lag,
        reference_delta,
        inference_status: IRREGULAR_EVENT_TIME_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "irregular_event_time",
        occasion_count,
        2,
        IRREGULAR_EVENT_TIME_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("irregular_event_time_artifact_{}", &digest[..16]),
        digest,
        IRREGULAR_EVENT_TIME_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(IrregularEventTimeExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        IRREGULAR_EVENT_TIME_ARTIFACT_BYTE_LIMIT, IRREGULAR_EVENT_TIME_ARTIFACT_SCHEMA_VERSION,
        IRREGULAR_EVENT_TIME_INFERENCE_STATUS, IrregularEventTimeArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> IrregularEventTimeArtifact {
        IrregularEventTimeArtifact {
            schema_version: IRREGULAR_EVENT_TIME_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            occasion_count: 3,
            interval_count: 2,
            excluded_after_cutoff_count: 0,
            mean_log_rate: -std::f64::consts::LN_2,
            mapped_reference_lag: 0.25,
            reference_delta: 2.0,
            inference_status: IRREGULAR_EVENT_TIME_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &IrregularEventTimeArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidIrregularEventTimeArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            IrregularEventTimeArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            IrregularEventTimeArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidIrregularEventTimeArtifact)
        );
        assert_eq!(
            IrregularEventTimeArtifact::from_json(
                &"x".repeat(IRREGULAR_EVENT_TIME_ARTIFACT_BYTE_LIMIT + 1)
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
                value.occasion_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.interval_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.interval_count = 4;
                value
            },
            {
                let mut value = artifact.clone();
                value.mean_log_rate = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.mapped_reference_lag = 0.0;
                value
            },
            {
                let mut value = artifact.clone();
                value.reference_delta = 0.0;
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
