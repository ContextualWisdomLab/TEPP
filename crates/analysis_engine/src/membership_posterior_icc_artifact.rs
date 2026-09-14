//! Digest-bound nested ICC of posterior coordinates under classified membership.

use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipDesign, MembershipError, MembershipNetwork,
    MembershipRole, MembershipWeight, NestedOutcome, classify_membership_design,
    kish_effective_sample_size, nested_intraclass_correlation,
};
use psychometric_core::posterior_draw_point_estimate_mean;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, format_digest, require_receipt_identity,
    valid_identifier,
};

/// Versioned schema for a completed membership-posterior ICC artifact.
pub const MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_SCHEMA_VERSION: &str =
    "tepp.membership_posterior_icc.v1";
/// Model contract required by the membership-posterior ICC execution path.
pub const MEMBERSHIP_POSTERIOR_ICC_MODEL_CONTRACT_VERSION: &str = "membership_posterior_icc_v1";
/// Analysis-run output profile required for a membership-posterior ICC artifact.
pub const MEMBERSHIP_POSTERIOR_ICC_OUTPUT_PROFILE: &str = "membership_posterior_icc_v1";
/// Maximum canonical artifact JSON size.
pub const MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const NESTED_INFERENCE_STATUS: &str = "nested_icc_of_posterior_means_not_mmmc";
const MULTIPLE_MEMBERSHIP_INFERENCE_STATUS: &str =
    "multiple_membership_preserved_nested_icc_refused";
const CROSS_CLASSIFIED_INFERENCE_STATUS: &str = "cross_classified_preserved_nested_icc_refused";
const DESIGN_NESTED: &str = "nested";
const DESIGN_MULTIPLE_MEMBERSHIP: &str = "multiple_membership";
const DESIGN_CROSS_CLASSIFIED: &str = "cross_classified";

/// One posterior-draw member offered with a time-varying membership assignment.
#[derive(Clone, Debug, PartialEq)]
pub struct MembershipPosteriorObservation {
    assignment: MembershipAssignment,
    posterior_draws: Vec<f64>,
    available_time: AvailableTime,
}

impl MembershipPosteriorObservation {
    /// Bind posterior draws to one membership assignment and availability clock.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when draws are empty or
    /// non-finite.
    pub fn new(
        assignment: MembershipAssignment,
        posterior_draws: Vec<f64>,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        if posterior_draws.is_empty() || posterior_draws.iter().any(|value| !value.is_finite()) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            assignment,
            posterior_draws,
            available_time,
        })
    }

    /// Return the opaque member identity.
    #[must_use]
    pub const fn member_id(&self) -> MemberId {
        self.assignment.member_id()
    }

    /// Return the opaque group identity.
    #[must_use]
    pub const fn group_id(&self) -> GroupId {
        self.assignment.group_id()
    }

    /// Return the membership role.
    #[must_use]
    pub const fn role(&self) -> MembershipRole {
        self.assignment.role()
    }

    /// Return the membership weight.
    #[must_use]
    pub const fn weight(&self) -> MembershipWeight {
        self.assignment.weight()
    }

    /// Return the posterior draws used for the point estimate.
    #[must_use]
    pub fn posterior_draws(&self) -> &[f64] {
        &self.posterior_draws
    }

    /// Return the availability clock used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded membership-posterior ICC composition for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipPosteriorIccArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the composition.
    pub knowledge_cutoff: String,
    /// Event-time instant at which membership design is classified.
    pub classification_instant: String,
    /// Classified membership design without collapsing multiple membership.
    pub membership_design: String,
    /// Distinct members admitted after cutoff and activity filters.
    pub eligible_member_count: u64,
    /// Membership assignments admitted after cutoff and activity filters.
    pub eligible_assignment_count: u64,
    /// Observations excluded because availability was after the cutoff.
    pub excluded_after_cutoff_count: u64,
    /// Nested ANOVA ICC of posterior means, or `null` when the design refuses it.
    pub nested_icc: Option<f64>,
    /// Kish effective sample size of admitted membership weights.
    pub kish_ess: f64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl MembershipPosteriorIccArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidMembershipPosteriorIccArtifact`]
    /// when the schema, identifiers, design, ICC, ESS, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidMembershipPosteriorIccArtifact)?;
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
        if payload.len() > MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_BYTE_LIMIT {
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
        if self.schema_version != MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || EventTime::parse_rfc3339(&self.classification_instant).is_err()
            || self.eligible_member_count == 0
            || self.eligible_assignment_count == 0
            || !self.kish_ess.is_finite()
            || self.kish_ess <= 0.0
        {
            return Err(AnalysisEngineError::InvalidMembershipPosteriorIccArtifact);
        }
        let design_ok = match self.membership_design.as_str() {
            DESIGN_NESTED => {
                matches!(self.nested_icc, Some(value) if value.is_finite() && (0.0..=1.0).contains(&value))
                    && self.inference_status == NESTED_INFERENCE_STATUS
            }
            DESIGN_MULTIPLE_MEMBERSHIP => {
                self.nested_icc.is_none()
                    && self.inference_status == MULTIPLE_MEMBERSHIP_INFERENCE_STATUS
            }
            DESIGN_CROSS_CLASSIFIED => {
                self.nested_icc.is_none()
                    && self.inference_status == CROSS_CLASSIFIED_INFERENCE_STATUS
            }
            _ => false,
        };
        if !design_ok {
            return Err(AnalysisEngineError::InvalidMembershipPosteriorIccArtifact);
        }
        Ok(())
    }
}

/// One completed membership-posterior ICC artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct MembershipPosteriorIccExecution {
    /// Digest-bound completed composition artifact.
    pub artifact: MembershipPosteriorIccArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

struct EligibleMembership {
    network: MembershipNetwork,
    outcomes: Vec<NestedOutcome>,
    weights: Vec<f64>,
    excluded_after_cutoff_count: u64,
}

fn admit_observations_at_cutoff(
    observations: &[MembershipPosteriorObservation],
    knowledge_cutoff: KnowledgeCutoff,
    classification_instant: EventTime,
) -> Result<EligibleMembership, AnalysisEngineError> {
    if observations.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    let mut network = MembershipNetwork::new();
    let mut outcomes = Vec::new();
    let mut weights = Vec::new();
    let mut excluded_after_cutoff_count = 0_u64;
    for observation in observations {
        if observation.available_time.instant() > knowledge_cutoff.instant() {
            excluded_after_cutoff_count += 1;
            continue;
        }
        if !observation.assignment.is_active_at(classification_instant) {
            continue;
        }
        let point_estimate = posterior_draw_point_estimate_mean(&observation.posterior_draws)?;
        network.insert(observation.assignment)?;
        outcomes.push(NestedOutcome::new(observation.member_id(), point_estimate)?);
        weights.push(observation.weight().value());
    }
    if outcomes.is_empty() {
        return Err(AnalysisEngineError::Membership(
            MembershipError::InsufficientClusterStructure,
        ));
    }
    Ok(EligibleMembership {
        network,
        outcomes,
        weights,
        excluded_after_cutoff_count,
    })
}

fn design_wire_name(
    design: MembershipDesign,
) -> Result<(&'static str, &'static str), AnalysisEngineError> {
    match design {
        MembershipDesign::Nested => Ok((DESIGN_NESTED, NESTED_INFERENCE_STATUS)),
        MembershipDesign::MultipleMembership => Ok((
            DESIGN_MULTIPLE_MEMBERSHIP,
            MULTIPLE_MEMBERSHIP_INFERENCE_STATUS,
        )),
        MembershipDesign::CrossClassified => {
            Ok((DESIGN_CROSS_CLASSIFIED, CROSS_CLASSIFIED_INFERENCE_STATUS))
        }
        _ => Err(AnalysisEngineError::InvalidEvidence),
    }
}

fn nested_icc_for_design(
    design: MembershipDesign,
    network: &MembershipNetwork,
    instant: EventTime,
    outcomes: &[NestedOutcome],
) -> Result<Option<f64>, AnalysisEngineError> {
    match design {
        MembershipDesign::Nested => Ok(Some(nested_intraclass_correlation(
            network, instant, outcomes,
        )?)),
        MembershipDesign::MultipleMembership | MembershipDesign::CrossClassified => Ok(None),
        _ => Err(AnalysisEngineError::InvalidEvidence),
    }
}

/// Execute cutoff-safe nested ICC of posterior means under classified membership.
///
/// Point estimates use [`posterior_draw_point_estimate_mean`], not Rubin pooling.
/// Multiple membership and cross-classification are classified without collapse;
/// nested ICC is refused for those designs while Kish ESS of membership weights
/// is still emitted. This is not ESEM, not DSEM, and not an MMMC sampler.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, psychometric or
/// membership recovery failure, or invalid artifact error.
pub fn execute_membership_posterior_icc_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    classification_instant: EventTime,
    observations: &[MembershipPosteriorObservation],
    completed_at: impl Into<String>,
) -> Result<MembershipPosteriorIccExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != MEMBERSHIP_POSTERIOR_ICC_MODEL_CONTRACT_VERSION
        || request.output_profile != MEMBERSHIP_POSTERIOR_ICC_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let eligible =
        admit_observations_at_cutoff(observations, knowledge_cutoff, classification_instant)?;
    let design = classify_membership_design(&eligible.network, classification_instant)?;
    let (membership_design, inference_status) = design_wire_name(design)?;
    let nested_icc = nested_icc_for_design(
        design,
        &eligible.network,
        classification_instant,
        &eligible.outcomes,
    )?;
    let kish_ess = kish_effective_sample_size(&eligible.weights)?;
    let eligible_assignment_count = u64::try_from(eligible.weights.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let eligible_member_count = u64::try_from(eligible.outcomes.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;

    let artifact = MembershipPosteriorIccArtifact {
        schema_version: MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        classification_instant: classification_instant.to_rfc3339(),
        membership_design: membership_design.into(),
        eligible_member_count,
        eligible_assignment_count,
        excluded_after_cutoff_count: eligible.excluded_after_cutoff_count,
        nested_icc,
        kish_ess,
        inference_status: inference_status.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "membership_posterior_icc",
        eligible_assignment_count,
        2,
        inference_status,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("membership_posterior_icc_artifact_{}", &digest[..16]),
        digest,
        MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(MembershipPosteriorIccExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CROSS_CLASSIFIED_INFERENCE_STATUS, DESIGN_CROSS_CLASSIFIED, DESIGN_NESTED,
        MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_BYTE_LIMIT,
        MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_SCHEMA_VERSION, MULTIPLE_MEMBERSHIP_INFERENCE_STATUS,
        MembershipPosteriorIccArtifact, NESTED_INFERENCE_STATUS,
    };
    use crate::AnalysisEngineError;

    fn nested_artifact() -> MembershipPosteriorIccArtifact {
        MembershipPosteriorIccArtifact {
            schema_version: MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            classification_instant: "2026-06-01T00:00:00Z".into(),
            membership_design: DESIGN_NESTED.into(),
            eligible_member_count: 8,
            eligible_assignment_count: 8,
            excluded_after_cutoff_count: 0,
            nested_icc: Some(0.25),
            kish_ess: 8.0,
            inference_status: NESTED_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &MembershipPosteriorIccArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidMembershipPosteriorIccArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = nested_artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            MembershipPosteriorIccArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            MembershipPosteriorIccArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidMembershipPosteriorIccArtifact)
        );
        assert_eq!(
            MembershipPosteriorIccArtifact::from_json(
                &"x".repeat(MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_BYTE_LIMIT + 1)
            ),
            Err(AnalysisEngineError::LimitExceeded)
        );
    }

    #[test]
    fn artifact_metadata_tampering_fails_closed() {
        let artifact = nested_artifact();
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
                value.classification_instant = "invalid".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.membership_design = "collapsed".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.eligible_member_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.eligible_assignment_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.nested_icc = None;
                value
            },
            {
                let mut value = artifact.clone();
                value.nested_icc = Some(1.5);
                value
            },
            {
                let mut value = artifact.clone();
                value.nested_icc = Some(f64::NAN);
                value
            },
            {
                let mut value = artifact.clone();
                value.kish_ess = 0.0;
                value
            },
            {
                let mut value = artifact.clone();
                value.inference_status.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.membership_design = DESIGN_CROSS_CLASSIFIED.into();
                value.nested_icc = Some(0.25);
                value.inference_status = CROSS_CLASSIFIED_INFERENCE_STATUS.into();
                value
            },
            {
                let mut value = artifact.clone();
                value.membership_design = "multiple_membership".into();
                value.nested_icc = None;
                value.inference_status = MULTIPLE_MEMBERSHIP_INFERENCE_STATUS.into();
                value.kish_ess = f64::INFINITY;
                value
            },
        ];
        for invalid in invalid_artifacts {
            assert_invalid(&invalid);
        }
    }
}
