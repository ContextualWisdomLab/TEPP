//! Orchestration routing must bound billable compute and access-list resources.

use tepp_api::{
    ApiError, DocumentControlAttempt, InterpretationTaskKind, MAX_ORCHESTRATION_ACCESS_ENTRIES,
    MAX_ORCHESTRATION_ACCESS_TOKEN_BYTES, MAX_ORCHESTRATION_TOKEN_BUDGET,
    ORCHESTRATION_POLICY_VERSION, OrchestrationMode, OrchestrationRequest, route_orchestration,
};

fn request() -> OrchestrationRequest {
    OrchestrationRequest {
        policy_version: ORCHESTRATION_POLICY_VERSION.into(),
        task_kind: InterpretationTaskKind::SpanClassification,
        risk_score: 0.1,
        ambiguity_score: 0.1,
        evidence_sufficiency: 1.0,
        compute_budget_tokens: 4_000,
        document_control: DocumentControlAttempt::None,
        scientific_gate_passed: true,
        access_list: vec!["evidence.read".into()],
    }
}

#[test]
fn token_budget_is_bounded_before_any_plan_is_created() {
    let mut oversized = request();
    oversized.compute_budget_tokens = MAX_ORCHESTRATION_TOKEN_BUDGET + 1;
    assert_eq!(
        route_orchestration(&oversized),
        Err(ApiError::LimitExceeded)
    );

    let mut boundary = request();
    boundary.compute_budget_tokens = MAX_ORCHESTRATION_TOKEN_BUDGET;
    assert!(route_orchestration(&boundary).is_ok());
}

#[test]
fn access_list_cardinality_and_token_bytes_are_bounded() {
    let mut too_many = request();
    too_many.access_list = (0..=MAX_ORCHESTRATION_ACCESS_ENTRIES)
        .map(|index| format!("scope.{index}"))
        .collect();
    assert_eq!(route_orchestration(&too_many), Err(ApiError::LimitExceeded));

    let mut too_long = request();
    too_long.access_list = vec!["x".repeat(MAX_ORCHESTRATION_ACCESS_TOKEN_BYTES + 1)];
    assert_eq!(route_orchestration(&too_long), Err(ApiError::LimitExceeded));
}

#[test]
fn duplicate_access_tokens_fail_closed_instead_of_amplifying_authority() {
    let mut duplicate = request();
    duplicate.access_list = vec!["evidence.read".into(), "evidence.read".into()];
    assert_eq!(
        route_orchestration(&duplicate),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn verify_mode_falls_back_to_direct_at_the_direct_budget_boundary() {
    let mut budgeted = request();
    budgeted.risk_score = 0.35;
    budgeted.compute_budget_tokens = OrchestrationMode::Direct.minimum_token_budget();

    let plan = route_orchestration(&budgeted).expect("bounded request");
    assert_eq!(plan.mode(), OrchestrationMode::Direct);
}
