//! Adaptive orchestration routing refuses document-controlled policy.

use tepp_api::{
    ApiError, DocumentControlAttempt, InterpretationTaskKind, ORCHESTRATION_CONTRACT_VERSION,
    ORCHESTRATION_POLICY_VERSION, OrchestrationMode, OrchestrationRequest, OrchestrationRole,
    ReasoningEffort, bind_contextual_orchestrator, record_budget_ablation, route_orchestration,
};

fn request(
    task_kind: InterpretationTaskKind,
    risk: f64,
    ambiguity: f64,
    evidence: f64,
    budget: u64,
) -> OrchestrationRequest {
    OrchestrationRequest {
        policy_version: ORCHESTRATION_POLICY_VERSION.into(),
        task_kind,
        risk_score: risk,
        ambiguity_score: ambiguity,
        evidence_sufficiency: evidence,
        compute_budget_tokens: budget,
        document_control: DocumentControlAttempt::None,
        scientific_gate_passed: true,
        access_list: vec!["evidence_spans".into()],
    }
}

#[test]
fn low_risk_span_classification_routes_direct() {
    let plan = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.10,
        0.10,
        0.90,
        8_000,
    ))
    .expect("direct");
    assert_eq!(plan.mode(), OrchestrationMode::Direct);
    assert_eq!(plan.mode().wire_name(), "direct");
    assert_eq!(plan.stage_count(), 1);
    assert_eq!(plan.recursion_depth(), 0);
    assert_eq!(plan.decomposition_code(), "single_call");
    assert_eq!(plan.fallback_mode(), OrchestrationMode::Abstain);
    assert!(plan.proposal_only());
    assert_eq!(
        plan.scientific_authority_code(),
        "deterministic_statistical_gates"
    );
    assert_eq!(plan.policy_version(), ORCHESTRATION_POLICY_VERSION);
    assert_eq!(plan.access_list(), ["evidence_spans"]);
    assert_eq!(plan.roles().len(), 1);
    assert_eq!(plan.roles()[0].role(), OrchestrationRole::Worker);
    assert_eq!(plan.roles()[0].effort(), ReasoningEffort::Low);
    assert_eq!(plan.token_budget(), 8_000);
    assert!(!plan.to_string().contains("token"));
}

#[test]
fn schema_conversion_is_direct_with_minimal_effort() {
    let plan = route_orchestration(&request(
        InterpretationTaskKind::SchemaConversion,
        0.90,
        0.90,
        0.80,
        4_000,
    ))
    .expect("schema");
    assert_eq!(plan.mode(), OrchestrationMode::Direct);
    assert_eq!(plan.roles()[0].effort(), ReasoningEffort::Minimal);
    assert_eq!(
        InterpretationTaskKind::SchemaConversion.default_effort(),
        ReasoningEffort::Minimal
    );
}

#[test]
fn material_risk_or_adversarial_work_routes_verify() {
    let high_risk = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.70,
        0.20,
        0.80,
        12_000,
    ))
    .expect("high risk");
    assert_eq!(high_risk.mode(), OrchestrationMode::Verify);
    assert_eq!(high_risk.decomposition_code(), "producer_then_verifier");
    assert_eq!(high_risk.stage_count(), 2);
    assert_eq!(high_risk.roles().len(), 2);
    assert_eq!(high_risk.roles()[1].role(), OrchestrationRole::Verifier);
    assert_eq!(high_risk.roles()[1].effort(), ReasoningEffort::High);
    assert_eq!(high_risk.fallback_mode(), OrchestrationMode::Direct);

    let adversarial = route_orchestration(&request(
        InterpretationTaskKind::AdversarialVerification,
        0.20,
        0.20,
        0.80,
        12_000,
    ))
    .expect("adversarial");
    assert_eq!(adversarial.mode(), OrchestrationMode::Verify);
    assert_eq!(
        InterpretationTaskKind::AdversarialVerification.default_effort(),
        ReasoningEffort::High
    );
}

#[test]
fn concept_alignment_and_low_narrative_use_verify_or_committee() {
    let aligned = route_orchestration(&request(
        InterpretationTaskKind::ConceptAlignment,
        0.20,
        0.20,
        0.80,
        16_000,
    ))
    .expect("concept low");
    assert_eq!(aligned.mode(), OrchestrationMode::Verify);
    assert_eq!(
        InterpretationTaskKind::ConceptAlignment.default_effort(),
        ReasoningEffort::Medium
    );

    let ambiguous = route_orchestration(&request(
        InterpretationTaskKind::ConceptAlignment,
        0.20,
        0.70,
        0.80,
        16_000,
    ))
    .expect("concept high");
    assert_eq!(ambiguous.mode(), OrchestrationMode::Committee);
    assert_eq!(
        ambiguous.decomposition_code(),
        "blinded_parallel_then_adjudicate"
    );
    assert_eq!(ambiguous.stage_count(), 3);
    assert!(
        ambiguous
            .roles()
            .iter()
            .any(|role| role.role() == OrchestrationRole::Adjudicator)
    );

    let narrative_low = route_orchestration(&request(
        InterpretationTaskKind::NarrativeSynthesis,
        0.20,
        0.20,
        0.80,
        16_000,
    ))
    .expect("narrative low");
    assert_eq!(narrative_low.mode(), OrchestrationMode::Verify);
}

#[test]
fn blinded_review_requires_scientific_gate_and_uses_committee() {
    let passed = route_orchestration(&request(
        InterpretationTaskKind::BlindedModelReview,
        0.40,
        0.40,
        0.80,
        20_000,
    ))
    .expect("committee");
    assert_eq!(passed.mode(), OrchestrationMode::Committee);
    assert_eq!(
        InterpretationTaskKind::BlindedModelReview.default_effort(),
        ReasoningEffort::High
    );

    let mut rejected = request(
        InterpretationTaskKind::BlindedModelReview,
        0.40,
        0.40,
        0.80,
        20_000,
    );
    rejected.scientific_gate_passed = false;
    let abstain = route_orchestration(&rejected).expect("llm cannot rescue");
    assert_eq!(abstain.mode(), OrchestrationMode::Abstain);
    assert_eq!(abstain.decomposition_code(), "no_forced_answer");
    assert!(abstain.roles().is_empty());
    assert_eq!(abstain.token_budget(), 0);
}

#[test]
fn complex_synthesis_routes_conductor_when_budget_allows() {
    let plan = route_orchestration(&request(
        InterpretationTaskKind::NarrativeSynthesis,
        0.60,
        0.70,
        0.85,
        32_000,
    ))
    .expect("conductor");
    assert_eq!(plan.mode(), OrchestrationMode::Conductor);
    assert_eq!(plan.mode().wire_name(), "conductor");
    assert_eq!(plan.decomposition_code(), "adaptive_roles_under_budget");
    assert_eq!(plan.recursion_depth(), 2);
    assert!(
        plan.roles()
            .iter()
            .any(|role| role.role() == OrchestrationRole::Conductor)
    );
    assert_eq!(plan.fallback_mode(), OrchestrationMode::Committee);
}

#[test]
fn insufficient_evidence_and_tiny_budget_abstain() {
    let evidence = route_orchestration(&request(
        InterpretationTaskKind::NarrativeSynthesis,
        0.80,
        0.80,
        0.20,
        32_000,
    ))
    .expect("evidence");
    assert_eq!(evidence.mode(), OrchestrationMode::Abstain);
    assert_eq!(evidence.mode().wire_name(), "abstain");

    let budget = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.10,
        0.10,
        0.90,
        100,
    ))
    .expect("tiny budget");
    assert_eq!(budget.mode(), OrchestrationMode::Abstain);
}

#[test]
fn budget_steps_down_without_changing_scientific_authority() {
    let stepped = route_orchestration(&request(
        InterpretationTaskKind::NarrativeSynthesis,
        0.70,
        0.70,
        0.90,
        10_000,
    ))
    .expect("step down");
    assert_eq!(stepped.mode(), OrchestrationMode::Verify);
    assert_eq!(
        stepped.scientific_authority_code(),
        "deterministic_statistical_gates"
    );
    assert!(stepped.proposal_only());
}

#[test]
fn document_controlled_policy_access_or_credentials_are_denied() {
    for attempt in [
        DocumentControlAttempt::Policy,
        DocumentControlAttempt::AccessList,
        DocumentControlAttempt::Credentials,
    ] {
        let mut hostile = request(
            InterpretationTaskKind::SpanClassification,
            0.10,
            0.10,
            0.90,
            8_000,
        );
        hostile.document_control = attempt;
        assert_eq!(
            route_orchestration(&hostile),
            Err(ApiError::AuthorizationDenied)
        );
    }
}

#[test]
fn invalid_scores_policy_version_and_access_tokens_fail_closed() {
    let mut version = request(
        InterpretationTaskKind::SpanClassification,
        0.10,
        0.10,
        0.90,
        8_000,
    );
    version.policy_version = "tepp.orchestration.v0".into();
    assert_eq!(
        route_orchestration(&version),
        Err(ApiError::UnsupportedContractVersion)
    );

    for (risk, ambiguity, evidence) in [
        (f64::NAN, 0.1, 0.9),
        (0.1, f64::INFINITY, 0.9),
        (0.1, 0.1, -0.01),
        (1.01, 0.1, 0.9),
    ] {
        assert_eq!(
            route_orchestration(&request(
                InterpretationTaskKind::SpanClassification,
                risk,
                ambiguity,
                evidence,
                8_000,
            )),
            Err(ApiError::InvalidWirePayload)
        );
    }

    let mut empty_token = request(
        InterpretationTaskKind::SpanClassification,
        0.10,
        0.10,
        0.90,
        8_000,
    );
    empty_token.access_list = vec!["   ".into()];
    assert_eq!(
        route_orchestration(&empty_token),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn comparable_budget_ablation_requires_direct_baseline() {
    let direct = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.10,
        0.10,
        0.90,
        16_000,
    ))
    .expect("direct");
    let verify = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.80,
        0.20,
        0.80,
        16_000,
    ))
    .expect("verify");
    let record = record_budget_ablation(&direct, &verify).expect("ablation");
    assert_eq!(record.baseline_mode(), OrchestrationMode::Direct);
    assert_eq!(record.compared_mode(), OrchestrationMode::Verify);
    assert_eq!(record.baseline_budget(), 16_000);
    assert_eq!(record.compared_budget(), 16_000);
    assert!(record.comparable());

    let conductor = route_orchestration(&request(
        InterpretationTaskKind::NarrativeSynthesis,
        0.70,
        0.70,
        0.90,
        32_000,
    ))
    .expect("conductor");
    assert_eq!(
        record_budget_ablation(&conductor, &direct),
        Err(ApiError::InvalidWirePayload)
    );

    let abstain = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.10,
        0.10,
        0.10,
        32_000,
    ))
    .expect("abstain");
    let not_comparable = record_budget_ablation(&direct, &abstain).expect("report difference");
    assert!(!not_comparable.comparable());
}

#[test]
fn contextual_orchestrator_binding_never_carries_credentials() {
    let plan = route_orchestration(&request(
        InterpretationTaskKind::ConceptAlignment,
        0.20,
        0.20,
        0.80,
        16_000,
    ))
    .expect("verify");
    let binding = bind_contextual_orchestrator(
        &plan,
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )
    .expect("bind");
    assert_eq!(binding.contract_version(), ORCHESTRATION_CONTRACT_VERSION);
    assert_eq!(binding.mode(), OrchestrationMode::Verify);
    assert_eq!(
        binding.evidence_manifest_hash(),
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert!(!binding.includes_credentials());
    assert_eq!(binding.access_list(), plan.access_list());
    assert_eq!(binding.policy_version(), ORCHESTRATION_POLICY_VERSION);

    let abstain = route_orchestration(&request(
        InterpretationTaskKind::NarrativeSynthesis,
        0.80,
        0.80,
        0.10,
        8_000,
    ))
    .expect("abstain");
    assert_eq!(
        bind_contextual_orchestrator(
            &abstain,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        bind_contextual_orchestrator(&plan, ""),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn wire_names_cover_every_public_variant() {
    assert_eq!(OrchestrationMode::Verify.wire_name(), "verify");
    assert_eq!(OrchestrationMode::Committee.wire_name(), "committee");
    assert_eq!(ReasoningEffort::Low.wire_name(), "low");
    assert_eq!(ReasoningEffort::Medium.wire_name(), "medium");
    assert_eq!(ReasoningEffort::High.wire_name(), "high");
    assert_eq!(ReasoningEffort::Minimal.wire_name(), "minimal");
    assert_eq!(
        InterpretationTaskKind::SpanClassification.wire_name(),
        "span_classification"
    );
    assert_eq!(
        InterpretationTaskKind::ConceptAlignment.wire_name(),
        "concept_alignment"
    );
    assert_eq!(
        InterpretationTaskKind::BlindedModelReview.wire_name(),
        "blinded_model_review"
    );
    assert_eq!(
        InterpretationTaskKind::NarrativeSynthesis.wire_name(),
        "narrative_synthesis"
    );
    assert_eq!(
        InterpretationTaskKind::AdversarialVerification.wire_name(),
        "adversarial_verification"
    );
    assert_eq!(
        InterpretationTaskKind::SchemaConversion.wire_name(),
        "schema_conversion"
    );
    assert_eq!(OrchestrationRole::Thinker.wire_name(), "thinker");
    assert_eq!(OrchestrationRole::Worker.wire_name(), "worker");
    assert_eq!(OrchestrationRole::Verifier.wire_name(), "verifier");
    assert_eq!(OrchestrationRole::Adjudicator.wire_name(), "adjudicator");
    assert_eq!(OrchestrationRole::Conductor.wire_name(), "conductor");
    assert_eq!(
        InterpretationTaskKind::SpanClassification.default_effort(),
        ReasoningEffort::Low
    );
    assert_eq!(
        InterpretationTaskKind::NarrativeSynthesis.default_effort(),
        ReasoningEffort::Medium
    );
}
