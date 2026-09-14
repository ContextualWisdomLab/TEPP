//! Digest-bound joint Gaussian Laplace plausible-value draws as an analysis-run profile.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};
use topic_measurement::{
    JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION, ReferenceTopicInput, ReferenceTopicModelConfig,
    fit_reference_topic_model,
};
use uuid::Uuid;

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed joint-posterior-draw artifact.
pub const JOINT_POSTERIOR_DRAWS_ARTIFACT_SCHEMA_VERSION: &str = "tepp.joint_posterior_draws.v1";
/// Model contract required by the joint-posterior-draw execution path.
pub const JOINT_POSTERIOR_DRAWS_MODEL_CONTRACT_VERSION: &str = "joint_posterior_draws_v1";
/// Analysis-run output profile required for a joint-posterior-draw artifact.
pub const JOINT_POSTERIOR_DRAWS_OUTPUT_PROFILE: &str = "joint_posterior_draws_v1";
/// Maximum canonical artifact JSON size.
pub const JOINT_POSTERIOR_DRAWS_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const JOINT_POSTERIOR_DRAWS_INFERENCE_STATUS: &str =
    "joint_gaussian_laplace_plausible_values_not_mcmc";
const JOINT_GAUSS_NEWTON_LAPLACE: &str = "joint_gauss_newton_laplace";

/// Completed, bounded joint posterior Laplace draws for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JointPosteriorDrawsArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the fit and draws.
    pub knowledge_cutoff: String,
    /// SHA-256 identity binding algorithm, seed, basis, fit, and draws.
    pub draw_set_id: String,
    /// Stable counter-based draw algorithm identity.
    pub algorithm_version: String,
    /// Explicit Philox seed used for the draw set.
    pub seed: u64,
    /// Number of joint Gaussian draws materialized.
    pub draw_count: u64,
    /// Number of modeled evidence documents.
    pub document_count: u64,
    /// Number of global topics in the fitted model.
    pub topic_count: u64,
    /// Laplace approximation identity (not MCMC).
    pub approximation: String,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl JointPosteriorDrawsArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidJointPosteriorDrawsArtifact`] when
    /// the schema, identifiers, counts, algorithm, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > JOINT_POSTERIOR_DRAWS_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidJointPosteriorDrawsArtifact)?;
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
        if payload.len() > JOINT_POSTERIOR_DRAWS_ARTIFACT_BYTE_LIMIT {
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
        if self.schema_version != JOINT_POSTERIOR_DRAWS_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || !valid_identifier(&self.draw_set_id)
            || self.draw_set_id.len() != 64
            || self.algorithm_version != JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION
            || self.draw_count == 0
            || self.document_count < 2
            || self.topic_count < 2
            || self.approximation != JOINT_GAUSS_NEWTON_LAPLACE
            || self.inference_status != JOINT_POSTERIOR_DRAWS_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidJointPosteriorDrawsArtifact);
        }
        Ok(())
    }
}

/// One completed joint-posterior-draw artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct JointPosteriorDrawsExecution {
    /// Digest-bound completed draw-set artifact.
    pub artifact: JointPosteriorDrawsArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe joint Gaussian Laplace draws as one analysis-run profile.
///
/// The executor fits the CPU `f64` TRSL-TM reference, builds the identified
/// joint Gauss-Newton Laplace precision, and invokes
/// [`topic_measurement::JointCoordinatePrecision::draw_joint_gaussian`]. It
/// does not invent MCMC, select GPU backends, score candidate `K`, or emit
/// topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, estimator failure,
/// invalid draw request, or invalid artifact error.
#[allow(clippy::too_many_arguments)]
pub fn execute_joint_posterior_draws_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &ReferenceTopicInput,
    config: &ReferenceTopicModelConfig,
    topic_ids: Vec<Uuid>,
    draw_count: usize,
    completed_at: impl Into<String>,
) -> Result<JointPosteriorDrawsExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != JOINT_POSTERIOR_DRAWS_MODEL_CONTRACT_VERSION
        || request.output_profile != JOINT_POSTERIOR_DRAWS_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let model = fit_reference_topic_model(input, config)?;
    let precision = input.build_joint_coordinate_precision(&model, config, topic_ids)?;
    let draws = precision.draw_joint_gaussian(model.seed, draw_count)?;
    let document_count = u64::try_from(draws.document_ids().len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let topic_count = u64::try_from(draws.topic_ids().len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let draw_count =
        u64::try_from(draws.draws().len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let artifact = JointPosteriorDrawsArtifact {
        schema_version: JOINT_POSTERIOR_DRAWS_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        draw_set_id: draws.draw_set_id().to_owned(),
        algorithm_version: JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION.into(),
        seed: draws.seed(),
        draw_count,
        document_count,
        topic_count,
        approximation: JOINT_GAUSS_NEWTON_LAPLACE.into(),
        inference_status: JOINT_POSTERIOR_DRAWS_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "joint_posterior_draws",
        document_count,
        4,
        JOINT_POSTERIOR_DRAWS_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("joint_posterior_draws_artifact_{}", &digest[..16]),
        digest,
        JOINT_POSTERIOR_DRAWS_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(JointPosteriorDrawsExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        JOINT_POSTERIOR_DRAWS_ARTIFACT_BYTE_LIMIT, JOINT_POSTERIOR_DRAWS_ARTIFACT_SCHEMA_VERSION,
        JOINT_POSTERIOR_DRAWS_INFERENCE_STATUS, JointPosteriorDrawsArtifact,
    };
    use crate::AnalysisEngineError;
    use topic_measurement::JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION;

    fn artifact() -> JointPosteriorDrawsArtifact {
        JointPosteriorDrawsArtifact {
            schema_version: JOINT_POSTERIOR_DRAWS_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            draw_set_id: "a".repeat(64),
            algorithm_version: JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION.into(),
            seed: 7,
            draw_count: 4,
            document_count: 4,
            topic_count: 2,
            approximation: "joint_gauss_newton_laplace".into(),
            inference_status: JOINT_POSTERIOR_DRAWS_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &JointPosteriorDrawsArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidJointPosteriorDrawsArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            JointPosteriorDrawsArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            JointPosteriorDrawsArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidJointPosteriorDrawsArtifact)
        );
        assert_eq!(
            JointPosteriorDrawsArtifact::from_json(
                &"x".repeat(JOINT_POSTERIOR_DRAWS_ARTIFACT_BYTE_LIMIT + 1)
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
                value.draw_set_id.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.algorithm_version.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.draw_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.document_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.topic_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.approximation.clear();
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
