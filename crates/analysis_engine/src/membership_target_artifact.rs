//! Digest-bound membership-target refusals as an analysis-run profile.

use membership_target::{refuse_collapsed_target, MembershipTargetError, MembershipTargetKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{format_digest, require_receipt_identity, valid_identifier, AnalysisEngineError};

/// Versioned schema for a completed membership-target artifact.
pub const MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION: &str = "tepp.membership_target.v1";
/// Model contract required by the membership-target execution path.
pub const MEMBERSHIP_TARGET_MODEL_CONTRACT_VERSION: &str = "membership_target_v1";
/// Analysis-run output profile required for a membership-target artifact.
pub const MEMBERSHIP_TARGET_OUTPUT_PROFILE: &str = "membership_target_v1";
/// Maximum canonical artifact JSON size.
pub const MEMBERSHIP_TARGET_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const MEMBERSHIP_TARGET_INFERENCE_STATUS: &str =
    "language_episode_template_department_opportunity_pool_are_not_entities";

/// One cutoff-admitted membership treatment with a closed target kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipTargetDocument {
    document_id: String,
    kind: MembershipTargetKind,
}

impl MembershipTargetDocument {
    /// Construct a bounded membership-target document.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the document
    /// identity is empty or oversized.
    pub fn new(
        document_id: impl Into<String>,
        kind: MembershipTargetKind,
    ) -> Result<Self, AnalysisEngineError> {
        let document_id = document_id.into();
        if !valid_identifier(&document_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self { document_id, kind })
    }

    /// Return the opaque document identity.
    #[must_use]
    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    /// Return the closed membership-target kind.
    #[must_use]
    pub const fn kind(&self) -> MembershipTargetKind {
        self.kind
    }
}

/// Completed, bounded membership-target census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipTargetArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used to admit documents.
    pub knowledge_cutoff: String,
    /// Number of documents admitted at the cutoff.
    pub document_count: u64,
    /// Language-community treatments admitted at the cutoff.
    pub language_count: u64,
    /// Episode treatments admitted at the cutoff.
    pub episode_count: u64,
    /// Template-family treatments admitted at the cutoff.
    pub template_count: u64,
    /// Department treatments admitted at the cutoff.
    pub department_count: u64,
    /// Opportunity-pool treatments admitted at the cutoff.
    pub opportunity_pool_count: u64,
    /// Entity treatments admitted at the cutoff.
    pub entity_count: u64,
    /// Project treatments admitted at the cutoff.
    pub project_count: u64,
    /// Typed non-entity/project kinds refused as entity.
    pub refused_as_entity_count: u64,
    /// Typed non-entity/project kinds refused as project.
    pub refused_as_project_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl MembershipTargetArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidMembershipTargetArtifact`] when
    /// the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > MEMBERSHIP_TARGET_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidMembershipTargetArtifact)?;
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
        if payload.len() > MEMBERSHIP_TARGET_ARTIFACT_BYTE_LIMIT {
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
        let typed_sum = self
            .language_count
            .checked_add(self.episode_count)
            .and_then(|value| value.checked_add(self.template_count))
            .and_then(|value| value.checked_add(self.department_count))
            .and_then(|value| value.checked_add(self.opportunity_pool_count));
        let persistence_sum = self.entity_count.checked_add(self.project_count);
        let kind_sum = typed_sum
            .and_then(|typed| persistence_sum.and_then(|persisted| typed.checked_add(persisted)));
        if self.schema_version != MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.document_count < 2
            || typed_sum == Some(0)
            || persistence_sum == Some(0)
            || kind_sum != Some(self.document_count)
            || self.refused_as_entity_count != typed_sum.unwrap_or(0)
            || self.refused_as_project_count != typed_sum.unwrap_or(0)
            || self.inference_status != MEMBERSHIP_TARGET_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidMembershipTargetArtifact);
        }
        Ok(())
    }
}

/// One completed membership-target artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct MembershipTargetExecution {
    /// Digest-bound completed membership-target census.
    pub artifact: MembershipTargetArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe membership-target refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_collapsed_target`] already on protected main.
/// Language, episode, template, department, and opportunity-pool kinds stay
/// distinct from entity and project. It does not emit `identity_recovery_rate`,
/// a `scientific_acceptance` inspect metric, GPU kernels, MCMC, or topic
/// birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-class corpus, duplicate document identity, or invalid artifact error.
#[allow(clippy::too_many_lines)]
pub fn execute_membership_target_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    documents: &[MembershipTargetDocument],
    completed_at: impl Into<String>,
) -> Result<MembershipTargetExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != MEMBERSHIP_TARGET_MODEL_CONTRACT_VERSION
        || request.output_profile != MEMBERSHIP_TARGET_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut language_count = 0_u64;
    let mut episode_count = 0_u64;
    let mut template_count = 0_u64;
    let mut department_count = 0_u64;
    let mut opportunity_pool_count = 0_u64;
    let mut entity_count = 0_u64;
    let mut project_count = 0_u64;
    let mut refused_as_entity_count = 0_u64;
    let mut refused_as_project_count = 0_u64;
    for document in documents {
        if !seen.insert(document.document_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        match document.kind() {
            MembershipTargetKind::Language
            | MembershipTargetKind::Episode
            | MembershipTargetKind::Template
            | MembershipTargetKind::Department
            | MembershipTargetKind::OpportunityPool => {
                match refuse_collapsed_target(document.kind(), MembershipTargetKind::Entity) {
                    Err(MembershipTargetError::TargetKindCollapsed) => {
                        refused_as_entity_count = refused_as_entity_count
                            .checked_add(1)
                            .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                match refuse_collapsed_target(document.kind(), MembershipTargetKind::Project) {
                    Err(MembershipTargetError::TargetKindCollapsed) => {
                        refused_as_project_count = refused_as_project_count
                            .checked_add(1)
                            .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                match document.kind() {
                    MembershipTargetKind::Language => {
                        language_count = increment(language_count)?;
                    }
                    MembershipTargetKind::Episode => {
                        episode_count = increment(episode_count)?;
                    }
                    MembershipTargetKind::Template => {
                        template_count = increment(template_count)?;
                    }
                    MembershipTargetKind::Department => {
                        department_count = increment(department_count)?;
                    }
                    MembershipTargetKind::OpportunityPool => {
                        opportunity_pool_count = increment(opportunity_pool_count)?;
                    }
                    MembershipTargetKind::Entity | MembershipTargetKind::Project => {
                        return Err(AnalysisEngineError::InvalidEvidence);
                    }
                }
            }
            MembershipTargetKind::Entity => {
                refuse_collapsed_target(document.kind(), MembershipTargetKind::Entity)
                    .map_err(map_membership_target_error)?;
                entity_count = increment(entity_count)?;
            }
            MembershipTargetKind::Project => {
                refuse_collapsed_target(document.kind(), MembershipTargetKind::Project)
                    .map_err(map_membership_target_error)?;
                project_count = increment(project_count)?;
            }
        }
    }

    let document_count =
        u64::try_from(documents.len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let typed_sum = language_count
        .checked_add(episode_count)
        .and_then(|value| value.checked_add(template_count))
        .and_then(|value| value.checked_add(department_count))
        .and_then(|value| value.checked_add(opportunity_pool_count))
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    let persistence_sum = entity_count
        .checked_add(project_count)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if document_count < 2 || typed_sum == 0 || persistence_sum == 0 {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = MembershipTargetArtifact {
        schema_version: MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        document_count,
        language_count,
        episode_count,
        template_count,
        department_count,
        opportunity_pool_count,
        entity_count,
        project_count,
        refused_as_entity_count,
        refused_as_project_count,
        inference_status: MEMBERSHIP_TARGET_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "membership_target",
        document_count,
        4,
        MEMBERSHIP_TARGET_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("membership_target_artifact_{}", &digest[..16]),
        digest,
        MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(MembershipTargetExecution {
        artifact,
        terminal_result,
    })
}

fn increment(count: u64) -> Result<u64, AnalysisEngineError> {
    count
        .checked_add(1)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)
}

fn map_membership_target_error(error: MembershipTargetError) -> AnalysisEngineError {
    match error {
        MembershipTargetError::TargetKindCollapsed
        | MembershipTargetError::InvalidTargetPayload
        | _ => AnalysisEngineError::InvalidEvidence,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MembershipTargetArtifact, MEMBERSHIP_TARGET_ARTIFACT_BYTE_LIMIT,
        MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION, MEMBERSHIP_TARGET_INFERENCE_STATUS,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> MembershipTargetArtifact {
        MembershipTargetArtifact {
            schema_version: MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            document_count: 7,
            language_count: 1,
            episode_count: 1,
            template_count: 1,
            department_count: 1,
            opportunity_pool_count: 1,
            entity_count: 1,
            project_count: 1,
            refused_as_entity_count: 5,
            refused_as_project_count: 5,
            inference_status: MEMBERSHIP_TARGET_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &MembershipTargetArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidMembershipTargetArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            MembershipTargetArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            MembershipTargetArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidMembershipTargetArtifact)
        );
        assert_eq!(
            MembershipTargetArtifact::from_json(
                &"x".repeat(MEMBERSHIP_TARGET_ARTIFACT_BYTE_LIMIT + 1)
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
                value.document_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.language_count = 0;
                value.episode_count = 0;
                value.template_count = 0;
                value.department_count = 0;
                value.opportunity_pool_count = 0;
                value.refused_as_entity_count = 0;
                value.refused_as_project_count = 0;
                value.document_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.entity_count = 0;
                value.project_count = 0;
                value.document_count = 5;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_entity_count = 0;
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
