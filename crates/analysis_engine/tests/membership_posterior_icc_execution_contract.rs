//! End-to-end contract for cutoff-safe membership-posterior ICC composition.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_SCHEMA_VERSION,
    MEMBERSHIP_POSTERIOR_ICC_MODEL_CONTRACT_VERSION, MEMBERSHIP_POSTERIOR_ICC_OUTPUT_PROFILE,
    MembershipPosteriorObservation, execute_membership_posterior_icc_run,
};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipError, MembershipRole, MembershipWeight,
};
use psychometric_core::PsychometricError;
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn event(stamp: &str) -> EventTime {
    EventTime::parse_rfc3339(stamp).expect("event")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn classification() -> EventTime {
    event("2026-06-01T00:00:00Z")
}

fn start() -> EventTime {
    event("2026-01-01T00:00:00Z")
}

fn end() -> EventTime {
    event("2026-12-31T00:00:00Z")
}

fn observation(
    member: MemberId,
    group: GroupId,
    role: MembershipRole,
    weight: f64,
    draws: Vec<f64>,
    available_stamp: &str,
) -> MembershipPosteriorObservation {
    let assignment = MembershipAssignment::new(
        member,
        group,
        role,
        MembershipWeight::new(weight).expect("weight"),
        start(),
        end(),
    )
    .expect("assignment");
    MembershipPosteriorObservation::new(assignment, draws, available(available_stamp))
        .expect("observation")
}

fn nested_observations() -> Vec<MembershipPosteriorObservation> {
    // Four groups of two: means 2,3,4,5 with within deviation ±1 → ICC = 1/4.
    // Posterior draws average to those known outcomes; this is not Rubin pooling.
    let groups = [
        GroupId::new(),
        GroupId::new(),
        GroupId::new(),
        GroupId::new(),
    ];
    let rows = [
        (groups[0], [1.0, 3.0]),
        (groups[1], [2.0, 4.0]),
        (groups[2], [3.0, 5.0]),
        (groups[3], [4.0, 6.0]),
    ];
    let mut observations = Vec::new();
    for (group, values) in rows {
        for value in values {
            observations.push(observation(
                MemberId::new(),
                group,
                MembershipRole::Author,
                1.0,
                vec![value - 1.0, value + 1.0],
                "2026-07-01T00:00:00Z",
            ));
        }
    }
    observations
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "membership-posterior-icc-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-membership-posterior-icc".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: MEMBERSHIP_POSTERIOR_ICC_MODEL_CONTRACT_VERSION.into(),
        output_profile: MEMBERSHIP_POSTERIOR_ICC_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-membership-posterior-icc",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
    observations: &[MembershipPosteriorObservation],
) -> Result<analysis_engine::MembershipPosteriorIccExecution, AnalysisEngineError> {
    execute_membership_posterior_icc_run(
        request,
        &accepted(request),
        "snapshot-membership-posterior-icc",
        cutoff(),
        classification(),
        observations,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn nested_posterior_means_recover_known_anova_icc_and_kish_ess() {
    let request = request();
    let observations = nested_observations();
    let execution = execute(&request, &observations).expect("execution");

    assert_eq!(
        execution.artifact.schema_version,
        MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.membership_design, "nested");
    assert_eq!(execution.artifact.eligible_member_count, 8);
    assert_eq!(execution.artifact.eligible_assignment_count, 8);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 0);
    let nested_icc = execution.artifact.nested_icc.expect("nested icc");
    assert!((nested_icc - 0.25).abs() < 1e-12);
    assert!((execution.artifact.kish_ess - 8.0).abs() < 1e-12);
    assert_eq!(
        execution.artifact.inference_status,
        "nested_icc_of_posterior_means_not_mmmc"
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
        Some(MEMBERSHIP_POSTERIOR_ICC_ARTIFACT_SCHEMA_VERSION)
    );
    assert_eq!(observations[0].role(), MembershipRole::Author);
    assert!((observations[0].weight().value() - 1.0).abs() < f64::EPSILON);
    assert_eq!(
        observations[0].available_time(),
        available("2026-07-01T00:00:00Z")
    );
    assert_eq!(observations[0].posterior_draws().len(), 2);
}

#[test]
fn execution_excludes_observations_unavailable_at_the_request_cutoff() {
    let request = request();
    let mut observations = nested_observations();
    observations.push(observation(
        MemberId::new(),
        GroupId::new(),
        MembershipRole::Author,
        1.0,
        vec![9.0, 11.0],
        "2026-08-15T00:00:00Z",
    ));
    let execution = execute(&request, &observations).expect("execution");
    assert_eq!(execution.artifact.eligible_member_count, 8);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 1);
    let nested_icc = execution.artifact.nested_icc.expect("nested icc");
    assert!((nested_icc - 0.25).abs() < 1e-12);
}

#[test]
fn multiple_membership_preserves_design_refuses_nested_icc_and_emits_kish_ess() {
    let request = request();
    let member = MemberId::new();
    let observations = vec![
        observation(
            member,
            GroupId::new(),
            MembershipRole::Department,
            0.6,
            vec![1.0, 3.0],
            "2026-07-01T00:00:00Z",
        ),
        observation(
            member,
            GroupId::new(),
            MembershipRole::Department,
            0.4,
            vec![0.0, 2.0],
            "2026-07-01T00:00:00Z",
        ),
    ];
    let execution = execute(&request, &observations).expect("execution");
    assert_eq!(execution.artifact.membership_design, "multiple_membership");
    assert_eq!(execution.artifact.nested_icc, None);
    assert_eq!(execution.artifact.eligible_assignment_count, 2);
    let expected_ess = 1.0 / 0.52;
    assert!((execution.artifact.kish_ess - expected_ess).abs() < 1e-12);
    assert_eq!(
        execution.artifact.inference_status,
        "multiple_membership_preserved_nested_icc_refused"
    );
}

#[test]
fn cross_classified_preserves_design_and_refuses_nested_icc() {
    let request = request();
    let member = MemberId::new();
    let observations = vec![
        observation(
            member,
            GroupId::new(),
            MembershipRole::Author,
            1.0,
            vec![1.0, 1.0],
            "2026-07-01T00:00:00Z",
        ),
        observation(
            member,
            GroupId::new(),
            MembershipRole::Project,
            0.5,
            vec![2.0, 2.0],
            "2026-07-01T00:00:00Z",
        ),
    ];
    let execution = execute(&request, &observations).expect("execution");
    assert_eq!(execution.artifact.membership_design, "cross_classified");
    assert_eq!(execution.artifact.nested_icc, None);
    assert_eq!(
        execution.artifact.inference_status,
        "cross_classified_preserved_nested_icc_refused"
    );
    assert!(execution.artifact.kish_ess > 1.0);
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let observations = nested_observations();
    assert_eq!(
        execute_membership_posterior_icc_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            classification(),
            &observations,
            "2026-08-02T00:00:00Z",
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
            execute(&invalid_request, &observations),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}

#[test]
fn execution_refuses_non_finite_draws_empty_eligibility_and_oversize() {
    assert_eq!(
        MembershipPosteriorObservation::new(
            MembershipAssignment::new(
                MemberId::new(),
                GroupId::new(),
                MembershipRole::Author,
                MembershipWeight::full().expect("full"),
                start(),
                end(),
            )
            .expect("assignment"),
            vec![f64::NAN],
            available("2026-07-01T00:00:00Z"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        MembershipPosteriorObservation::new(
            MembershipAssignment::new(
                MemberId::new(),
                GroupId::new(),
                MembershipRole::Author,
                MembershipWeight::full().expect("full"),
                start(),
                end(),
            )
            .expect("assignment"),
            vec![],
            available("2026-07-01T00:00:00Z"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let request = request();
    let late_only = vec![observation(
        MemberId::new(),
        GroupId::new(),
        MembershipRole::Author,
        1.0,
        vec![1.0, 3.0],
        "2026-08-15T00:00:00Z",
    )];
    assert_eq!(
        execute(&request, &late_only),
        Err(AnalysisEngineError::Membership(
            MembershipError::InsufficientClusterStructure
        ))
    );
    let pad_member = MemberId::new();
    let pad_group = GroupId::new();
    let oversized = vec![
        observation(
            pad_member,
            pad_group,
            MembershipRole::Author,
            1.0,
            vec![1.0, 1.0],
            "2026-07-01T00:00:00Z",
        );
        MAX_EVIDENCE_UNITS + 1
    ];
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}

#[test]
fn posterior_mean_is_not_rubin_pooling_and_psychometric_errors_surface() {
    let mean = psychometric_core::posterior_draw_point_estimate_mean(&[1.0, 3.0]).expect("mean");
    assert!((mean - 2.0).abs() < 1e-15);
    assert_eq!(
        psychometric_core::posterior_draw_point_estimate_mean(&[]),
        Err(PsychometricError::InvalidNumericInput)
    );
}
