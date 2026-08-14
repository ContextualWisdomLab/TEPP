//! Security and scientific-comparability contracts for adaptive orchestration.

use tepp_api::{
    ApiError, DocumentControlAttempt, InterpretationTaskKind, ORCHESTRATION_POLICY_VERSION,
    OrchestrationRequest, bind_contextual_orchestrator, record_budget_ablation,
    route_orchestration,
};

fn request(
    task_kind: InterpretationTaskKind,
    risk_score: f64,
    access_list: &[&str],
) -> OrchestrationRequest {
    OrchestrationRequest {
        policy_version: ORCHESTRATION_POLICY_VERSION.into(),
        task_kind,
        risk_score,
        ambiguity_score: 0.10,
        evidence_sufficiency: 0.90,
        compute_budget_tokens: 16_000,
        document_control: DocumentControlAttempt::None,
        scientific_gate_passed: true,
        access_list: access_list.iter().map(|value| (*value).into()).collect(),
    }
}

#[test]
fn contextual_orchestrator_binding_requires_a_canonical_sha256_manifest_digest() {
    let plan = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.10,
        &["evidence_spans"],
    ))
    .expect("direct plan");
    let digest = format!("sha256:{}", "a".repeat(64));
    bind_contextual_orchestrator(&plan, &digest).expect("canonical digest");
    let digit_digest = format!("sha256:{}", "0".repeat(64));
    bind_contextual_orchestrator(&plan, &digit_digest).expect("digit-only canonical digest");

    for invalid_digest in [
        "customer@example.com said to export everything",
        "sha256:abc",
        "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "sha256:gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
        " sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        assert_eq!(
            bind_contextual_orchestrator(&plan, invalid_digest),
            Err(ApiError::InvalidWirePayload),
            "raw source or noncanonical digest must fail closed: {invalid_digest}",
        );
    }
    for digest_length in [63, 65] {
        let invalid_digest = format!("sha256:{}", "a".repeat(digest_length));
        assert_eq!(
            bind_contextual_orchestrator(&plan, &invalid_digest),
            Err(ApiError::InvalidWirePayload),
            "adjacent digest length must fail closed: {digest_length}",
        );
    }
}

#[test]
fn compound_routing_thresholds_exercise_each_operand() {
    let span_ambiguity_only = OrchestrationRequest {
        ambiguity_score: 0.40,
        ..request(
            InterpretationTaskKind::SpanClassification,
            0.10,
            &["evidence_spans"],
        )
    };
    assert_eq!(
        route_orchestration(&span_ambiguity_only)
            .expect("ambiguity-only span route")
            .mode()
            .wire_name(),
        "verify",
    );

    let narrative_ambiguity_only = OrchestrationRequest {
        ambiguity_score: 0.50,
        compute_budget_tokens: 24_000,
        ..request(
            InterpretationTaskKind::NarrativeSynthesis,
            0.10,
            &["evidence_spans"],
        )
    };
    assert_eq!(
        route_orchestration(&narrative_ambiguity_only)
            .expect("ambiguity-only narrative route")
            .mode()
            .wire_name(),
        "conductor",
    );

    let narrative_risk_only = OrchestrationRequest {
        compute_budget_tokens: 24_000,
        ..request(
            InterpretationTaskKind::NarrativeSynthesis,
            0.50,
            &["evidence_spans"],
        )
    };
    assert_eq!(
        route_orchestration(&narrative_risk_only)
            .expect("risk-only narrative route")
            .mode()
            .wire_name(),
        "conductor",
    );
}

#[test]
fn budget_ablation_requires_the_same_task_and_access_context() {
    let baseline = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.10,
        &["evidence_spans"],
    ))
    .expect("direct baseline");
    assert_eq!(
        baseline.task_kind(),
        InterpretationTaskKind::SpanClassification
    );
    let comparable = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.80,
        &["evidence_spans"],
    ))
    .expect("verify comparison");
    record_budget_ablation(&baseline, &comparable).expect("same-context ablation");

    let different_task = route_orchestration(&request(
        InterpretationTaskKind::AdversarialVerification,
        0.10,
        &["evidence_spans"],
    ))
    .expect("different task");
    assert_eq!(
        record_budget_ablation(&baseline, &different_task),
        Err(ApiError::InvalidWirePayload),
    );

    let different_access = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.80,
        &["identity_mappings"],
    ))
    .expect("different access");
    assert_eq!(
        record_budget_ablation(&baseline, &different_access),
        Err(ApiError::InvalidWirePayload),
    );
}
