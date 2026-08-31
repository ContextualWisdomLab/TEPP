//! End-to-end contract for cutoff-safe topic activity/dormancy/reactivation.

use analysis_engine::{
    AnalysisEngineError, TOPIC_ACTIVITY_ARTIFACT_SCHEMA_VERSION,
    TOPIC_ACTIVITY_MODEL_CONTRACT_VERSION, TOPIC_ACTIVITY_OUTPUT_PROFILE, TopicActivityInput,
    TopicActivityTransition, execute_topic_activity_run,
};
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};
use topic_lineage::{TopicIdentity, TopicLineageError};
use uuid::Uuid;

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff")
}

fn identity() -> TopicIdentity {
    TopicIdentity::from_uuid(Uuid::from_u128(11))
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "topic-activity-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-topic-activity".into(),
        knowledge_cutoff: "2026-02-01T00:00:00Z".into(),
        model_contract_version: TOPIC_ACTIVITY_MODEL_CONTRACT_VERSION.into(),
        output_profile: TOPIC_ACTIVITY_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-topic-activity", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn recovered_input() -> TopicActivityInput {
    let identity = identity();
    TopicActivityInput::new(
        identity,
        vec![
            TopicActivityTransition::MakeDormant,
            TopicActivityTransition::Reactivate,
        ],
        identity,
        vec![identity, identity, identity],
        vec![identity, identity, identity],
    )
}

fn execute(
    request: &AnalysisRunRequest,
    input: &TopicActivityInput,
) -> Result<analysis_engine::TopicActivityExecution, AnalysisEngineError> {
    execute_topic_activity_run(
        request,
        &accepted(request),
        "snapshot-topic-activity",
        cutoff(),
        input,
        "2026-02-02T00:00:00Z",
    )
}

#[test]
fn dormancy_then_reactivation_keeps_identity_and_unit_recovery() {
    let request = request();
    let execution = execute(&request, &recovered_input()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        TOPIC_ACTIVITY_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(
        execution.artifact.topic_identity,
        identity().as_uuid().to_string()
    );
    assert_eq!(execution.artifact.activity, "reactivated");
    assert_eq!(execution.artifact.transition_count, 2);
    assert!((execution.artifact.identity_recovery_rate - 1.0).abs() < f64::EPSILON);
    assert!(execution.artifact.reactivation_identity_preserved);
    assert_eq!(
        execution.artifact.inference_status,
        "reactivation_is_not_new_topic_not_birth_split_merge"
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
        Some(TOPIC_ACTIVITY_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn reminted_identity_and_illegal_transitions_fail_closed() {
    let request = request();
    let identity = identity();
    let reminted = TopicActivityInput::new(
        identity,
        vec![
            TopicActivityTransition::MakeDormant,
            TopicActivityTransition::Reactivate,
        ],
        TopicIdentity::from_uuid(Uuid::from_u128(99)),
        vec![identity],
        vec![identity],
    );
    assert_eq!(
        execute(&request, &reminted),
        Err(AnalysisEngineError::TopicActivity(
            TopicLineageError::ReactivationIsNotNewTopic
        ))
    );
    let double_dormant = TopicActivityInput::new(
        identity,
        vec![
            TopicActivityTransition::MakeDormant,
            TopicActivityTransition::MakeDormant,
        ],
        identity,
        vec![identity],
        vec![identity],
    );
    assert_eq!(
        execute(&request, &double_dormant),
        Err(AnalysisEngineError::TopicActivity(
            TopicLineageError::InvalidActivityTransition
        ))
    );
    let empty_truth = TopicActivityInput::new(
        identity,
        vec![TopicActivityTransition::MakeDormant],
        identity,
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(
        execute(&request, &empty_truth),
        Err(AnalysisEngineError::TopicActivity(
            TopicLineageError::InvalidIdentityPayload
        ))
    );
}

#[test]
fn minted_replacements_record_lower_recovery_than_stable_identity() {
    let request = request();
    let identity = identity();
    let other = TopicIdentity::from_uuid(Uuid::from_u128(4));
    let minted = TopicActivityInput::new(
        identity,
        vec![
            TopicActivityTransition::MakeDormant,
            TopicActivityTransition::Reactivate,
        ],
        identity,
        vec![identity, identity, identity],
        vec![identity, identity, other],
    );
    let execution = execute(&request, &minted).expect("execution");
    assert!((execution.artifact.identity_recovery_rate - (2.0 / 3.0)).abs() < 1e-12);
    assert_eq!(execution.artifact.activity, "reactivated");
    assert!(execution.artifact.reactivation_identity_preserved);
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    assert_eq!(
        execute_topic_activity_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &recovered_input(),
            "2026-02-02T00:00:00Z",
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
            value.output_profile = "trsl_topic_lineage_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "fitted_candidate_k_v1".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request, &recovered_input()),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
