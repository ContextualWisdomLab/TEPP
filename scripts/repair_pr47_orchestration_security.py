"""Apply PR 47 evidence-digest and comparable-ablation security repairs."""

from pathlib import Path


CANONICAL_DIGEST = "sha256:" + "a" * 64


def replace_once(text: str, old: str, new: str, label: str) -> str:
    """Replace exactly one fragment or fail closed."""
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


def update_orchestration_module() -> None:
    """Bind plans to task context and evidence manifests."""
    path = Path("crates/tepp_api/src/orchestration.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        """pub struct OrchestrationPlan {
    mode: OrchestrationMode,
""",
        """pub struct OrchestrationPlan {
    mode: OrchestrationMode,
    task_kind: InterpretationTaskKind,
""",
        "plan task identity",
    )
    text = replace_once(
        text,
        """    /// Recorded workflow stage count.
    #[must_use]
    pub const fn stage_count(&self) -> u8 {
""",
        """    /// Interpretation task whose context this plan represents.
    #[must_use]
    pub const fn task_kind(&self) -> InterpretationTaskKind {
        self.task_kind
    }

    /// Recorded workflow stage count.
    #[must_use]
    pub const fn stage_count(&self) -> u8 {
""",
        "plan task getter",
    )
    text = replace_once(
        text,
        """    if baseline.mode != OrchestrationMode::Direct {
        return Err(ApiError::InvalidWirePayload);
    }
""",
        """    if baseline.mode != OrchestrationMode::Direct
        || baseline.task_kind != compared.task_kind
        || baseline.policy_version != compared.policy_version
        || baseline.access_list != compared.access_list
    {
        return Err(ApiError::InvalidWirePayload);
    }
""",
        "ablation context gate",
    )
    text = replace_once(
        text,
        """/// Returns [`ApiError::InvalidWirePayload`] when the evidence manifest hash is
/// empty.
""",
        """/// Returns [`ApiError::InvalidWirePayload`] unless the evidence manifest is a
/// canonical lowercase `sha256:` digest rather than raw source text.
""",
        "binding error documentation",
    )
    text = replace_once(
        text,
        """    require_nonempty(evidence_manifest_hash)?;
    if plan.mode == OrchestrationMode::Abstain {
""",
        """    require_sha256_digest(evidence_manifest_hash)?;
    if plan.mode == OrchestrationMode::Abstain {
""",
        "binding digest validation",
    )
    text = replace_once(
        text,
        "fn validate_request(request: &OrchestrationRequest) -> Result<(), ApiError> {\n",
        """fn require_sha256_digest(value: &str) -> Result<(), ApiError> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(ApiError::InvalidWirePayload);
    };
    if digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn validate_request(request: &OrchestrationRequest) -> Result<(), ApiError> {
""",
        "digest helper insertion",
    )
    text = replace_once(
        text,
        """    OrchestrationPlan {
        mode,
        stages: mode.stage_count(),
""",
        """    OrchestrationPlan {
        mode,
        task_kind: request.task_kind,
        stages: mode.stage_count(),
""",
        "plan construction",
    )
    text = replace_once(
        text,
        """        let direct = route_orchestration(&request(
            InterpretationTaskKind::SchemaConversion,
            1.0,
            0.0,
            1.0,
            8_000,
        ))
""",
        """        let direct = route_orchestration(&request(
            InterpretationTaskKind::SpanClassification,
            0.1,
            0.1,
            1.0,
            8_000,
        ))
""",
        "internal direct ablation",
    )
    text = replace_once(
        text,
        """        let verify = route_orchestration(&request(
            InterpretationTaskKind::AdversarialVerification,
            0.0,
            0.0,
            1.0,
            32_000,
        ))
""",
        """        let verify = route_orchestration(&request(
            InterpretationTaskKind::SpanClassification,
            0.8,
            0.1,
            1.0,
            32_000,
        ))
""",
        "internal verify ablation",
    )
    text = replace_once(
        text,
        'bind_contextual_orchestrator(&verify, "sha256:manifest")',
        f'bind_contextual_orchestrator(&verify, "{CANONICAL_DIGEST}")',
        "internal binding digest",
    )
    path.write_text(text, encoding="utf-8")


def update_public_contract_tests() -> None:
    """Keep existing ablation and binding examples scientifically comparable."""
    path = Path("crates/tepp_api/tests/orchestration_router_contract.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        """    let verify = route_orchestration(&request(
        InterpretationTaskKind::AdversarialVerification,
        0.20,
        0.20,
        0.80,
        16_000,
    ))
""",
        """    let verify = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.80,
        0.20,
        0.80,
        16_000,
    ))
""",
        "public comparable task",
    )
    text = replace_once(
        text,
        """    let abstain = route_orchestration(&request(
        InterpretationTaskKind::NarrativeSynthesis,
        0.80,
        0.80,
        0.10,
        32_000,
    ))
""",
        """    let abstain = route_orchestration(&request(
        InterpretationTaskKind::SpanClassification,
        0.10,
        0.10,
        0.10,
        32_000,
    ))
""",
        "public comparable abstention",
    )
    text = text.replace('"sha256:evidence-manifest-1"', f'"{CANONICAL_DIGEST}"')
    text = text.replace('"sha256:evidence-manifest-1"\n    );', f'"{CANONICAL_DIGEST}"\n    );')
    path.write_text(text, encoding="utf-8")


def update_security_test() -> None:
    """Cover the public task identity getter on the valid baseline."""
    path = Path("crates/tepp_api/tests/orchestration_security_contract.rs")
    text = path.read_text(encoding="utf-8")
    anchor = """    .expect("direct baseline");
    let comparable = route_orchestration(&request(
"""
    replacement = """    .expect("direct baseline");
    assert_eq!(baseline.task_kind(), InterpretationTaskKind::SpanClassification);
    let comparable = route_orchestration(&request(
"""
    path.write_text(replace_once(text, anchor, replacement, "task getter coverage"), encoding="utf-8")


update_orchestration_module()
update_public_contract_tests()
update_security_test()
