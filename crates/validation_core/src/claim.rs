//! Exact-head claim promotion gates for ADR 0014 authorities.

use crate::ValidationError;
use crate::accept_within_standard_errors;
use crate::rmse_standard_error;
use crate::root_mean_square_error;

/// Four claim authorities separated by ADR 0014.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClaimAuthority {
    /// Accepted PRD/ADR design authority.
    DecisionAccepted,
    /// Source integrated on the exact protected head with passing tests.
    ImplementedMain,
    /// Implementation plus claim-specific computed recovery evidence.
    ScientificallySupported,
    /// One exact protected head satisfying every release gate together.
    Released,
}

impl ClaimAuthority {
    /// Stable wire name for this authority.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::DecisionAccepted => "decision_accepted",
            Self::ImplementedMain => "implemented_main",
            Self::ScientificallySupported => "scientifically_supported",
            Self::Released => "released",
        }
    }

    fn required_kinds(self) -> &'static [ClaimEvidenceKind] {
        match self {
            Self::DecisionAccepted => &[],
            Self::ImplementedMain => &[ClaimEvidenceKind::ExactHeadTests],
            Self::ScientificallySupported => &[
                ClaimEvidenceKind::ExactHeadTests,
                ClaimEvidenceKind::ScientificRecovery,
            ],
            Self::Released => &[
                ClaimEvidenceKind::ExactHeadTests,
                ClaimEvidenceKind::ScientificRecovery,
                ClaimEvidenceKind::SecuritySupplyChain,
                ClaimEvidenceKind::QualifyingReview,
                ClaimEvidenceKind::OperationalReadiness,
                ClaimEvidenceKind::SbomProvenance,
            ],
        }
    }
}

/// Kind of evidence offered for a promotion request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ClaimEvidenceKind {
    /// Exact-head unit/integration tests on the candidate commit.
    ExactHeadTests,
    /// Claim-specific recovery or calibration evidence.
    ScientificRecovery,
    /// Security and supply-chain gates on the same head.
    SecuritySupplyChain,
    /// Qualifying independent review, not self-approval.
    QualifyingReview,
    /// Operational readiness on the same head.
    OperationalReadiness,
    /// SBOM and provenance bound to the same head.
    SbomProvenance,
    /// A queued or in-progress check.
    QueuedCheck,
    /// Evidence collected on a predecessor or other commit.
    PredecessorHead,
    /// Model or LLM narrative treated as authority.
    LlmJudgment,
    /// A required test that was skipped or ignored.
    SkippedRequired,
}

impl ClaimEvidenceKind {
    /// Stable wire name for this evidence kind.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ExactHeadTests => "exact_head_tests",
            Self::ScientificRecovery => "scientific_recovery",
            Self::SecuritySupplyChain => "security_supply_chain",
            Self::QualifyingReview => "qualifying_review",
            Self::OperationalReadiness => "operational_readiness",
            Self::SbomProvenance => "sbom_provenance",
            Self::QueuedCheck => "queued_check",
            Self::PredecessorHead => "predecessor_head",
            Self::LlmJudgment => "llm_judgment",
            Self::SkippedRequired => "skipped_required",
        }
    }

    /// Whether this kind may ever promote a claim.
    #[must_use]
    pub const fn is_promotable(self) -> bool {
        !matches!(
            self,
            Self::QueuedCheck | Self::PredecessorHead | Self::LlmJudgment | Self::SkippedRequired
        )
    }
}

/// One evidence item offered for promotion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClaimEvidence {
    kind: ClaimEvidenceKind,
    passed: bool,
}

impl ClaimEvidence {
    /// Construct one evidence item.
    #[must_use]
    pub const fn new(kind: ClaimEvidenceKind, passed: bool) -> Self {
        Self { kind, passed }
    }

    /// Return the evidence kind.
    #[must_use]
    pub const fn kind(self) -> ClaimEvidenceKind {
        self.kind
    }

    /// Return whether the presented evidence is marked passing.
    #[must_use]
    pub const fn passed(self) -> bool {
        self.passed
    }
}

/// A request to promote one claim authority on a candidate head.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromotionRequest<'evidence> {
    target: ClaimAuthority,
    candidate_head: [u8; 20],
    protected_head: [u8; 20],
    evidence: &'evidence [ClaimEvidence],
}

impl<'evidence> PromotionRequest<'evidence> {
    /// Parse commit identities and bind the offered evidence.
    ///
    /// Both heads are validated as exact forty-character hexadecimal Git
    /// commit SHAs at construction, so a request can never bind an
    /// unparseable identity. Evidence truthfulness remains adapter-trust
    /// based: only trusted CI and repository adapters may construct requests,
    /// because [`ClaimEvidence::passed`] flags cannot be independently proven
    /// inside this crate.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when either head is not a
    /// forty-character hexadecimal Git commit SHA.
    pub fn new(
        target: ClaimAuthority,
        candidate_head: &str,
        protected_head: &str,
        evidence: &'evidence [ClaimEvidence],
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            target,
            candidate_head: parse_commit_head(candidate_head)?,
            protected_head: parse_commit_head(protected_head)?,
            evidence,
        })
    }

    /// Requested claim authority.
    #[must_use]
    pub const fn target(self) -> ClaimAuthority {
        self.target
    }

    /// Candidate commit identity.
    #[must_use]
    pub const fn candidate_head(self) -> [u8; 20] {
        self.candidate_head
    }

    /// Protected-main commit identity.
    #[must_use]
    pub const fn protected_head(self) -> [u8; 20] {
        self.protected_head
    }

    /// Offered evidence slice.
    #[must_use]
    pub const fn evidence(self) -> &'evidence [ClaimEvidence] {
        self.evidence
    }
}

/// A claim that passed every required exact-head gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromotedClaim {
    authority: ClaimAuthority,
    bound_head: [u8; 20],
}

impl PromotedClaim {
    /// Bind a promoted authority to one commit identity.
    ///
    /// Crate-internal on purpose: only the validated promotion flows in this
    /// module ([`promote_claim`] and [`promote_scientific_recovery`]) may mint
    /// a promoted claim, so external callers cannot bypass the exact-head
    /// evidence gates by direct construction.
    #[must_use]
    pub(crate) const fn new(authority: ClaimAuthority, bound_head: [u8; 20]) -> Self {
        Self {
            authority,
            bound_head,
        }
    }

    /// Promoted authority.
    #[must_use]
    pub const fn authority(self) -> ClaimAuthority {
        self.authority
    }

    /// Exact commit the promotion is bound to.
    #[must_use]
    pub const fn bound_head(self) -> [u8; 20] {
        self.bound_head
    }
}

/// Parse a forty-character hexadecimal Git commit SHA.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when the value is not exactly
/// forty hexadecimal characters.
pub fn parse_commit_head(value: &str) -> Result<[u8; 20], ValidationError> {
    let bytes = value.as_bytes();
    if bytes.len() != 40 {
        return Err(ValidationError::InvalidInput);
    }
    let mut decoded = [0_u8; 20];
    for (index, pair) in bytes.as_chunks::<2>().0.iter().enumerate() {
        decoded[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Ok(decoded)
}

fn hex_nibble(value: u8) -> Result<u8, ValidationError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(ValidationError::InvalidInput),
    }
}

/// Promote a claim only when exact-head evidence satisfies ADR 0014.
///
/// Design authority may bind a non-protected head. Implementation, scientific,
/// and release authorities require the candidate to equal the protected head
/// and every required gate to be present with at least one passing item and no
/// failing item. Queued, predecessor, skipped-required, and LLM evidence fail
/// closed.
///
/// # Errors
///
/// Returns a claim-specific [`ValidationError`] when heads differ, required
/// evidence is missing, a required evidence kind carries a failing item, or
/// unusable evidence is present.
pub fn promote_claim(request: &PromotionRequest<'_>) -> Result<PromotedClaim, ValidationError> {
    for item in request.evidence {
        match item.kind {
            ClaimEvidenceKind::QueuedCheck => {
                return Err(ValidationError::ClaimQueuedEvidence);
            }
            ClaimEvidenceKind::PredecessorHead => {
                return Err(ValidationError::ClaimPredecessorHead);
            }
            ClaimEvidenceKind::LlmJudgment => {
                return Err(ValidationError::ClaimLlmJudgment);
            }
            ClaimEvidenceKind::SkippedRequired => {
                return Err(ValidationError::ClaimSkippedRequired);
            }
            ClaimEvidenceKind::ExactHeadTests
            | ClaimEvidenceKind::ScientificRecovery
            | ClaimEvidenceKind::SecuritySupplyChain
            | ClaimEvidenceKind::QualifyingReview
            | ClaimEvidenceKind::OperationalReadiness
            | ClaimEvidenceKind::SbomProvenance => {}
        }
    }
    if request.target != ClaimAuthority::DecisionAccepted
        && request.candidate_head != request.protected_head
    {
        return Err(ValidationError::ClaimHeadMismatch);
    }
    for required in request.target.required_kinds() {
        let mut any_passed = false;
        for item in request.evidence {
            if item.kind != *required {
                continue;
            }
            if item.passed {
                any_passed = true;
            } else {
                return Err(ValidationError::ClaimEvidenceFailed);
            }
        }
        if !any_passed {
            return Err(ValidationError::ClaimEvidenceMissing);
        }
    }
    Ok(PromotedClaim::new(request.target, request.candidate_head))
}

/// Promote a scientific claim from computed RMSE, not a hardcoded threshold.
///
/// The candidate must equal the protected head. RMSE is accepted only when it
/// lies within `se_multiplier` standard errors of exact recovery.
///
/// # Errors
///
/// Returns head, input, configuration, or recovery-rejection errors.
pub fn promote_scientific_recovery(
    candidate_head: &str,
    protected_head: &str,
    truth: &[f64],
    recovered: &[f64],
    se_multiplier: f64,
) -> Result<PromotedClaim, ValidationError> {
    let candidate = parse_commit_head(candidate_head)?;
    let protected = parse_commit_head(protected_head)?;
    if candidate != protected {
        return Err(ValidationError::ClaimHeadMismatch);
    }
    let rmse = root_mean_square_error(truth, recovered)?;
    let rmse_se = rmse_standard_error(truth, recovered)?;
    if !accept_within_standard_errors(rmse, 0.0, rmse_se, se_multiplier)? {
        return Err(ValidationError::ClaimRecoveryRejected);
    }
    Ok(PromotedClaim::new(
        ClaimAuthority::ScientificallySupported,
        candidate,
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        ClaimAuthority, ClaimEvidence, ClaimEvidenceKind, PromotionRequest, parse_commit_head,
        promote_claim, promote_scientific_recovery,
    };
    use crate::ValidationError;

    const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn wire_names_and_accessors_cover_every_variant() {
        assert_eq!(
            ClaimAuthority::ImplementedMain.wire_name(),
            "implemented_main"
        );
        assert_eq!(
            ClaimAuthority::ScientificallySupported.wire_name(),
            "scientifically_supported"
        );
        for kind in [
            ClaimEvidenceKind::ExactHeadTests,
            ClaimEvidenceKind::ScientificRecovery,
            ClaimEvidenceKind::SecuritySupplyChain,
            ClaimEvidenceKind::QualifyingReview,
            ClaimEvidenceKind::OperationalReadiness,
            ClaimEvidenceKind::SbomProvenance,
            ClaimEvidenceKind::QueuedCheck,
            ClaimEvidenceKind::PredecessorHead,
            ClaimEvidenceKind::LlmJudgment,
            ClaimEvidenceKind::SkippedRequired,
        ] {
            assert!(!kind.wire_name().is_empty());
            assert_eq!(
                kind.is_promotable(),
                !matches!(
                    kind,
                    ClaimEvidenceKind::QueuedCheck
                        | ClaimEvidenceKind::PredecessorHead
                        | ClaimEvidenceKind::LlmJudgment
                        | ClaimEvidenceKind::SkippedRequired
                )
            );
        }
        let evidence = ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true);
        assert_eq!(evidence.kind(), ClaimEvidenceKind::ExactHeadTests);
        assert!(evidence.passed());
        let evidence_row = [evidence];
        let request =
            PromotionRequest::new(ClaimAuthority::DecisionAccepted, HEAD, HEAD, &evidence_row)
                .expect("request");
        assert_eq!(request.target(), ClaimAuthority::DecisionAccepted);
        assert_eq!(request.candidate_head(), parse_commit_head(HEAD).unwrap());
        assert_eq!(request.protected_head(), parse_commit_head(HEAD).unwrap());
        assert_eq!(request.evidence(), evidence_row.as_slice());
        let promoted = promote_claim(&request).expect("design promotion");
        assert_eq!(promoted.authority(), ClaimAuthority::DecisionAccepted);
        assert_eq!(promoted.bound_head(), parse_commit_head(HEAD).unwrap());
        assert_eq!(
            parse_commit_head("0123456789abcdef0123456789abcdef0123456g"),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            promote_scientific_recovery(HEAD, HEAD, &[1.0, 2.0], &[1.0, 2.0], -1.0),
            Err(ValidationError::InvalidConfiguration)
        );
        let extra = [
            ClaimEvidence::new(ClaimEvidenceKind::SecuritySupplyChain, true),
            ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true),
        ];
        let extra_request =
            PromotionRequest::new(ClaimAuthority::ImplementedMain, HEAD, HEAD, &extra)
                .expect("extra");
        assert_eq!(
            promote_claim(&extra_request).expect("ok").authority(),
            ClaimAuthority::ImplementedMain
        );
    }
}
