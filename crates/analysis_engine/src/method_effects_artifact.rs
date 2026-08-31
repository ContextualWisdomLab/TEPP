//! Digest-bound simulation method-effect labels as an analysis-run profile.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};
use tepp_simulation::{
    DocumentMethodEffect, SimulationConfig, SimulationError, generate, refuse_unavailable_document,
};

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed method-effects artifact.
pub const METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION: &str = "tepp.method_effects.v1";
/// Model contract required by the method-effects execution path.
pub const METHOD_EFFECTS_MODEL_CONTRACT_VERSION: &str = "method_effects_v1";
/// Analysis-run output profile required for a method-effects artifact.
pub const METHOD_EFFECTS_OUTPUT_PROFILE: &str = "method_effects_v1";
/// Maximum canonical artifact JSON size.
pub const METHOD_EFFECTS_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const METHOD_EFFECTS_INFERENCE_STATUS: &str = "simulation_method_effect_labels_not_estimator_model";

/// Completed, bounded method-effect census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MethodEffectsArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used to admit documents.
    pub knowledge_cutoff: String,
    /// Explicit simulation seed.
    pub seed: u64,
    /// Digest of the simulation configuration.
    pub config_digest: String,
    /// Digest of the generated truth rows.
    pub content_digest: String,
    /// Number of documents available at the cutoff.
    pub document_count: u64,
    /// Original reports admitted at the cutoff.
    pub original_count: u64,
    /// Revision variants admitted at the cutoff.
    pub revision_count: u64,
    /// Translation variants admitted at the cutoff.
    pub translation_count: u64,
    /// Template-copy variants admitted at the cutoff.
    pub template_copy_count: u64,
    /// Derivative (non-original) documents admitted at the cutoff.
    pub derivative_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl MethodEffectsArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidMethodEffectsArtifact`] when the
    /// schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > METHOD_EFFECTS_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidMethodEffectsArtifact)?;
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
        if payload.len() > METHOD_EFFECTS_ARTIFACT_BYTE_LIMIT {
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
        let kind_sum = self
            .original_count
            .checked_add(self.revision_count)
            .and_then(|value| value.checked_add(self.translation_count))
            .and_then(|value| value.checked_add(self.template_copy_count));
        let derivative_sum = self
            .revision_count
            .checked_add(self.translation_count)
            .and_then(|value| value.checked_add(self.template_copy_count));
        if self.schema_version != METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || !valid_identifier(&self.config_digest)
            || self.config_digest.len() != 64
            || !valid_identifier(&self.content_digest)
            || self.content_digest.len() != 64
            || self.document_count < 2
            || self.original_count == 0
            || kind_sum != Some(self.document_count)
            || derivative_sum != Some(self.derivative_count)
            || self.inference_status != METHOD_EFFECTS_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidMethodEffectsArtifact);
        }
        Ok(())
    }
}

/// One completed method-effects artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct MethodEffectsExecution {
    /// Digest-bound completed method-effect census.
    pub artifact: MethodEffectsArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe simulation method-effect labels as one analysis-run profile.
///
/// The executor invokes [`generate`] and [`refuse_unavailable_document`] already
/// on protected main. It does not invent an estimator-side method model, GPU
/// kernels, MCMC, or topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, simulation failure,
/// empty/undersized available corpus, or invalid artifact error.
pub fn execute_method_effects_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    config: SimulationConfig,
    completed_at: impl Into<String>,
) -> Result<MethodEffectsExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != METHOD_EFFECTS_MODEL_CONTRACT_VERSION
        || request.output_profile != METHOD_EFFECTS_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let manifest = generate(config).map_err(map_simulation_error)?;
    let mut original_count = 0_u64;
    let mut revision_count = 0_u64;
    let mut translation_count = 0_u64;
    let mut template_copy_count = 0_u64;
    let mut derivative_count = 0_u64;
    let mut document_count = 0_u64;
    for document in manifest.documents() {
        if refuse_unavailable_document(document, &knowledge_cutoff).is_err() {
            continue;
        }
        document_count = document_count
            .checked_add(1)
            .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
        match document.method_effect() {
            DocumentMethodEffect::Original => {
                original_count = original_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            DocumentMethodEffect::Revision => {
                revision_count = revision_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                derivative_count = derivative_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            DocumentMethodEffect::Translation => {
                translation_count = translation_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                derivative_count = derivative_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            DocumentMethodEffect::TemplateCopy => {
                template_copy_count = template_copy_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                derivative_count = derivative_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            _ => return Err(AnalysisEngineError::InvalidEvidence),
        }
    }
    if document_count < 2 || original_count == 0 {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = MethodEffectsArtifact {
        schema_version: METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        seed: manifest.seed(),
        config_digest: manifest.config_digest().to_owned(),
        content_digest: manifest.content_digest().to_owned(),
        document_count,
        original_count,
        revision_count,
        translation_count,
        template_copy_count,
        derivative_count,
        inference_status: METHOD_EFFECTS_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "method_effects",
        document_count,
        6,
        METHOD_EFFECTS_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("method_effects_artifact_{}", &digest[..16]),
        digest,
        METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(MethodEffectsExecution {
        artifact,
        terminal_result,
    })
}

fn map_simulation_error(error: SimulationError) -> AnalysisEngineError {
    match error {
        SimulationError::InvalidConfiguration
        | SimulationError::TemporalInvariantViolation
        | SimulationError::ManifestInvariantViolation
        | _ => AnalysisEngineError::InvalidEvidence,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        METHOD_EFFECTS_ARTIFACT_BYTE_LIMIT, METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION,
        METHOD_EFFECTS_INFERENCE_STATUS, MethodEffectsArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> MethodEffectsArtifact {
        MethodEffectsArtifact {
            schema_version: METHOD_EFFECTS_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            seed: 7,
            config_digest: "a".repeat(64),
            content_digest: "b".repeat(64),
            document_count: 4,
            original_count: 2,
            revision_count: 1,
            translation_count: 1,
            template_copy_count: 0,
            derivative_count: 2,
            inference_status: METHOD_EFFECTS_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &MethodEffectsArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidMethodEffectsArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            MethodEffectsArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            MethodEffectsArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidMethodEffectsArtifact)
        );
        assert_eq!(
            MethodEffectsArtifact::from_json(&"x".repeat(METHOD_EFFECTS_ARTIFACT_BYTE_LIMIT + 1)),
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
                value.config_digest.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.content_digest = "short".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.document_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.original_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.derivative_count = 0;
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
