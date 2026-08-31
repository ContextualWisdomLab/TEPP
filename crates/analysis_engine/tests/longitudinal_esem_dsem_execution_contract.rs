//! End-to-end contract for cutoff-safe longitudinal ESEM/DSEM composition.

use analysis_engine::{
    AnalysisEngineError, LONGITUDINAL_ESEM_DSEM_ARTIFACT_SCHEMA_VERSION,
    LONGITUDINAL_ESEM_DSEM_MODEL_CONTRACT_VERSION, LONGITUDINAL_ESEM_DSEM_OUTPUT_PROFILE,
    LongitudinalEsemDsemDesign, LongitudinalEsemDsemObservation, MAX_EVIDENCE_UNITS,
    execute_longitudinal_esem_dsem_run,
};
use longitudinal_core::{ComponentLevel, LongitudinalError};
use membership_core::{MembershipDesign, MembershipError};
use psychometric_core::{ConstructClass, LagClock, MeanInvarianceStatus, PsychometricError};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn observation(stamp: &str, draws: Vec<f64>) -> LongitudinalEsemDsemObservation {
    LongitudinalEsemDsemObservation::new(available(stamp), draws).expect("observation")
}

fn eligible_observations() -> Vec<LongitudinalEsemDsemObservation> {
    vec![
        observation("2026-07-01T00:00:00Z", vec![0.0, 1.0]),
        observation("2026-07-15T00:00:00Z", vec![0.0, 1.0]),
    ]
}

#[allow(
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools,
    reason = "test helper mirrors the audited design constructor"
)]
fn design(
    construct_class: ConstructClass,
    membership_design: MembershipDesign,
    collapse_hierarchy: bool,
    component_level: ComponentLevel,
    lag_clock: LagClock,
    invariance_status: MeanInvarianceStatus,
    treat_ols_as_dsem: bool,
    promote_causal: bool,
) -> LongitudinalEsemDsemDesign {
    LongitudinalEsemDsemDesign::new(
        construct_class,
        membership_design,
        collapse_hierarchy,
        component_level,
        lag_clock,
        invariance_status,
        "group-contrast",
        "composition-v1",
        treat_ols_as_dsem,
        promote_causal,
    )
    .expect("design")
}

fn valid_design() -> LongitudinalEsemDsemDesign {
    design(
        ConstructClass::Reflective,
        MembershipDesign::Nested,
        false,
        ComponentLevel::Within,
        LagClock::EventTime,
        MeanInvarianceStatus::Strong,
        false,
        false,
    )
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "longitudinal-esem-dsem-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-longitudinal-esem-dsem".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: LONGITUDINAL_ESEM_DSEM_MODEL_CONTRACT_VERSION.into(),
        output_profile: LONGITUDINAL_ESEM_DSEM_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-longitudinal-esem-dsem",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

#[allow(
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools,
    reason = "test helper mirrors the audited design constructor"
)]
fn execute(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    design: &LongitudinalEsemDsemDesign,
    observations: &[LongitudinalEsemDsemObservation],
) -> Result<analysis_engine::LongitudinalEsemDsemExecution, AnalysisEngineError> {
    execute_longitudinal_esem_dsem_run(
        request,
        accepted,
        snapshot_id,
        knowledge_cutoff,
        design,
        observations,
        "2026-08-02T00:00:00Z",
    )
}

fn refuse(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    observations: &[LongitudinalEsemDsemObservation],
    design: &LongitudinalEsemDsemDesign,
    expected: AnalysisEngineError,
) {
    assert_eq!(
        execute(
            request,
            accepted,
            "snapshot-longitudinal-esem-dsem",
            cutoff(),
            design,
            observations,
        ),
        Err(expected)
    );
}

#[test]
fn composed_engine_emits_digest_bound_posterior_mean_not_an_estimator() {
    let request = request();
    let accepted = accepted(&request);
    let design = valid_design();
    let observations = eligible_observations();
    let execution = execute(
        &request,
        &accepted,
        "snapshot-longitudinal-esem-dsem",
        cutoff(),
        &design,
        &observations,
    )
    .expect("execution");

    assert_eq!(
        execution.artifact.schema_version,
        LONGITUDINAL_ESEM_DSEM_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.observation_count, 2);
    assert_eq!(execution.artifact.draw_count, 4);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 0);
    assert!((execution.artifact.posterior_draw_mean - 0.5).abs() < 1e-12);
    assert_eq!(execution.artifact.construct_class, "reflective");
    assert_eq!(execution.artifact.membership_design, "nested");
    assert_eq!(execution.artifact.component_level, "within");
    assert_eq!(execution.artifact.lag_clock, "event_time");
    assert_eq!(execution.artifact.invariance_status, "strong");
    assert_eq!(
        execution.artifact.inference_status,
        "composed_engine_not_estimator"
    );
    assert_eq!(
        execution.terminal_result.run_state,
        AnalysisRunTerminalState::Succeeded
    );
    assert_eq!(
        execution.terminal_result.result_sha256.as_deref(),
        Some(execution.artifact.sha256().expect("digest").as_str())
    );
    assert_eq!(
        execution.terminal_result.result_schema_version.as_deref(),
        Some(LONGITUDINAL_ESEM_DSEM_ARTIFACT_SCHEMA_VERSION)
    );
    assert_eq!(design.construct_class(), ConstructClass::Reflective);
    assert_eq!(design.membership_design(), MembershipDesign::Nested);
    assert!(!design.collapse_hierarchy());
    assert_eq!(design.component_level(), ComponentLevel::Within);
    assert_eq!(design.lag_clock(), LagClock::EventTime);
    assert_eq!(design.invariance_status(), MeanInvarianceStatus::Strong);
    assert_eq!(design.comparison_scope(), "group-contrast");
    assert_eq!(design.model_version(), "composition-v1");
    assert!(!design.treat_ols_as_dsem());
    assert!(!design.promote_causal());
    assert_eq!(
        observations[0].available_time(),
        available("2026-07-01T00:00:00Z")
    );
    assert_eq!(observations[0].posterior_draws(), &[0.0, 1.0]);
}

#[test]
fn execution_excludes_draws_unavailable_at_the_request_cutoff() {
    let request = request();
    let accepted = accepted(&request);
    let mut observations = eligible_observations();
    observations.push(observation("2026-08-15T00:00:00Z", vec![9.0, 9.0]));
    let execution = execute(
        &request,
        &accepted,
        "snapshot-longitudinal-esem-dsem",
        cutoff(),
        &valid_design(),
        &observations,
    )
    .expect("execution");
    assert_eq!(execution.artifact.observation_count, 2);
    assert_eq!(execution.artifact.draw_count, 4);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 1);
    assert!((execution.artifact.posterior_draw_mean - 0.5).abs() < 1e-12);
}

#[test]
fn cross_classified_membership_is_preserved_without_collapse() {
    let request = request();
    let accepted = accepted(&request);
    let design = design(
        ConstructClass::Reflective,
        MembershipDesign::CrossClassified,
        false,
        ComponentLevel::Within,
        LagClock::EventTime,
        MeanInvarianceStatus::Strict,
        false,
        false,
    );
    let execution = execute(
        &request,
        &accepted,
        "snapshot-longitudinal-esem-dsem",
        cutoff(),
        &design,
        &eligible_observations(),
    )
    .expect("execution");
    assert_eq!(execution.artifact.membership_design, "cross_classified");
    assert_eq!(execution.artifact.invariance_status, "strict");
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let accepted = accepted(&request);
    let observations = eligible_observations();
    let design = valid_design();
    assert_eq!(
        execute(
            &request,
            &accepted,
            "other-snapshot",
            cutoff(),
            &design,
            &observations,
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    for invalid_request in [
        {
            let mut value = request.clone();
            value.knowledge_cutoff = "2026-08-02T00:00:00Z".into();
            value
        },
        {
            let mut value = request.clone();
            value.model_contract_version = "other-model".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "other-profile".into();
            value
        },
    ] {
        assert_eq!(
            execute(
                &invalid_request,
                &accepted,
                "snapshot-longitudinal-esem-dsem",
                cutoff(),
                &design,
                &observations,
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}

#[test]
fn composition_gates_refuse_formative_network_unresolved_metric_and_between() {
    let request = request();
    let accepted = accepted(&request);
    let observations = eligible_observations();
    refuse(
        &request,
        &accepted,
        &observations,
        &design(
            ConstructClass::Formative,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Within,
            LagClock::EventTime,
            MeanInvarianceStatus::Strong,
            false,
            false,
        ),
        AnalysisEngineError::Psychometric(PsychometricError::FormativeReinterpretationForbidden),
    );
    refuse(
        &request,
        &accepted,
        &observations,
        &design(
            ConstructClass::Network,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Within,
            LagClock::EventTime,
            MeanInvarianceStatus::Strong,
            false,
            false,
        ),
        AnalysisEngineError::Psychometric(PsychometricError::FormativeReinterpretationForbidden),
    );
    refuse(
        &request,
        &accepted,
        &observations,
        &design(
            ConstructClass::Unresolved,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Within,
            LagClock::EventTime,
            MeanInvarianceStatus::Strong,
            false,
            false,
        ),
        AnalysisEngineError::Psychometric(PsychometricError::UnresolvedConstruct),
    );
    refuse(
        &request,
        &accepted,
        &observations,
        &design(
            ConstructClass::Reflective,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Within,
            LagClock::EventTime,
            MeanInvarianceStatus::Metric,
            false,
            false,
        ),
        AnalysisEngineError::Psychometric(PsychometricError::StrongInvarianceRequired),
    );
    refuse(
        &request,
        &accepted,
        &observations,
        &design(
            ConstructClass::Reflective,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Between,
            LagClock::EventTime,
            MeanInvarianceStatus::Strong,
            false,
            false,
        ),
        AnalysisEngineError::Longitudinal(LongitudinalError::BetweenIsNotWithinChange),
    );
}

#[test]
fn composition_gates_refuse_clock_collapse_ols_and_causal() {
    let request = request();
    let accepted = accepted(&request);
    let observations = eligible_observations();
    refuse(
        &request,
        &accepted,
        &observations,
        &design(
            ConstructClass::Reflective,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Within,
            LagClock::SystemTime,
            MeanInvarianceStatus::Strong,
            false,
            false,
        ),
        AnalysisEngineError::Psychometric(PsychometricError::EventTimeRequired),
    );
    refuse(
        &request,
        &accepted,
        &observations,
        &design(
            ConstructClass::Reflective,
            MembershipDesign::CrossClassified,
            true,
            ComponentLevel::Within,
            LagClock::EventTime,
            MeanInvarianceStatus::Strong,
            false,
            false,
        ),
        AnalysisEngineError::Membership(MembershipError::NestedIccInapplicable),
    );
    refuse(
        &request,
        &accepted,
        &observations,
        &design(
            ConstructClass::Reflective,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Within,
            LagClock::EventTime,
            MeanInvarianceStatus::Strong,
            true,
            false,
        ),
        AnalysisEngineError::InvalidEvidence,
    );
    refuse(
        &request,
        &accepted,
        &observations,
        &design(
            ConstructClass::Reflective,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Within,
            LagClock::EventTime,
            MeanInvarianceStatus::Strong,
            false,
            true,
        ),
        AnalysisEngineError::Psychometric(PsychometricError::CausalUnderidentified),
    );
}

#[test]
fn execution_refuses_point_estimates_empty_cutoff_receipt_mismatch_and_limits() {
    let request = request();
    let accepted = accepted(&request);
    assert_eq!(
        LongitudinalEsemDsemObservation::new(available("2026-07-01T00:00:00Z"), vec![0.5]),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InsufficientDraws
        ))
    );
    assert_eq!(
        LongitudinalEsemDsemObservation::new(
            available("2026-07-01T00:00:00Z"),
            vec![0.5, f64::NAN]
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput
        ))
    );
    assert_eq!(
        LongitudinalEsemDsemDesign::new(
            ConstructClass::Reflective,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Within,
            LagClock::EventTime,
            MeanInvarianceStatus::Strong,
            "",
            "composition-v1",
            false,
            false,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        LongitudinalEsemDsemDesign::new(
            ConstructClass::Reflective,
            MembershipDesign::Nested,
            false,
            ComponentLevel::Within,
            LagClock::EventTime,
            MeanInvarianceStatus::Strong,
            "group-contrast",
            "",
            false,
            false,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let nested_collapse = design(
        ConstructClass::Reflective,
        MembershipDesign::Nested,
        true,
        ComponentLevel::Within,
        LagClock::EventTime,
        MeanInvarianceStatus::Strong,
        false,
        false,
    );
    assert!(
        execute(
            &request,
            &accepted,
            "snapshot-longitudinal-esem-dsem",
            cutoff(),
            &nested_collapse,
            &eligible_observations(),
        )
        .is_ok()
    );
    let mut early_request = request.clone();
    early_request.knowledge_cutoff = "2026-06-01T00:00:00Z".into();
    let too_early = KnowledgeCutoff::parse_rfc3339("2026-06-01T00:00:00Z").expect("cutoff");
    assert_eq!(
        execute(
            &early_request,
            &accepted,
            "snapshot-longitudinal-esem-dsem",
            too_early,
            &valid_design(),
            &eligible_observations(),
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput
        ))
    );

    let wrong_receipt =
        AnalysisRunAccepted::new("run-longitudinal-esem-dsem", "accepted", "other-key")
            .expect("accepted");
    assert_eq!(
        execute(
            &request,
            &wrong_receipt,
            "snapshot-longitudinal-esem-dsem",
            cutoff(),
            &valid_design(),
            &eligible_observations(),
        )
        .expect_err("receipt"),
        AnalysisEngineError::Api(tepp_api::ApiError::InvalidWirePayload)
    );

    let oversized =
        vec![observation("2026-07-01T00:00:00Z", vec![0.0, 1.0]); MAX_EVIDENCE_UNITS + 1];
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-longitudinal-esem-dsem",
            cutoff(),
            &valid_design(),
            &oversized,
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );

    assert_eq!(
        execute_longitudinal_esem_dsem_run(
            &request,
            &accepted,
            "snapshot-longitudinal-esem-dsem",
            cutoff(),
            &valid_design(),
            &eligible_observations(),
            "invalid",
        ),
        Err(AnalysisEngineError::Api(
            tepp_api::ApiError::InvalidWirePayload
        ))
    );
}
