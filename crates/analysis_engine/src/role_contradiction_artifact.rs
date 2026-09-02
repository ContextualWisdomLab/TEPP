//! Digest-bound role-contradiction refusals as an analysis-run profile.

use std::collections::{BTreeMap, BTreeSet};

use role_contradiction::{
    ContextualRole, RoleContradictionError, refuse_contradictory_roles, refuse_role_as_entity_class,
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

/// Versioned schema for a completed role-contradiction artifact.
pub const ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION: &str = "tepp.role_contradiction.v1";
/// Model contract required by the role-contradiction execution path.
pub const ROLE_CONTRADICTION_MODEL_CONTRACT_VERSION: &str = "role_contradiction_v1";
/// Analysis-run output profile required for a role-contradiction artifact.
pub const ROLE_CONTRADICTION_OUTPUT_PROFILE: &str = "role_contradiction_v1";
/// Maximum canonical artifact JSON size.
pub const ROLE_CONTRADICTION_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const ROLE_CONTRADICTION_INFERENCE_STATUS: &str =
    "customer_competitor_cannot_share_group_role_is_not_entity_class";

/// One cutoff-admitted contextual-role assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleContradictionAssignment {
    assignment_id: String,
    group_id: String,
    role: ContextualRole,
    available_time: AvailableTime,
}

impl RoleContradictionAssignment {
    /// Construct a bounded role assignment.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the assignment or
    /// group identity is empty or oversized.
    pub fn new(
        assignment_id: impl Into<String>,
        group_id: impl Into<String>,
        role: ContextualRole,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let assignment_id = assignment_id.into();
        let group_id = group_id.into();
        if !valid_identifier(&assignment_id) || !valid_identifier(&group_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            assignment_id,
            group_id,
            role,
            available_time,
        })
    }

    /// Return the opaque assignment identity.
    #[must_use]
    pub fn assignment_id(&self) -> &str {
        &self.assignment_id
    }

    /// Return the opaque group identity.
    #[must_use]
    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    /// Return the closed contextual role.
    #[must_use]
    pub const fn role(&self) -> ContextualRole {
        self.role
    }

    /// Return the availability time used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded role-contradiction census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleContradictionArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used to admit assignments.
    pub knowledge_cutoff: String,
    /// Number of assignments admitted at the cutoff.
    pub assignment_count: u64,
    /// Customer-role assignments admitted at the cutoff.
    pub customer_count: u64,
    /// Partner-role assignments admitted at the cutoff.
    pub partner_count: u64,
    /// Competitor-role assignments admitted at the cutoff.
    pub competitor_count: u64,
    /// Assignments refused as a permanent entity class.
    pub refused_as_entity_class_count: u64,
    /// Customer/competitor pairs refused inside one group.
    pub refused_contradictory_pair_count: u64,
    /// Compatible unique-role pairs counted inside groups.
    pub compatible_pair_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl RoleContradictionArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidRoleContradictionArtifact`] when
    /// the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > ROLE_CONTRADICTION_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidRoleContradictionArtifact)?;
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
        if payload.len() > ROLE_CONTRADICTION_ARTIFACT_BYTE_LIMIT {
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
        let role_sum = self
            .customer_count
            .checked_add(self.partner_count)
            .and_then(|value| value.checked_add(self.competitor_count));
        if self.schema_version != ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.assignment_count < 3
            || self.assignment_count > MAX_EVIDENCE_UNITS as u64
            || self.customer_count == 0
            || self.partner_count == 0
            || self.competitor_count == 0
            || role_sum != Some(self.assignment_count)
            || self.refused_as_entity_class_count != self.assignment_count
            || self.refused_contradictory_pair_count == 0
            || self.compatible_pair_count == 0
            || self.inference_status != ROLE_CONTRADICTION_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidRoleContradictionArtifact);
        }
        Ok(())
    }
}

/// One completed role-contradiction artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct RoleContradictionExecution {
    /// Digest-bound completed role-contradiction census.
    pub artifact: RoleContradictionArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe role-contradiction refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_contradictory_roles`] and
/// [`refuse_role_as_entity_class`] already on protected main. Customer and
/// competitor cannot share a group. A contextual role is never a permanent
/// entity class. It does not emit `identity_recovery_rate`, a
/// `scientific_acceptance` inspect metric, GPU kernels, MCMC, or topic
/// birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-class corpus, missing contradiction or compatible pair, duplicate
/// assignment identity, oversized corpus, or invalid artifact error.
pub fn execute_role_contradiction_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    assignments: &[RoleContradictionAssignment],
    completed_at: impl Into<String>,
) -> Result<RoleContradictionExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != ROLE_CONTRADICTION_MODEL_CONTRACT_VERSION
        || request.output_profile != ROLE_CONTRADICTION_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if assignments.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let census = census_admitted_assignments(assignments, knowledge_cutoff)?;
    let artifact = RoleContradictionArtifact {
        schema_version: ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        assignment_count: census.assignment_count,
        customer_count: census.customer_count,
        partner_count: census.partner_count,
        competitor_count: census.competitor_count,
        refused_as_entity_class_count: census.refused_as_entity_class_count,
        refused_contradictory_pair_count: census.refused_contradictory_pair_count,
        compatible_pair_count: census.compatible_pair_count,
        inference_status: ROLE_CONTRADICTION_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "role_contradiction",
        census.assignment_count,
        4,
        "validated",
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("role_contradiction_artifact_{}", &digest[..16]),
        digest,
        ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(RoleContradictionExecution {
        artifact,
        terminal_result,
    })
}

#[allow(clippy::struct_field_names)]
struct RoleContradictionCensus {
    assignment_count: u64,
    customer_count: u64,
    partner_count: u64,
    competitor_count: u64,
    refused_as_entity_class_count: u64,
    refused_contradictory_pair_count: u64,
    compatible_pair_count: u64,
}

fn census_admitted_assignments(
    assignments: &[RoleContradictionAssignment],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<RoleContradictionCensus, AnalysisEngineError> {
    let mut seen = BTreeSet::new();
    let mut groups: BTreeMap<&str, BTreeSet<&'static str>> = BTreeMap::new();
    let mut customer_count = 0_u64;
    let mut partner_count = 0_u64;
    let mut competitor_count = 0_u64;
    let mut refused_as_entity_class_count = 0_u64;
    for assignment in assignments {
        if assignment.available_time().instant() > knowledge_cutoff.instant() {
            continue;
        }
        if !seen.insert(assignment.assignment_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        match refuse_role_as_entity_class(assignment.role()) {
            Err(RoleContradictionError::RoleIsNotEntityClass) => {
                refused_as_entity_class_count = increment(refused_as_entity_class_count)?;
            }
            Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
        }
        match assignment.role() {
            ContextualRole::Customer => customer_count = increment(customer_count)?,
            ContextualRole::Partner => partner_count = increment(partner_count)?,
            ContextualRole::Competitor => competitor_count = increment(competitor_count)?,
        }
        groups
            .entry(assignment.group_id())
            .or_default()
            .insert(assignment.role().wire_name());
    }

    let (refused_contradictory_pair_count, compatible_pair_count) = count_group_pairs(&groups)?;
    let assignment_count = customer_count
        .checked_add(partner_count)
        .and_then(|value| value.checked_add(competitor_count))
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if assignment_count < 3
        || customer_count == 0
        || partner_count == 0
        || competitor_count == 0
        || refused_as_entity_class_count != assignment_count
        || refused_contradictory_pair_count == 0
        || compatible_pair_count == 0
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    Ok(RoleContradictionCensus {
        assignment_count,
        customer_count,
        partner_count,
        competitor_count,
        refused_as_entity_class_count,
        refused_contradictory_pair_count,
        compatible_pair_count,
    })
}

fn count_group_pairs(
    groups: &BTreeMap<&str, BTreeSet<&'static str>>,
) -> Result<(u64, u64), AnalysisEngineError> {
    let mut refused_contradictory_pair_count = 0_u64;
    let mut compatible_pair_count = 0_u64;
    for role_names in groups.values() {
        let roles: Result<Vec<ContextualRole>, AnalysisEngineError> = role_names
            .iter()
            .map(|name| {
                ContextualRole::from_wire_name(name)
                    .map_err(|_| AnalysisEngineError::InvalidEvidence)
            })
            .collect();
        let roles = roles?;
        for left_index in 0..roles.len() {
            for right in roles.iter().skip(left_index + 1) {
                match refuse_contradictory_roles(roles[left_index], *right) {
                    Err(RoleContradictionError::CustomerCompetitorOverlap) => {
                        refused_contradictory_pair_count =
                            increment(refused_contradictory_pair_count)?;
                    }
                    Ok(()) => compatible_pair_count = increment(compatible_pair_count)?,
                    Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
            }
        }
    }
    Ok((refused_contradictory_pair_count, compatible_pair_count))
}

fn increment(count: u64) -> Result<u64, AnalysisEngineError> {
    count
        .checked_add(1)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::{
        ROLE_CONTRADICTION_ARTIFACT_BYTE_LIMIT, ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION,
        ROLE_CONTRADICTION_INFERENCE_STATUS, RoleContradictionArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> RoleContradictionArtifact {
        RoleContradictionArtifact {
            schema_version: ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            assignment_count: 3,
            customer_count: 1,
            partner_count: 1,
            competitor_count: 1,
            refused_as_entity_class_count: 3,
            refused_contradictory_pair_count: 1,
            compatible_pair_count: 2,
            inference_status: ROLE_CONTRADICTION_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &RoleContradictionArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidRoleContradictionArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            RoleContradictionArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            RoleContradictionArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidRoleContradictionArtifact)
        );
        assert_eq!(
            RoleContradictionArtifact::from_json(
                &"x".repeat(ROLE_CONTRADICTION_ARTIFACT_BYTE_LIMIT + 1)
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
                value.assignment_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.customer_count = 0;
                value.assignment_count = 2;
                value.refused_as_entity_class_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.partner_count = 0;
                value.assignment_count = 2;
                value.refused_as_entity_class_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.competitor_count = 0;
                value.assignment_count = 2;
                value.refused_as_entity_class_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_entity_class_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_contradictory_pair_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.compatible_pair_count = 0;
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
