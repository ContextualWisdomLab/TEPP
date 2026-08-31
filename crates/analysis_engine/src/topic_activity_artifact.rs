//! Digest-bound topic activity/dormancy/reactivation as an analysis-run profile.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};
use topic_lineage::{
    TopicIdentity, TopicLineageRecord, identity_recovery_rate, refuse_new_identity_on_reactivation,
};

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed topic-activity artifact.
pub const TOPIC_ACTIVITY_ARTIFACT_SCHEMA_VERSION: &str = "tepp.topic_activity.v1";
/// Model contract required by the topic-activity execution path.
pub const TOPIC_ACTIVITY_MODEL_CONTRACT_VERSION: &str = "topic_activity_v1";
/// Analysis-run output profile required for a topic-activity artifact.
pub const TOPIC_ACTIVITY_OUTPUT_PROFILE: &str = "topic_activity_v1";
/// Maximum canonical artifact JSON size.
pub const TOPIC_ACTIVITY_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const TOPIC_ACTIVITY_INFERENCE_STATUS: &str = "reactivation_is_not_new_topic_not_birth_split_merge";

/// One fail-closed activity transition applied to a durable topic identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopicActivityTransition {
    /// Move an active or reactivated topic into dormancy.
    MakeDormant,
    /// Reactivate a dormant topic without minting a new identity.
    Reactivate,
}

/// Cutoff-safe topic-activity input bound to one durable identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopicActivityInput {
    identity: TopicIdentity,
    transitions: Vec<TopicActivityTransition>,
    proposed_reactivation_identity: TopicIdentity,
    truth: Vec<TopicIdentity>,
    decided: Vec<TopicIdentity>,
}

impl TopicActivityInput {
    /// Construct an activity sequence for one durable topic identity.
    #[must_use]
    pub fn new(
        identity: TopicIdentity,
        transitions: Vec<TopicActivityTransition>,
        proposed_reactivation_identity: TopicIdentity,
        truth: Vec<TopicIdentity>,
        decided: Vec<TopicIdentity>,
    ) -> Self {
        Self {
            identity,
            transitions,
            proposed_reactivation_identity,
            truth,
            decided,
        }
    }

    /// Return the durable topic identity.
    #[must_use]
    pub const fn identity(&self) -> TopicIdentity {
        self.identity
    }

    /// Borrow the ordered activity transitions.
    #[must_use]
    pub fn transitions(&self) -> &[TopicActivityTransition] {
        &self.transitions
    }

    /// Return the identity proposed at reactivation.
    #[must_use]
    pub const fn proposed_reactivation_identity(&self) -> TopicIdentity {
        self.proposed_reactivation_identity
    }

    /// Borrow known-truth identities for recovery-rate scoring.
    #[must_use]
    pub fn truth(&self) -> &[TopicIdentity] {
        &self.truth
    }

    /// Borrow decided identities for recovery-rate scoring.
    #[must_use]
    pub fn decided(&self) -> &[TopicIdentity] {
        &self.decided
    }
}

/// Completed, bounded topic-activity result for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicActivityArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the activity sequence.
    pub knowledge_cutoff: String,
    /// Durable P0 topic identity that survived the sequence.
    pub topic_identity: String,
    /// Final activity wire name (`active`, `dormant`, or `reactivated`).
    pub activity: String,
    /// Number of applied activity transitions.
    pub transition_count: u64,
    /// Known-truth identity recovery rate in `[0, 1]`.
    pub identity_recovery_rate: f64,
    /// Whether reactivation preserved the incumbent identity.
    pub reactivation_identity_preserved: bool,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl TopicActivityArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidTopicActivityArtifact`] when the
    /// schema, identifiers, activity, rate, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > TOPIC_ACTIVITY_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidTopicActivityArtifact)?;
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
        if payload.len() > TOPIC_ACTIVITY_ARTIFACT_BYTE_LIMIT {
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
        if self.schema_version != TOPIC_ACTIVITY_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || uuid::Uuid::parse_str(&self.topic_identity).is_err()
            || !matches!(self.activity.as_str(), "active" | "dormant" | "reactivated")
            || !self.identity_recovery_rate.is_finite()
            || self.identity_recovery_rate < 0.0
            || self.identity_recovery_rate > 1.0
            || !self.reactivation_identity_preserved
            || self.inference_status != TOPIC_ACTIVITY_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidTopicActivityArtifact);
        }
        Ok(())
    }
}

/// One completed topic-activity artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct TopicActivityExecution {
    /// Digest-bound completed activity artifact.
    pub artifact: TopicActivityArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe topic activity/dormancy/reactivation as one analysis-run profile.
///
/// The executor invokes [`TopicLineageRecord`] transitions,
/// [`refuse_new_identity_on_reactivation`], and [`identity_recovery_rate`].
/// Reactivation cannot mint a new topic. This is not topic birth/split/merge,
/// not a Bayesian sampler, and not the fitted `trsl_topic_lineage_v1` profile.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, topic-lineage
/// refusal, or invalid artifact error.
pub fn execute_topic_activity_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &TopicActivityInput,
    completed_at: impl Into<String>,
) -> Result<TopicActivityExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != TOPIC_ACTIVITY_MODEL_CONTRACT_VERSION
        || request.output_profile != TOPIC_ACTIVITY_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let mut record = TopicLineageRecord::active(input.identity());
    for transition in input.transitions() {
        record = match transition {
            TopicActivityTransition::MakeDormant => record.make_dormant()?,
            TopicActivityTransition::Reactivate => record.reactivate()?,
        };
    }
    refuse_new_identity_on_reactivation(record.identity(), input.proposed_reactivation_identity())?;
    let recovery_rate = identity_recovery_rate(input.truth(), input.decided())?;
    let artifact = TopicActivityArtifact {
        schema_version: TOPIC_ACTIVITY_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        topic_identity: record.identity().as_uuid().to_string(),
        activity: record.activity().wire_name().to_owned(),
        transition_count: input.transitions().len() as u64,
        identity_recovery_rate: recovery_rate,
        reactivation_identity_preserved: true,
        inference_status: TOPIC_ACTIVITY_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary =
        AnalysisResultSummary::new("topic_activity", 1, 2, TOPIC_ACTIVITY_INFERENCE_STATUS)?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("topic_activity_artifact_{}", &digest[..16]),
        digest,
        TOPIC_ACTIVITY_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(TopicActivityExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        TOPIC_ACTIVITY_ARTIFACT_BYTE_LIMIT, TOPIC_ACTIVITY_ARTIFACT_SCHEMA_VERSION,
        TOPIC_ACTIVITY_INFERENCE_STATUS, TopicActivityArtifact, TopicActivityInput,
        TopicActivityTransition,
    };
    use crate::AnalysisEngineError;
    use topic_lineage::TopicIdentity;
    use uuid::Uuid;

    fn artifact() -> TopicActivityArtifact {
        TopicActivityArtifact {
            schema_version: TOPIC_ACTIVITY_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            topic_identity: Uuid::from_u128(11).to_string(),
            activity: "reactivated".into(),
            transition_count: 2,
            identity_recovery_rate: 1.0,
            reactivation_identity_preserved: true,
            inference_status: TOPIC_ACTIVITY_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &TopicActivityArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidTopicActivityArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            TopicActivityArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            TopicActivityArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidTopicActivityArtifact)
        );
        assert_eq!(
            TopicActivityArtifact::from_json(&"x".repeat(TOPIC_ACTIVITY_ARTIFACT_BYTE_LIMIT + 1)),
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
                value.topic_identity.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.activity = "birth".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.identity_recovery_rate = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.identity_recovery_rate = -0.1;
                value
            },
            {
                let mut value = artifact.clone();
                value.identity_recovery_rate = 1.5;
                value
            },
            {
                let mut value = artifact.clone();
                value.reactivation_identity_preserved = false;
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
    fn input_accessors_expose_identity_and_transitions() {
        let identity = TopicIdentity::from_uuid(Uuid::from_u128(11));
        let input = TopicActivityInput::new(
            identity,
            vec![
                TopicActivityTransition::MakeDormant,
                TopicActivityTransition::Reactivate,
            ],
            identity,
            vec![identity],
            vec![identity],
        );
        assert_eq!(input.identity(), identity);
        assert_eq!(input.transitions().len(), 2);
        assert_eq!(input.proposed_reactivation_identity(), identity);
        assert_eq!(input.truth(), &[identity]);
        assert_eq!(input.decided(), &[identity]);
    }
}
