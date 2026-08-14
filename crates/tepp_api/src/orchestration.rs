//! Deterministic ADR 0010 orchestration routing and orchestrator binding.

use crate::ApiError;
use crate::wire::require_nonempty;
use std::fmt;

/// Versioned TEPP orchestration policy identity.
pub const ORCHESTRATION_POLICY_VERSION: &str = "tepp.orchestration.v1";

/// Contract version for a contextual-orchestrator binding.
pub const ORCHESTRATION_CONTRACT_VERSION: u16 = 1;

/// Maximum billable token budget accepted by one orchestration request.
pub const MAX_ORCHESTRATION_TOKEN_BUDGET: u64 = 1_000_000;
/// Maximum number of TEPP-owned access capabilities on one request.
pub const MAX_ORCHESTRATION_ACCESS_ENTRIES: usize = 64;
/// Maximum UTF-8 byte length of one access capability token.
pub const MAX_ORCHESTRATION_ACCESS_TOKEN_BYTES: usize = 128;

const EVIDENCE_FLOOR: f64 = 0.35;
const LOW_COMPLEXITY: f64 = 0.35;
const HIGH_COMPLEXITY: f64 = 0.50;
const COMPARABLE_BUDGET_NUMERATOR: u64 = 10;

/// Versioned orchestration mode selected by the governed router.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationMode {
    /// One model call for low-ambiguity schema-constrained work.
    Direct,
    /// Producer plus an independent verifier.
    Verify,
    /// Blinded parallel raters plus adjudication.
    Committee,
    /// Adaptive roles and topology under an explicit budget.
    Conductor,
    /// No forced answer when evidence, budget, or gates are insufficient.
    Abstain,
}

impl OrchestrationMode {
    /// Return the stable wire name for this mode.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Verify => "verify",
            Self::Committee => "committee",
            Self::Conductor => "conductor",
            Self::Abstain => "abstain",
        }
    }

    /// Minimum token budget required to execute this mode.
    #[must_use]
    pub const fn minimum_token_budget(self) -> u64 {
        match self {
            Self::Direct => 4_000,
            Self::Verify => 8_000,
            Self::Committee => 16_000,
            Self::Conductor => 24_000,
            Self::Abstain => 0,
        }
    }

    /// Default workflow stage count recorded for this mode.
    #[must_use]
    pub const fn stage_count(self) -> u8 {
        match self {
            Self::Direct => 1,
            Self::Verify => 2,
            Self::Committee => 3,
            Self::Conductor => 4,
            Self::Abstain => 0,
        }
    }

    /// Default recursion depth recorded for this mode.
    #[must_use]
    pub const fn recursion_depth(self) -> u8 {
        match self {
            Self::Conductor => 2,
            Self::Direct | Self::Verify | Self::Committee | Self::Abstain => 0,
        }
    }

    /// Cheaper bounded fallback when this mode cannot complete.
    ///
    /// Keeping this match at one function boundary prevents callers from
    /// duplicating unreachable enum branches during coverage instrumentation.
    #[must_use]
    #[inline(never)]
    pub const fn fallback_mode(self) -> Self {
        match self {
            Self::Conductor => Self::Committee,
            Self::Committee => Self::Verify,
            Self::Verify => Self::Direct,
            Self::Direct | Self::Abstain => Self::Abstain,
        }
    }

    /// Decomposition code recorded on the plan.
    #[must_use]
    pub const fn decomposition_code(self) -> &'static str {
        match self {
            Self::Direct => "single_call",
            Self::Verify => "producer_then_verifier",
            Self::Committee => "blinded_parallel_then_adjudicate",
            Self::Conductor => "adaptive_roles_under_budget",
            Self::Abstain => "no_forced_answer",
        }
    }
}

/// Role-specific reasoning effort recorded for ablation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasoningEffort {
    /// Schema conversion or formatting.
    Minimal,
    /// Low-ambiguity span classification.
    Low,
    /// Concept alignment or narrative synthesis.
    Medium,
    /// Verification, adjudication, or blinded review.
    High,
}

impl ReasoningEffort {
    /// Return the stable wire name for this effort.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Bounded interpretation task the router may schedule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpretationTaskKind {
    /// Semantic span extraction or classification.
    SpanClassification,
    /// Concept merge or alignment review.
    ConceptAlignment,
    /// Blinded K/model-selection review after statistical gates.
    BlindedModelReview,
    /// Evidence-grounded narrative synthesis.
    NarrativeSynthesis,
    /// Adversarial unsupported-claim verification.
    AdversarialVerification,
    /// Routine schema conversion or formatting.
    SchemaConversion,
}

impl InterpretationTaskKind {
    /// Return the stable wire name for this task kind.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::SpanClassification => "span_classification",
            Self::ConceptAlignment => "concept_alignment",
            Self::BlindedModelReview => "blinded_model_review",
            Self::NarrativeSynthesis => "narrative_synthesis",
            Self::AdversarialVerification => "adversarial_verification",
            Self::SchemaConversion => "schema_conversion",
        }
    }

    /// Default reasoning effort for a worker assigned this task.
    #[must_use]
    pub const fn default_effort(self) -> ReasoningEffort {
        match self {
            Self::SchemaConversion => ReasoningEffort::Minimal,
            Self::SpanClassification => ReasoningEffort::Low,
            Self::ConceptAlignment | Self::NarrativeSynthesis => ReasoningEffort::Medium,
            Self::BlindedModelReview | Self::AdversarialVerification => ReasoningEffort::High,
        }
    }
}

/// Orchestration role assigned under the selected mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationRole {
    /// Lightweight coordinator or thinker.
    Thinker,
    /// Task worker that proposes an interpretation.
    Worker,
    /// Independent evidence-only verifier.
    Verifier,
    /// Committee adjudicator.
    Adjudicator,
    /// Adaptive conductor under an explicit budget.
    Conductor,
}

impl OrchestrationRole {
    /// Return the stable wire name for this role.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Thinker => "thinker",
            Self::Worker => "worker",
            Self::Verifier => "verifier",
            Self::Adjudicator => "adjudicator",
            Self::Conductor => "conductor",
        }
    }
}

/// One role plus its recorded reasoning effort.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoleAssignment {
    role: OrchestrationRole,
    effort: ReasoningEffort,
}

impl RoleAssignment {
    /// Assigned orchestration role.
    #[must_use]
    pub const fn role(self) -> OrchestrationRole {
        self.role
    }

    /// Recorded reasoning effort for this role.
    #[must_use]
    pub const fn effort(self) -> ReasoningEffort {
        self.effort
    }
}

/// Document attempt to override TEPP orchestration authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentControlAttempt {
    /// No document-supplied override.
    None,
    /// Document tried to set orchestration policy.
    Policy,
    /// Document tried to set the access list.
    AccessList,
    /// Document tried to supply provider credentials.
    Credentials,
}

/// Inputs to the governed orchestration router.
///
/// Scores are CPU `f64` unit intervals. Documents cannot set policy, access
/// lists, or credentials; those attempts fail closed.
#[derive(Clone, Debug)]
pub struct OrchestrationRequest {
    /// Must equal [`ORCHESTRATION_POLICY_VERSION`].
    pub policy_version: String,
    /// Interpretation task being scheduled.
    pub task_kind: InterpretationTaskKind,
    /// Task risk in `[0, 1]`.
    pub risk_score: f64,
    /// Ambiguity in `[0, 1]`.
    pub ambiguity_score: f64,
    /// Evidence sufficiency in `[0, 1]`.
    pub evidence_sufficiency: f64,
    /// Explicit test-time token budget.
    pub compute_budget_tokens: u64,
    /// Document-supplied override attempt, if any.
    pub document_control: DocumentControlAttempt,
    /// Whether deterministic scientific gates already passed.
    pub scientific_gate_passed: bool,
    /// TEPP-owned access profile identifiers.
    pub access_list: Vec<String>,
}

/// Recorded orchestration plan. Construct only via [`route_orchestration`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationPlan {
    mode: OrchestrationMode,
    task_kind: InterpretationTaskKind,
    stages: u8,
    recursion_depth: u8,
    decomposition_code: &'static str,
    access_list: Vec<String>,
    roles: Vec<RoleAssignment>,
    token_budget: u64,
    fallback_mode: OrchestrationMode,
    policy_version: String,
    proposal_only: bool,
    scientific_authority_code: &'static str,
}

impl OrchestrationPlan {
    /// Selected orchestration mode.
    #[must_use]
    pub const fn mode(&self) -> OrchestrationMode {
        self.mode
    }

    /// Interpretation task whose context this plan represents.
    #[must_use]
    pub const fn task_kind(&self) -> InterpretationTaskKind {
        self.task_kind
    }

    /// Recorded workflow stage count.
    #[must_use]
    pub const fn stage_count(&self) -> u8 {
        self.stages
    }

    /// Recorded recursion depth.
    #[must_use]
    pub const fn recursion_depth(&self) -> u8 {
        self.recursion_depth
    }

    /// Decomposition code used for ablation.
    #[must_use]
    pub const fn decomposition_code(&self) -> &'static str {
        self.decomposition_code
    }

    /// TEPP-owned access list copied onto the plan.
    #[must_use]
    pub fn access_list(&self) -> &[String] {
        &self.access_list
    }

    /// Role assignments with per-role reasoning effort.
    #[must_use]
    pub fn roles(&self) -> &[RoleAssignment] {
        &self.roles
    }

    /// Allocated test-time budget in tokens.
    #[must_use]
    pub const fn token_budget(&self) -> u64 {
        self.token_budget
    }

    /// Bounded fallback mode if this plan cannot complete.
    #[must_use]
    pub const fn fallback_mode(&self) -> OrchestrationMode {
        self.fallback_mode
    }

    /// Policy version bound into the plan.
    #[must_use]
    pub fn policy_version(&self) -> &str {
        &self.policy_version
    }

    /// LLM output remains a proposal, never scientific authority.
    #[must_use]
    pub const fn proposal_only(&self) -> bool {
        self.proposal_only
    }

    /// Stable code naming the authoritative scientific gate family.
    #[must_use]
    pub const fn scientific_authority_code(&self) -> &'static str {
        self.scientific_authority_code
    }
}

impl fmt::Display for OrchestrationPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "orchestration mode={} stages={} recursion={} proposal={}",
            self.mode.wire_name(),
            self.stages,
            self.recursion_depth,
            self.proposal_only
        )
    }
}

/// Comparable-budget ablation record. Direct is the required baseline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BudgetAblationRecord {
    baseline_mode: OrchestrationMode,
    compared_mode: OrchestrationMode,
    baseline_budget: u64,
    compared_budget: u64,
    comparable: bool,
}

impl BudgetAblationRecord {
    /// Required direct baseline mode.
    #[must_use]
    pub const fn baseline_mode(self) -> OrchestrationMode {
        self.baseline_mode
    }

    /// Mode compared against the direct baseline.
    #[must_use]
    pub const fn compared_mode(self) -> OrchestrationMode {
        self.compared_mode
    }

    /// Baseline token budget.
    #[must_use]
    pub const fn baseline_budget(self) -> u64 {
        self.baseline_budget
    }

    /// Compared token budget.
    #[must_use]
    pub const fn compared_budget(self) -> u64 {
        self.compared_budget
    }

    /// Whether the two budgets are within a 10 percent relative band.
    #[must_use]
    pub const fn comparable(self) -> bool {
        self.comparable
    }
}

/// Credential-free binding TEPP may hand to contextual-orchestrator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextualOrchestratorBinding {
    contract_version: u16,
    mode: OrchestrationMode,
    policy_version: String,
    evidence_manifest_hash: String,
    access_list: Vec<String>,
    roles: Vec<RoleAssignment>,
    token_budget: u64,
    includes_credentials: bool,
}

impl ContextualOrchestratorBinding {
    /// Binding contract version.
    #[must_use]
    pub const fn contract_version(&self) -> u16 {
        self.contract_version
    }

    /// Mode the orchestrator may execute.
    #[must_use]
    pub const fn mode(&self) -> OrchestrationMode {
        self.mode
    }

    /// Policy version the orchestrator must not replace.
    #[must_use]
    pub fn policy_version(&self) -> &str {
        &self.policy_version
    }

    /// Evidence manifest digest; not raw source.
    #[must_use]
    pub fn evidence_manifest_hash(&self) -> &str {
        &self.evidence_manifest_hash
    }

    /// TEPP-owned access list.
    #[must_use]
    pub fn access_list(&self) -> &[String] {
        &self.access_list
    }

    /// Role assignments copied from the plan.
    #[must_use]
    pub fn roles(&self) -> &[RoleAssignment] {
        &self.roles
    }

    /// Allocated budget copied from the plan.
    #[must_use]
    pub const fn token_budget(&self) -> u64 {
        self.token_budget
    }

    /// Credentials are never included on this binding.
    #[must_use]
    pub const fn includes_credentials(&self) -> bool {
        self.includes_credentials
    }
}

/// Route an interpretation task onto a versioned orchestration plan.
///
/// The selected plan is a proposal. Deterministic statistical gates remain
/// authoritative. Documents cannot change policy, access lists, or credentials.
///
/// # Errors
///
/// Returns [`ApiError::UnsupportedContractVersion`] when the policy version is
/// not [`ORCHESTRATION_POLICY_VERSION`]. Returns
/// [`ApiError::AuthorizationDenied`] when a document supplied policy, access,
/// or credentials. Returns [`ApiError::InvalidWirePayload`] for non-unit
/// scores or empty access-list tokens.
pub fn route_orchestration(request: &OrchestrationRequest) -> Result<OrchestrationPlan, ApiError> {
    validate_request(request)?;
    let preferred = preferred_mode(request);
    let minimum_mode = minimum_acceptable_mode(request.task_kind);
    let mode = fit_mode(preferred, request.compute_budget_tokens, minimum_mode);
    Ok(build_plan(request, mode))
}

/// Record a comparable-budget ablation against a required direct baseline.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when the baseline is not
/// [`OrchestrationMode::Direct`], or when the two plans disagree on task kind
/// or access list. Plans cannot disagree on policy version because the only
/// constructor, [`route_orchestration`], accepts the current version exactly.
pub fn record_budget_ablation(
    baseline: &OrchestrationPlan,
    compared: &OrchestrationPlan,
) -> Result<BudgetAblationRecord, ApiError> {
    if baseline.mode != OrchestrationMode::Direct
        || baseline.task_kind != compared.task_kind
        || baseline.access_list != compared.access_list
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(BudgetAblationRecord {
        baseline_mode: baseline.mode,
        compared_mode: compared.mode,
        baseline_budget: baseline.token_budget,
        compared_budget: compared.token_budget,
        comparable: budgets_are_comparable(baseline.token_budget, compared.token_budget),
    })
}

/// Bind a non-abstaining plan for contextual-orchestrator execution.
///
/// The binding never includes credentials or raw source. TEPP retains
/// scientific authority; the orchestrator may only execute the recorded mode.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for [`OrchestrationMode::Abstain`].
/// Returns [`ApiError::InvalidWirePayload`] unless the evidence manifest is a
/// canonical lowercase `sha256:` digest rather than raw source text.
pub fn bind_contextual_orchestrator(
    plan: &OrchestrationPlan,
    evidence_manifest_hash: &str,
) -> Result<ContextualOrchestratorBinding, ApiError> {
    require_sha256_digest(evidence_manifest_hash)?;
    if plan.mode == OrchestrationMode::Abstain {
        return Err(ApiError::AuthorizationDenied);
    }
    Ok(ContextualOrchestratorBinding {
        contract_version: ORCHESTRATION_CONTRACT_VERSION,
        mode: plan.mode,
        policy_version: plan.policy_version.clone(),
        evidence_manifest_hash: evidence_manifest_hash.to_owned(),
        access_list: plan.access_list.clone(),
        roles: plan.roles.clone(),
        token_budget: plan.token_budget,
        includes_credentials: false,
    })
}

fn require_sha256_digest(value: &str) -> Result<(), ApiError> {
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
    if request.policy_version != ORCHESTRATION_POLICY_VERSION {
        return Err(ApiError::UnsupportedContractVersion);
    }
    if request.document_control != DocumentControlAttempt::None {
        return Err(ApiError::AuthorizationDenied);
    }
    if request.compute_budget_tokens > MAX_ORCHESTRATION_TOKEN_BUDGET
        || request.access_list.len() > MAX_ORCHESTRATION_ACCESS_ENTRIES
    {
        return Err(ApiError::LimitExceeded);
    }
    require_unit_score(request.risk_score)?;
    require_unit_score(request.ambiguity_score)?;
    require_unit_score(request.evidence_sufficiency)?;
    for (index, token) in request.access_list.iter().enumerate() {
        require_nonempty(token)?;
        if token.len() > MAX_ORCHESTRATION_ACCESS_TOKEN_BYTES {
            return Err(ApiError::LimitExceeded);
        }
        if request.access_list[..index].contains(token) {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    Ok(())
}

fn require_unit_score(value: f64) -> Result<(), ApiError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn preferred_mode(request: &OrchestrationRequest) -> OrchestrationMode {
    if request.evidence_sufficiency < EVIDENCE_FLOOR {
        return OrchestrationMode::Abstain;
    }
    if request.task_kind == InterpretationTaskKind::BlindedModelReview
        && !request.scientific_gate_passed
    {
        return OrchestrationMode::Abstain;
    }
    match request.task_kind {
        InterpretationTaskKind::SchemaConversion => OrchestrationMode::Direct,
        InterpretationTaskKind::BlindedModelReview => OrchestrationMode::Committee,
        InterpretationTaskKind::AdversarialVerification => OrchestrationMode::Verify,
        InterpretationTaskKind::SpanClassification => {
            if request.risk_score < LOW_COMPLEXITY && request.ambiguity_score < LOW_COMPLEXITY {
                OrchestrationMode::Direct
            } else {
                OrchestrationMode::Verify
            }
        }
        InterpretationTaskKind::ConceptAlignment => {
            if request.ambiguity_score >= HIGH_COMPLEXITY {
                OrchestrationMode::Committee
            } else {
                OrchestrationMode::Verify
            }
        }
        InterpretationTaskKind::NarrativeSynthesis => {
            if request.risk_score >= HIGH_COMPLEXITY || request.ambiguity_score >= HIGH_COMPLEXITY {
                OrchestrationMode::Conductor
            } else {
                OrchestrationMode::Verify
            }
        }
    }
}

const fn minimum_acceptable_mode(task_kind: InterpretationTaskKind) -> OrchestrationMode {
    match task_kind {
        InterpretationTaskKind::BlindedModelReview => OrchestrationMode::Committee,
        InterpretationTaskKind::SpanClassification
        | InterpretationTaskKind::ConceptAlignment
        | InterpretationTaskKind::NarrativeSynthesis
        | InterpretationTaskKind::AdversarialVerification
        | InterpretationTaskKind::SchemaConversion => OrchestrationMode::Direct,
    }
}

fn fit_mode(
    preferred: OrchestrationMode,
    budget: u64,
    minimum_mode: OrchestrationMode,
) -> OrchestrationMode {
    let mut mode = preferred;
    loop {
        if budget >= mode.minimum_token_budget() {
            return mode;
        }
        if mode == minimum_mode {
            return OrchestrationMode::Abstain;
        }
        mode = mode.fallback_mode();
    }
}

fn build_plan(request: &OrchestrationRequest, mode: OrchestrationMode) -> OrchestrationPlan {
    let token_budget = if mode == OrchestrationMode::Abstain {
        0
    } else {
        request.compute_budget_tokens
    };
    OrchestrationPlan {
        mode,
        task_kind: request.task_kind,
        stages: mode.stage_count(),
        recursion_depth: mode.recursion_depth(),
        decomposition_code: mode.decomposition_code(),
        access_list: request.access_list.clone(),
        roles: assign_roles(mode, request.task_kind),
        token_budget,
        fallback_mode: mode.fallback_mode(),
        policy_version: request.policy_version.clone(),
        proposal_only: true,
        scientific_authority_code: "deterministic_statistical_gates",
    }
}

fn assign_roles(mode: OrchestrationMode, task: InterpretationTaskKind) -> Vec<RoleAssignment> {
    let worker = RoleAssignment {
        role: OrchestrationRole::Worker,
        effort: task.default_effort(),
    };
    match mode {
        OrchestrationMode::Direct => vec![worker],
        OrchestrationMode::Verify => vec![
            worker,
            RoleAssignment {
                role: OrchestrationRole::Verifier,
                effort: ReasoningEffort::High,
            },
        ],
        OrchestrationMode::Committee => vec![
            worker,
            worker,
            RoleAssignment {
                role: OrchestrationRole::Adjudicator,
                effort: ReasoningEffort::High,
            },
        ],
        OrchestrationMode::Conductor => vec![
            RoleAssignment {
                role: OrchestrationRole::Conductor,
                effort: ReasoningEffort::High,
            },
            RoleAssignment {
                role: OrchestrationRole::Thinker,
                effort: task.default_effort(),
            },
            worker,
            RoleAssignment {
                role: OrchestrationRole::Verifier,
                effort: ReasoningEffort::High,
            },
        ],
        OrchestrationMode::Abstain => Vec::new(),
    }
}

fn budgets_are_comparable(left: u64, right: u64) -> bool {
    if left == 0 || right == 0 {
        return false;
    }
    left.abs_diff(right)
        .saturating_mul(COMPARABLE_BUDGET_NUMERATOR)
        <= left.max(right)
}

#[cfg(test)]
mod tests {
    use super::{
        DocumentControlAttempt, InterpretationTaskKind, ORCHESTRATION_CONTRACT_VERSION,
        ORCHESTRATION_POLICY_VERSION, OrchestrationMode, OrchestrationRequest, OrchestrationRole,
        ReasoningEffort, bind_contextual_orchestrator, budgets_are_comparable,
        record_budget_ablation, route_orchestration,
    };
    use crate::ApiError;

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
            access_list: Vec::new(),
        }
    }

    #[test]
    fn thresholds_and_fallbacks_cover_remaining_arms() {
        let boundary = route_orchestration(&request(
            InterpretationTaskKind::SpanClassification,
            0.35,
            0.10,
            0.35,
            8_000,
        ))
        .expect("boundary");
        assert_eq!(boundary.mode(), OrchestrationMode::Verify);
        assert_eq!(boundary.fallback_mode(), OrchestrationMode::Direct);
        assert_eq!(
            OrchestrationMode::Committee.fallback_mode(),
            OrchestrationMode::Verify
        );
        assert_eq!(
            OrchestrationMode::Abstain.fallback_mode(),
            OrchestrationMode::Abstain
        );
        assert_eq!(OrchestrationMode::Committee.recursion_depth(), 0);
        assert_eq!(OrchestrationMode::Conductor.stage_count(), 4);
        assert_eq!(OrchestrationMode::Direct.minimum_token_budget(), 4_000);
        assert_eq!(OrchestrationMode::Verify.minimum_token_budget(), 8_000);
        assert_eq!(OrchestrationMode::Committee.minimum_token_budget(), 16_000);
        assert_eq!(OrchestrationMode::Conductor.minimum_token_budget(), 24_000);
        assert_eq!(OrchestrationMode::Abstain.minimum_token_budget(), 0);

        let narrative_boundary = route_orchestration(&request(
            InterpretationTaskKind::NarrativeSynthesis,
            0.50,
            0.10,
            0.90,
            24_000,
        ))
        .expect("narrative boundary");
        assert_eq!(narrative_boundary.mode(), OrchestrationMode::Conductor);
        assert_eq!(narrative_boundary.stage_count(), 4);

        let committee_budget = route_orchestration(&request(
            InterpretationTaskKind::NarrativeSynthesis,
            0.70,
            0.70,
            0.90,
            16_000,
        ))
        .expect("committee fit");
        assert_eq!(committee_budget.mode(), OrchestrationMode::Committee);
        assert_eq!(committee_budget.fallback_mode(), OrchestrationMode::Verify);
        assert_eq!(committee_budget.roles().len(), 3);

        let underfunded_blinded_review = route_orchestration(&request(
            InterpretationTaskKind::BlindedModelReview,
            0.40,
            0.40,
            0.80,
            8_000,
        ))
        .expect("underfunded blinded review");
        assert_eq!(
            underfunded_blinded_review.mode(),
            OrchestrationMode::Abstain
        );
    }

    #[test]
    fn ablation_getters_and_rejection_paths_are_observable() {
        assert!(!budgets_are_comparable(8_000, 32_000));
        assert!(budgets_are_comparable(16_000, 16_000));
        assert!(!budgets_are_comparable(0, 16_000));

        let direct = route_orchestration(&request(
            InterpretationTaskKind::SpanClassification,
            0.1,
            0.1,
            1.0,
            8_000,
        ))
        .expect("direct");
        let verify = route_orchestration(&request(
            InterpretationTaskKind::SpanClassification,
            0.8,
            0.1,
            1.0,
            32_000,
        ))
        .expect("verify");
        let record = record_budget_ablation(&direct, &verify).expect("wide band");
        assert!(!record.comparable());
        assert_eq!(record.baseline_mode(), OrchestrationMode::Direct);
        assert_eq!(record.compared_mode(), OrchestrationMode::Verify);
        assert_eq!(record.baseline_budget(), 8_000);
        assert_eq!(record.compared_budget(), 32_000);
        assert!(
            record_budget_ablation(&direct, &direct)
                .expect("same-budget direct comparison")
                .comparable()
        );

        let committee = route_orchestration(&request(
            InterpretationTaskKind::BlindedModelReview,
            0.8,
            0.8,
            1.0,
            32_000,
        ))
        .expect("committee");
        assert_eq!(
            record_budget_ablation(&committee, &direct),
            Err(ApiError::InvalidWirePayload),
        );
        assert_eq!(
            record_budget_ablation(&direct, &committee),
            Err(ApiError::InvalidWirePayload),
        );
        let mut different_access = request(
            InterpretationTaskKind::SpanClassification,
            0.8,
            0.1,
            1.0,
            32_000,
        );
        different_access.access_list.push("evidence_spans".into());
        let different_access = route_orchestration(&different_access).expect("different access");
        assert_eq!(
            record_budget_ablation(&direct, &different_access),
            Err(ApiError::InvalidWirePayload),
        );
    }

    #[test]
    fn binding_getters_and_rejection_paths_are_observable() {
        let direct = route_orchestration(&request(
            InterpretationTaskKind::SpanClassification,
            0.1,
            0.1,
            1.0,
            8_000,
        ))
        .expect("direct");
        let verify = route_orchestration(&request(
            InterpretationTaskKind::SpanClassification,
            0.8,
            0.1,
            1.0,
            32_000,
        ))
        .expect("verify");

        let binding = bind_contextual_orchestrator(
            &verify,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .expect("bind");
        bind_contextual_orchestrator(
            &verify,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        )
        .expect("digit-only canonical digest");
        assert_eq!(binding.contract_version(), ORCHESTRATION_CONTRACT_VERSION);
        assert_eq!(binding.roles().len(), 2);
        assert_eq!(binding.token_budget(), 32_000);
        assert!(binding.access_list().is_empty());
        assert!(!binding.includes_credentials());
        assert_eq!(binding.mode(), OrchestrationMode::Verify);
        assert_eq!(binding.policy_version(), ORCHESTRATION_POLICY_VERSION);
        assert_eq!(
            binding.evidence_manifest_hash(),
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(
            bind_contextual_orchestrator(&verify, "raw evidence"),
            Err(ApiError::InvalidWirePayload),
        );
        assert_eq!(
            bind_contextual_orchestrator(&verify, "sha256:a"),
            Err(ApiError::InvalidWirePayload),
        );
        assert_eq!(
            bind_contextual_orchestrator(
                &verify,
                "sha256:gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
            ),
            Err(ApiError::InvalidWirePayload),
        );
        let abstain = route_orchestration(&request(
            InterpretationTaskKind::SpanClassification,
            0.1,
            0.1,
            0.1,
            8_000,
        ))
        .expect("abstain");
        assert_eq!(
            bind_contextual_orchestrator(
                &abstain,
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
            Err(ApiError::AuthorizationDenied),
        );

        let conductor = route_orchestration(&request(
            InterpretationTaskKind::NarrativeSynthesis,
            0.8,
            0.8,
            1.0,
            32_000,
        ))
        .expect("conductor");
        let thinker = conductor
            .roles()
            .iter()
            .find(|assignment| assignment.role() == OrchestrationRole::Thinker)
            .expect("routed thinker");
        assert_eq!(thinker.role().wire_name(), "thinker");
        assert_eq!(thinker.effort(), ReasoningEffort::Medium);
        assert_eq!(thinker.effort().wire_name(), "medium");
        assert_eq!(
            direct.to_string(),
            "orchestration mode=direct stages=1 recursion=0 proposal=true"
        );
    }

    #[test]
    fn preferred_modes_cover_concept_and_narrative_boundaries() {
        let cases = [
            (
                InterpretationTaskKind::SpanClassification,
                0.10,
                0.40,
                OrchestrationMode::Verify,
            ),
            (
                InterpretationTaskKind::ConceptAlignment,
                0.1,
                0.50,
                OrchestrationMode::Committee,
            ),
            (
                InterpretationTaskKind::ConceptAlignment,
                0.1,
                0.49,
                OrchestrationMode::Verify,
            ),
            (
                InterpretationTaskKind::NarrativeSynthesis,
                0.1,
                0.50,
                OrchestrationMode::Conductor,
            ),
            (
                InterpretationTaskKind::NarrativeSynthesis,
                0.1,
                0.49,
                OrchestrationMode::Verify,
            ),
        ];

        for (task_kind, risk_score, ambiguity_score, expected_mode) in cases {
            let plan = route_orchestration(&request(
                task_kind,
                risk_score,
                ambiguity_score,
                1.0,
                32_000,
            ))
            .expect("boundary route");
            assert_eq!(plan.mode(), expected_mode);
        }
    }

    #[test]
    fn empty_policy_and_negative_infinity_fail_closed() {
        let mut empty = request(
            InterpretationTaskKind::SpanClassification,
            0.1,
            0.1,
            0.9,
            8_000,
        );
        empty.policy_version.clear();
        assert_eq!(
            route_orchestration(&empty),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            route_orchestration(&request(
                InterpretationTaskKind::SpanClassification,
                f64::NEG_INFINITY,
                0.1,
                0.9,
                8_000,
            )),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
