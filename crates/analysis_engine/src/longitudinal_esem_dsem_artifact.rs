//! Digest-bound longitudinal ESEM/DSEM engine composition as an analysis-run profile.

use longitudinal_core::{ComponentLevel, refuse_between_as_within_change};
use membership_core::{MembershipDesign, MembershipError};
use psychometric_core::{
    CausalHeuristic, ConstructClass, LagClock, LatentMeanComparisonEvidence, MeanInvarianceStatus,
    PsychometricError, TwoGroupMeasurement, claim_causal_effect, compare_latent_means,
    interpret_as_reflective, posterior_draw_point_estimate_mean,
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

/// Versioned schema for a completed longitudinal ESEM/DSEM composition artifact.
pub const LONGITUDINAL_ESEM_DSEM_ARTIFACT_SCHEMA_VERSION: &str =
    "tepp.longitudinal_esem_dsem_composition.v1";
/// Model contract required by the composition execution path.
pub const LONGITUDINAL_ESEM_DSEM_MODEL_CONTRACT_VERSION: &str =
    "longitudinal_esem_dsem_composition_v1";
/// Analysis-run output profile required for the composition artifact.
pub const LONGITUDINAL_ESEM_DSEM_OUTPUT_PROFILE: &str = "longitudinal_esem_dsem_composition_v1";
/// Maximum canonical artifact JSON size.
pub const LONGITUDINAL_ESEM_DSEM_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const LONGITUDINAL_ESEM_DSEM_INFERENCE_STATUS: &str = "composed_engine_not_estimator";

/// Run-level scientific design offered to one composition execution.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent fail-closed composition gates"
)]
pub struct LongitudinalEsemDsemDesign {
    construct_class: ConstructClass,
    membership_design: MembershipDesign,
    collapse_hierarchy: bool,
    component_level: ComponentLevel,
    lag_clock: LagClock,
    invariance_status: MeanInvarianceStatus,
    comparison_scope: String,
    model_version: String,
    treat_ols_as_dsem: bool,
    promote_causal: bool,
}

impl LongitudinalEsemDsemDesign {
    /// Bind the fail-closed composition gates for one analysis run.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when comparison-scope
    /// or model-version labels are empty.
    #[allow(
        clippy::too_many_arguments,
        clippy::fn_params_excessive_bools,
        reason = "audited composition gate sequence"
    )]
    pub fn new(
        construct_class: ConstructClass,
        membership_design: MembershipDesign,
        collapse_hierarchy: bool,
        component_level: ComponentLevel,
        lag_clock: LagClock,
        invariance_status: MeanInvarianceStatus,
        comparison_scope: impl Into<String>,
        model_version: impl Into<String>,
        treat_ols_as_dsem: bool,
        promote_causal: bool,
    ) -> Result<Self, AnalysisEngineError> {
        let comparison_scope = comparison_scope.into();
        let model_version = model_version.into();
        if comparison_scope.is_empty() || model_version.is_empty() {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            construct_class,
            membership_design,
            collapse_hierarchy,
            component_level,
            lag_clock,
            invariance_status,
            comparison_scope,
            model_version,
            treat_ols_as_dsem,
            promote_causal,
        })
    }

    /// Return the classified construct class.
    #[must_use]
    pub const fn construct_class(&self) -> ConstructClass {
        self.construct_class
    }

    /// Return the membership design that must not be silently collapsed.
    #[must_use]
    pub const fn membership_design(&self) -> MembershipDesign {
        self.membership_design
    }

    /// Return whether the caller asked to collapse non-nested membership.
    #[must_use]
    pub const fn collapse_hierarchy(&self) -> bool {
        self.collapse_hierarchy
    }

    /// Return the within/between component level.
    #[must_use]
    pub const fn component_level(&self) -> ComponentLevel {
        self.component_level
    }

    /// Return the lag clock offered for structural dynamics.
    #[must_use]
    pub const fn lag_clock(&self) -> LagClock {
        self.lag_clock
    }

    /// Return the invariance status offered for mean comparison.
    #[must_use]
    pub const fn invariance_status(&self) -> MeanInvarianceStatus {
        self.invariance_status
    }

    /// Return the comparison-scope label carried in invariance evidence.
    #[must_use]
    pub fn comparison_scope(&self) -> &str {
        &self.comparison_scope
    }

    /// Return the model-version label carried in invariance evidence.
    #[must_use]
    pub fn model_version(&self) -> &str {
        &self.model_version
    }

    /// Return whether OLS recovery was offered as a DSEM estimator.
    #[must_use]
    pub const fn treat_ols_as_dsem(&self) -> bool {
        self.treat_ols_as_dsem
    }

    /// Return whether temporal precedence was offered as a causal claim.
    #[must_use]
    pub const fn promote_causal(&self) -> bool {
        self.promote_causal
    }
}

/// One already-mapped posterior-draw observation offered to a cutoff-safe run.
#[derive(Clone, Debug, PartialEq)]
pub struct LongitudinalEsemDsemObservation {
    available_time: AvailableTime,
    posterior_draws: Vec<f64>,
}

impl LongitudinalEsemDsemObservation {
    /// Bind posterior draws to an availability clock.
    ///
    /// Point topic estimates are not admitted. Each observation must carry at
    /// least two finite posterior draws.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::Psychometric`] when fewer than two draws
    /// are supplied or a draw is non-finite.
    pub fn new(
        available_time: AvailableTime,
        posterior_draws: Vec<f64>,
    ) -> Result<Self, AnalysisEngineError> {
        if posterior_draws.len() < 2 {
            return Err(AnalysisEngineError::Psychometric(
                PsychometricError::InsufficientDraws,
            ));
        }
        for value in &posterior_draws {
            if !value.is_finite() {
                return Err(AnalysisEngineError::Psychometric(
                    PsychometricError::InvalidNumericInput,
                ));
            }
        }
        Ok(Self {
            available_time,
            posterior_draws,
        })
    }

    /// Return the availability clock used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }

    /// Return the posterior draws in source order.
    #[must_use]
    pub fn posterior_draws(&self) -> &[f64] {
        &self.posterior_draws
    }
}

/// Completed, bounded ESEM/DSEM composition consumed by analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LongitudinalEsemDsemArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the composition.
    pub knowledge_cutoff: String,
    /// Eligible observations after cutoff.
    pub observation_count: u64,
    /// Posterior draws among eligible observations.
    pub draw_count: u64,
    /// Observations excluded because availability was after the cutoff.
    pub excluded_after_cutoff_count: u64,
    /// Arithmetic mean of eligible posterior draws. Not an ESEM/DSEM fit.
    pub posterior_draw_mean: f64,
    /// Classified construct class admitted as reflective.
    pub construct_class: String,
    /// Membership design preserved without collapse.
    pub membership_design: String,
    /// Within-unit component level.
    pub component_level: String,
    /// Event-time lag clock.
    pub lag_clock: String,
    /// Strong or strict invariance status that licenses means.
    pub invariance_status: String,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl LongitudinalEsemDsemArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidLongitudinalEsemDsemArtifact`]
    /// when the schema, identifiers, counts, mean, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > LONGITUDINAL_ESEM_DSEM_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidLongitudinalEsemDsemArtifact)?;
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
        if payload.len() > LONGITUDINAL_ESEM_DSEM_ARTIFACT_BYTE_LIMIT {
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
        if self.schema_version != LONGITUDINAL_ESEM_DSEM_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.observation_count == 0
            || self.draw_count < 2
            || !self.posterior_draw_mean.is_finite()
            || self.construct_class != ConstructClass::Reflective.as_str()
            || membership_design_wire_name_is_unknown(&self.membership_design)
            || self.component_level != ComponentLevel::Within.wire_name()
            || self.lag_clock != LagClock::EventTime.as_str()
            || !(self.invariance_status == MeanInvarianceStatus::Strong.as_str()
                || self.invariance_status == MeanInvarianceStatus::Strict.as_str())
            || self.inference_status != LONGITUDINAL_ESEM_DSEM_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidLongitudinalEsemDsemArtifact);
        }
        Ok(())
    }
}

fn membership_design_wire_name_is_unknown(name: &str) -> bool {
    !matches!(name, "nested" | "cross_classified" | "multiple_membership")
}

fn membership_design_wire_name(
    design: MembershipDesign,
) -> Result<&'static str, AnalysisEngineError> {
    match design {
        MembershipDesign::Nested => Ok("nested"),
        MembershipDesign::CrossClassified => Ok("cross_classified"),
        MembershipDesign::MultipleMembership => Ok("multiple_membership"),
        _ => Err(AnalysisEngineError::InvalidEvidence),
    }
}

/// One completed composition artifact and its request-bound terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct LongitudinalEsemDsemExecution {
    /// Digest-bound completed composition artifact.
    pub artifact: LongitudinalEsemDsemArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

struct EligibleDraws {
    draws: Vec<f64>,
    observation_count: u64,
    excluded_after_cutoff_count: u64,
}

fn admit_draws_at_cutoff(
    observations: &[LongitudinalEsemDsemObservation],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<EligibleDraws, AnalysisEngineError> {
    if observations.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }
    let mut draws = Vec::new();
    let mut observation_count = 0_u64;
    let mut excluded_after_cutoff_count = 0_u64;
    for observation in observations {
        if observation.available_time.instant() <= knowledge_cutoff.instant() {
            draws.extend_from_slice(&observation.posterior_draws);
            observation_count = observation_count
                .checked_add(1)
                .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
        } else {
            excluded_after_cutoff_count = excluded_after_cutoff_count
                .checked_add(1)
                .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
        }
    }
    if draws.is_empty() {
        return Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput,
        ));
    }
    Ok(EligibleDraws {
        draws,
        observation_count,
        excluded_after_cutoff_count,
    })
}

fn apply_composition_gates(design: &LongitudinalEsemDsemDesign) -> Result<(), AnalysisEngineError> {
    interpret_as_reflective(design.construct_class, true)?;
    if design.collapse_hierarchy && !design.membership_design.allows_nested_icc() {
        return Err(AnalysisEngineError::Membership(
            MembershipError::NestedIccInapplicable,
        ));
    }
    refuse_between_as_within_change(design.component_level)?;
    if !design.lag_clock.admits_structural_lag() {
        return Err(AnalysisEngineError::Psychometric(
            PsychometricError::EventTimeRequired,
        ));
    }
    let evidence = LatentMeanComparisonEvidence::from_two_group_measurement(
        &TwoGroupMeasurement {
            reference_intercept: 0.0,
            reference_loading: 1.0,
            comparison_intercept: 0.0,
            comparison_loading: 1.0,
            reference_residual_variance: 1.0,
            comparison_residual_variance: 1.0,
            status: design.invariance_status,
        },
        design.comparison_scope(),
        design.model_version(),
    )?;
    compare_latent_means(&evidence)?;
    if design.treat_ols_as_dsem {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if design.promote_causal {
        claim_causal_effect(CausalHeuristic::TemporalPrecedence)?;
    }
    let _ = claim_causal_effect(CausalHeuristic::TemporalPrecedence);
    Ok(())
}

/// Execute cutoff-safe longitudinal ESEM/DSEM engine composition.
///
/// The caller supplies already-mapped posterior draws and an explicit
/// measurement design. This executor does not invent an ESEM/DSEM sampler,
/// persist rows, treat OLS as DSEM, collapse non-nested membership, or promote
/// temporal precedence to a causal estimand.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, a psychometric,
/// membership, or longitudinal gate failure, or an invalid artifact error.
pub fn execute_longitudinal_esem_dsem_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    design: &LongitudinalEsemDsemDesign,
    observations: &[LongitudinalEsemDsemObservation],
    completed_at: impl Into<String>,
) -> Result<LongitudinalEsemDsemExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != LONGITUDINAL_ESEM_DSEM_MODEL_CONTRACT_VERSION
        || request.output_profile != LONGITUDINAL_ESEM_DSEM_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    apply_composition_gates(design)?;
    let eligible = admit_draws_at_cutoff(observations, knowledge_cutoff)?;
    let posterior_draw_mean = posterior_draw_point_estimate_mean(&eligible.draws)?;
    let draw_count =
        u64::try_from(eligible.draws.len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let artifact = LongitudinalEsemDsemArtifact {
        schema_version: LONGITUDINAL_ESEM_DSEM_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        observation_count: eligible.observation_count,
        draw_count,
        excluded_after_cutoff_count: eligible.excluded_after_cutoff_count,
        posterior_draw_mean,
        construct_class: design.construct_class.as_str().into(),
        membership_design: membership_design_wire_name(design.membership_design)?.into(),
        component_level: design.component_level.wire_name().into(),
        lag_clock: design.lag_clock.as_str().into(),
        invariance_status: design.invariance_status.as_str().into(),
        inference_status: LONGITUDINAL_ESEM_DSEM_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "longitudinal_esem_dsem_composition",
        eligible.observation_count,
        1,
        LONGITUDINAL_ESEM_DSEM_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("longitudinal_esem_dsem_artifact_{}", &digest[..16]),
        digest,
        LONGITUDINAL_ESEM_DSEM_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(LongitudinalEsemDsemExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        LONGITUDINAL_ESEM_DSEM_ARTIFACT_BYTE_LIMIT, LONGITUDINAL_ESEM_DSEM_ARTIFACT_SCHEMA_VERSION,
        LONGITUDINAL_ESEM_DSEM_INFERENCE_STATUS, LongitudinalEsemDsemArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> LongitudinalEsemDsemArtifact {
        LongitudinalEsemDsemArtifact {
            schema_version: LONGITUDINAL_ESEM_DSEM_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            observation_count: 2,
            draw_count: 4,
            excluded_after_cutoff_count: 0,
            posterior_draw_mean: 0.5,
            construct_class: "reflective".into(),
            membership_design: "nested".into(),
            component_level: "within".into(),
            lag_clock: "event_time".into(),
            invariance_status: "strong".into(),
            inference_status: LONGITUDINAL_ESEM_DSEM_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &LongitudinalEsemDsemArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidLongitudinalEsemDsemArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            LongitudinalEsemDsemArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            LongitudinalEsemDsemArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidLongitudinalEsemDsemArtifact)
        );
        assert_eq!(
            LongitudinalEsemDsemArtifact::from_json(
                &"x".repeat(LONGITUDINAL_ESEM_DSEM_ARTIFACT_BYTE_LIMIT + 1)
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
                value.observation_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.draw_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.posterior_draw_mean = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.construct_class = "formative".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.membership_design = "collapsed".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.component_level = "between".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.lag_clock = "system_time".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.invariance_status = "metric".into();
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
