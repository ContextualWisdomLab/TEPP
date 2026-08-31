//! Digest-bound location-membership refusals as an analysis-run profile.

use location_membership::{
    LocationKind, LocationMembershipError, refuse_location_as_entity_identity,
    refuse_location_as_language_channel,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed location-membership artifact.
pub const LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION: &str = "tepp.location_membership.v1";
/// Model contract required by the location-membership execution path.
pub const LOCATION_MEMBERSHIP_MODEL_CONTRACT_VERSION: &str = "location_membership_v1";
/// Analysis-run output profile required for a location-membership artifact.
pub const LOCATION_MEMBERSHIP_OUTPUT_PROFILE: &str = "location_membership_v1";
/// Maximum canonical artifact JSON size.
pub const LOCATION_MEMBERSHIP_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const LOCATION_MEMBERSHIP_INFERENCE_STATUS: &str =
    "location_is_not_entity_identity_not_language_channel";

/// One cutoff-admitted membership treatment with a closed location kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocationMembershipDocument {
    document_id: String,
    kind: LocationKind,
}

impl LocationMembershipDocument {
    /// Construct a bounded location-membership document.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the document
    /// identity is empty or oversized.
    pub fn new(
        document_id: impl Into<String>,
        kind: LocationKind,
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

    /// Return the closed location kind.
    #[must_use]
    pub const fn kind(&self) -> LocationKind {
        self.kind
    }
}

/// Completed, bounded location-membership census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocationMembershipArtifact {
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
    /// Time-varying location memberships admitted at the cutoff.
    pub location_count: u64,
    /// Permanent entity-identity treatments admitted at the cutoff.
    pub entity_identity_count: u64,
    /// Language-channel treatments admitted at the cutoff.
    pub language_channel_count: u64,
    /// Location memberships refused as permanent entity identity.
    pub refused_as_entity_identity_count: u64,
    /// Location memberships refused as a language channel.
    pub refused_as_language_channel_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl LocationMembershipArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidLocationMembershipArtifact`] when
    /// the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > LOCATION_MEMBERSHIP_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidLocationMembershipArtifact)?;
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
        if payload.len() > LOCATION_MEMBERSHIP_ARTIFACT_BYTE_LIMIT {
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
            .location_count
            .checked_add(self.entity_identity_count)
            .and_then(|value| value.checked_add(self.language_channel_count));
        let non_location = self
            .entity_identity_count
            .checked_add(self.language_channel_count);
        if self.schema_version != LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.document_count < 2
            || self.location_count == 0
            || non_location == Some(0)
            || kind_sum != Some(self.document_count)
            || self.refused_as_entity_identity_count != self.location_count
            || self.refused_as_language_channel_count != self.location_count
            || self.inference_status != LOCATION_MEMBERSHIP_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidLocationMembershipArtifact);
        }
        Ok(())
    }
}

/// One completed location-membership artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct LocationMembershipExecution {
    /// Digest-bound completed location-membership census.
    pub artifact: LocationMembershipArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe location-membership refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_location_as_entity_identity`] and
/// [`refuse_location_as_language_channel`] already on protected main.
/// It does not emit `identity_recovery_rate`, a `scientific_acceptance`
/// inspect metric, GPU kernels, MCMC, or topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-kind corpus, duplicate document identity, or invalid artifact error.
#[allow(clippy::too_many_lines)]
pub fn execute_location_membership_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    documents: &[LocationMembershipDocument],
    completed_at: impl Into<String>,
) -> Result<LocationMembershipExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != LOCATION_MEMBERSHIP_MODEL_CONTRACT_VERSION
        || request.output_profile != LOCATION_MEMBERSHIP_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut location_count = 0_u64;
    let mut entity_identity_count = 0_u64;
    let mut language_channel_count = 0_u64;
    let mut refused_as_entity_identity_count = 0_u64;
    let mut refused_as_language_channel_count = 0_u64;
    for document in documents {
        if !seen.insert(document.document_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        match document.kind() {
            LocationKind::Location => {
                match refuse_location_as_entity_identity(document.kind()) {
                    Err(LocationMembershipError::LocationIsNotEntityIdentity) => {
                        refused_as_entity_identity_count = refused_as_entity_identity_count
                            .checked_add(1)
                            .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                match refuse_location_as_language_channel(document.kind()) {
                    Err(LocationMembershipError::LocationIsNotLanguageChannel) => {
                        refused_as_language_channel_count = refused_as_language_channel_count
                            .checked_add(1)
                            .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                location_count = location_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            LocationKind::EntityIdentity => {
                refuse_location_as_entity_identity(document.kind()).map_err(map_location_error)?;
                refuse_location_as_language_channel(document.kind()).map_err(map_location_error)?;
                entity_identity_count = entity_identity_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            LocationKind::LanguageChannel => {
                refuse_location_as_entity_identity(document.kind()).map_err(map_location_error)?;
                refuse_location_as_language_channel(document.kind()).map_err(map_location_error)?;
                language_channel_count = language_channel_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
        }
    }
    let document_count =
        u64::try_from(documents.len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    if document_count < 2
        || location_count == 0
        || entity_identity_count
            .checked_add(language_channel_count)
            .unwrap_or(0)
            == 0
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = LocationMembershipArtifact {
        schema_version: LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        document_count,
        location_count,
        entity_identity_count,
        language_channel_count,
        refused_as_entity_identity_count,
        refused_as_language_channel_count,
        inference_status: LOCATION_MEMBERSHIP_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "location_membership",
        document_count,
        4,
        LOCATION_MEMBERSHIP_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("location_membership_artifact_{}", &digest[..16]),
        digest,
        LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(LocationMembershipExecution {
        artifact,
        terminal_result,
    })
}

fn map_location_error(error: LocationMembershipError) -> AnalysisEngineError {
    match error {
        LocationMembershipError::LocationIsNotEntityIdentity
        | LocationMembershipError::LocationIsNotLanguageChannel
        | LocationMembershipError::InvalidLocationPayload
        | _ => AnalysisEngineError::InvalidEvidence,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LOCATION_MEMBERSHIP_ARTIFACT_BYTE_LIMIT, LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION,
        LOCATION_MEMBERSHIP_INFERENCE_STATUS, LocationMembershipArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> LocationMembershipArtifact {
        LocationMembershipArtifact {
            schema_version: LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            document_count: 3,
            location_count: 1,
            entity_identity_count: 1,
            language_channel_count: 1,
            refused_as_entity_identity_count: 1,
            refused_as_language_channel_count: 1,
            inference_status: LOCATION_MEMBERSHIP_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &LocationMembershipArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidLocationMembershipArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            LocationMembershipArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            LocationMembershipArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidLocationMembershipArtifact)
        );
        assert_eq!(
            LocationMembershipArtifact::from_json(
                &"x".repeat(LOCATION_MEMBERSHIP_ARTIFACT_BYTE_LIMIT + 1)
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
                value.location_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.entity_identity_count = 0;
                value.language_channel_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_entity_identity_count = 0;
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
