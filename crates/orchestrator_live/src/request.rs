//! Versioned interpretation-run request and accepted-response contracts.

use serde::{Deserialize, Serialize};

use crate::error::OrchestratorLiveError;
use crate::mode::OrchestrationMode;

/// Supported interpretation-run contract version.
pub const INTERPRETATION_RUN_CONTRACT_VERSION: u16 = 1;

/// Versioned path contextual-orchestrator may POST or GET.
pub const INTERPRETATION_RUN_PATH: &str = "/v1/interpretation-runs";

/// Default maximum interpretation-run JSON payload size in bytes.
pub const DEFAULT_INTERPRETATION_BYTE_LIMIT: usize = 64 * 1024;

/// Canonical claim status for every accepted interpretation.
pub const HYPOTHETICAL_CLAIM_STATUS: &str = "hypothetical";

const MAX_BUDGET_TOKENS: u32 = 1_000_000;
const MAX_EVIDENCE_SPANS: usize = 32;

/// Request to create a hypothetical interpretation run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterpretationRunRequest {
    contract_version: u16,
    idempotency_key: String,
    tenant_workspace_id: String,
    snapshot_id: String,
    knowledge_cutoff: String,
    orchestration_mode: OrchestrationMode,
    compute_budget_tokens: u32,
    evidence_span_ids: Vec<String>,
    #[serde(default)]
    scientific_authority: bool,
}

/// Server-accepted interpretation run that cannot carry scientific authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterpretationRunAccepted {
    contract_version: u16,
    interpretation_run_id: String,
    orchestration_mode: OrchestrationMode,
    claim_status: String,
    scientific_authority: bool,
    idempotency_key: String,
}

impl InterpretationRunRequest {
    /// Construct and validate an interpretation-run request.
    ///
    /// # Errors
    ///
    /// Returns wire, version, budget, or scientific-authority errors.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        contract_version: u16,
        idempotency_key: impl Into<String>,
        tenant_workspace_id: impl Into<String>,
        snapshot_id: impl Into<String>,
        knowledge_cutoff: impl Into<String>,
        orchestration_mode: OrchestrationMode,
        compute_budget_tokens: u32,
        evidence_span_ids: Vec<String>,
        scientific_authority: bool,
    ) -> Result<Self, OrchestratorLiveError> {
        let request = Self {
            contract_version,
            idempotency_key: idempotency_key.into(),
            tenant_workspace_id: tenant_workspace_id.into(),
            snapshot_id: snapshot_id.into(),
            knowledge_cutoff: knowledge_cutoff.into(),
            orchestration_mode,
            compute_budget_tokens,
            evidence_span_ids,
            scientific_authority,
        };
        request.validate()?;
        Ok(request)
    }

    /// Parse and validate a JSON interpretation-run request.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, or scientific-authority errors.
    pub fn from_json(payload: &str) -> Result<Self, OrchestratorLiveError> {
        require_byte_limit(payload, DEFAULT_INTERPRETATION_BYTE_LIMIT)?;
        let request: Self = from_json(payload)?;
        request.validate()?;
        Ok(request)
    }

    /// Serialize this request to JSON after validation.
    ///
    /// # Errors
    ///
    /// Returns field-validation or serialization errors.
    pub fn to_json(&self) -> Result<String, OrchestratorLiveError> {
        self.validate()?;
        to_json(self)
    }

    /// Opaque client idempotency key.
    #[must_use]
    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    /// Opaque tenant or workspace identity.
    #[must_use]
    pub fn tenant_workspace_id(&self) -> &str {
        &self.tenant_workspace_id
    }

    /// Immutable evidence snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Knowledge-cutoff instant as an RFC 3339 string.
    #[must_use]
    pub fn knowledge_cutoff(&self) -> &str {
        &self.knowledge_cutoff
    }

    /// Selected orchestration mode.
    #[must_use]
    pub const fn orchestration_mode(&self) -> OrchestrationMode {
        self.orchestration_mode
    }

    fn validate(&self) -> Result<(), OrchestratorLiveError> {
        require_contract_version(self.contract_version, INTERPRETATION_RUN_CONTRACT_VERSION)?;
        require_nonempty(&self.idempotency_key)?;
        require_nonempty(&self.tenant_workspace_id)?;
        require_nonempty(&self.snapshot_id)?;
        require_nonempty(&self.knowledge_cutoff)?;
        refuse_table_label(&self.snapshot_id)?;
        if self.compute_budget_tokens == 0 || self.compute_budget_tokens > MAX_BUDGET_TOKENS {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        if self.evidence_span_ids.is_empty() || self.evidence_span_ids.len() > MAX_EVIDENCE_SPANS {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        for span_id in &self.evidence_span_ids {
            require_nonempty(span_id)?;
            refuse_table_label(span_id)?;
        }
        if self.scientific_authority {
            return Err(OrchestratorLiveError::ScientificAuthorityRefused);
        }
        Ok(())
    }
}

impl InterpretationRunAccepted {
    /// Construct an accepted interpretation that cannot be scientific authority.
    ///
    /// # Errors
    ///
    /// Returns wire or scientific-authority errors.
    pub fn new(
        interpretation_run_id: impl Into<String>,
        orchestration_mode: OrchestrationMode,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, OrchestratorLiveError> {
        let accepted = Self {
            contract_version: INTERPRETATION_RUN_CONTRACT_VERSION,
            interpretation_run_id: interpretation_run_id.into(),
            orchestration_mode,
            claim_status: HYPOTHETICAL_CLAIM_STATUS.to_owned(),
            scientific_authority: false,
            idempotency_key: idempotency_key.into(),
        };
        accepted.validate()?;
        Ok(accepted)
    }

    /// Parse and validate an accepted interpretation JSON payload.
    ///
    /// # Errors
    ///
    /// Returns wire or scientific-authority errors.
    pub fn from_json(payload: &str) -> Result<Self, OrchestratorLiveError> {
        require_byte_limit(payload, DEFAULT_INTERPRETATION_BYTE_LIMIT)?;
        let accepted: Self = from_json(payload)?;
        accepted.validate()?;
        Ok(accepted)
    }

    /// Serialize this accepted response after validation.
    ///
    /// # Errors
    ///
    /// Returns field-validation or serialization errors.
    pub fn to_json(&self) -> Result<String, OrchestratorLiveError> {
        self.validate()?;
        to_json(self)
    }

    /// Server-assigned opaque interpretation-run identity.
    #[must_use]
    pub fn interpretation_run_id(&self) -> &str {
        &self.interpretation_run_id
    }

    /// Echo of the validated idempotency key.
    #[must_use]
    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    /// Selected orchestration mode.
    #[must_use]
    pub const fn orchestration_mode(&self) -> OrchestrationMode {
        self.orchestration_mode
    }

    /// Always [`HYPOTHETICAL_CLAIM_STATUS`].
    #[must_use]
    pub fn claim_status(&self) -> &str {
        &self.claim_status
    }

    /// Always `false`; LLM output is never scientific authority.
    #[must_use]
    pub const fn scientific_authority(&self) -> bool {
        self.scientific_authority
    }

    /// Build an accepted response from a request that already passed validation.
    pub(crate) fn from_validated_request(
        interpretation_run_id: String,
        request: &InterpretationRunRequest,
    ) -> Self {
        Self {
            contract_version: INTERPRETATION_RUN_CONTRACT_VERSION,
            interpretation_run_id,
            orchestration_mode: request.orchestration_mode(),
            claim_status: HYPOTHETICAL_CLAIM_STATUS.to_owned(),
            scientific_authority: false,
            idempotency_key: request.idempotency_key().to_owned(),
        }
    }

    fn validate(&self) -> Result<(), OrchestratorLiveError> {
        require_contract_version(self.contract_version, INTERPRETATION_RUN_CONTRACT_VERSION)?;
        require_nonempty(&self.interpretation_run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.claim_status != HYPOTHETICAL_CLAIM_STATUS || self.scientific_authority {
            return Err(OrchestratorLiveError::ScientificAuthorityRefused);
        }
        Ok(())
    }
}

pub(crate) fn to_json<T: Serialize>(value: &T) -> Result<String, OrchestratorLiveError> {
    serde_json::to_string(value).map_err(|_| OrchestratorLiveError::InvalidWirePayload)
}

pub(crate) fn from_json<'de, T: Deserialize<'de>>(
    payload: &'de str,
) -> Result<T, OrchestratorLiveError> {
    serde_json::from_str(payload).map_err(|_| OrchestratorLiveError::InvalidWirePayload)
}

pub(crate) fn require_nonempty(value: &str) -> Result<(), OrchestratorLiveError> {
    if value.trim().is_empty() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok(())
}

pub(crate) fn require_byte_limit(
    payload: &str,
    maximum_bytes: usize,
) -> Result<(), OrchestratorLiveError> {
    if payload.len() > maximum_bytes {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    Ok(())
}

pub(crate) fn require_contract_version(
    version: u16,
    expected: u16,
) -> Result<(), OrchestratorLiveError> {
    if version == expected {
        Ok(())
    } else {
        Err(OrchestratorLiveError::UnsupportedContractVersion)
    }
}

pub(crate) fn refuse_table_label(value: &str) -> Result<(), OrchestratorLiveError> {
    if host_implies_table_access(value) {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok(())
}

pub(crate) fn host_implies_table_access(host: &str) -> bool {
    let lowered = host.to_ascii_lowercase();
    lowered.contains("postgres")
        || lowered.contains("jdbc")
        || lowered.contains("/sql")
        || lowered.contains("/tables/")
        || lowered.contains('\'')
        || lowered.contains(';')
        || lowered.contains('\\')
        || lowered.contains(' ')
        || lowered.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::{
        InterpretationRunAccepted, InterpretationRunRequest, from_json, host_implies_table_access,
        require_byte_limit, require_contract_version, require_nonempty, to_json,
    };
    use crate::error::OrchestratorLiveError;
    use crate::mode::OrchestrationMode;
    use serde::Serialize;
    use serde::ser::Serializer;

    struct SerializationFailure;

    impl Serialize for SerializationFailure {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("intentional"))
        }
    }

    fn valid_request() -> InterpretationRunRequest {
        InterpretationRunRequest::new(
            1,
            "idem-1",
            "tenant-1",
            "snapshot-1",
            "2026-08-01T00:00:00Z",
            OrchestrationMode::Abstain,
            16,
            vec!["span-a".into()],
            false,
        )
        .expect("valid")
    }

    fn refuse_request(
        idempotency_key: &str,
        snapshot_id: &str,
        budget: u32,
        spans: Vec<String>,
    ) -> OrchestratorLiveError {
        InterpretationRunRequest::new(
            1,
            idempotency_key,
            "t",
            snapshot_id,
            "k",
            OrchestrationMode::Direct,
            budget,
            spans,
            false,
        )
        .expect_err("refused")
    }

    #[test]
    fn request_validation_covers_empty_table_and_budget_failures() {
        assert_eq!(
            refuse_request(" ", "s", 1, vec!["span".into()]),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            refuse_request("i", "postgres://db", 1, vec!["span".into()]),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            refuse_request("i", "s", 0, vec!["span".into()]),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            refuse_request("i", "s", 1, vec![]),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            refuse_request("i", "s", 1_000_001, vec!["span".into()]),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            refuse_request("i", "s", 1, vec![" ".into()]),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            refuse_request("i", "s", 1, vec!["span".into(); 33]),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            refuse_request("i", "s", 1, vec!["postgres-span".into()]),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunRequest::new(
                1,
                "i",
                " ",
                "s",
                "k",
                OrchestrationMode::Direct,
                1,
                vec!["span".into()],
                false
            )
            .expect_err("tenant"),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunRequest::new(
                1,
                "i",
                "t",
                "s",
                " ",
                OrchestrationMode::Direct,
                1,
                vec!["span".into()],
                false
            )
            .expect_err("cutoff"),
            OrchestratorLiveError::InvalidWirePayload
        );
    }

    #[test]
    fn request_constructor_rejects_unsupported_version() {
        assert_eq!(
            InterpretationRunRequest::new(
                0,
                "i",
                "t",
                "s",
                "k",
                OrchestrationMode::Direct,
                1,
                vec!["span".into()],
                false,
            )
            .expect_err("version"),
            OrchestratorLiveError::UnsupportedContractVersion
        );
    }

    #[test]
    fn accepted_and_wire_helpers_cover_failure_arms() {
        assert_eq!(
            InterpretationRunAccepted::from_json(
                r#"{"contract_version":1,"interpretation_run_id":"r","orchestration_mode":"direct","claim_status":"hypothetical","scientific_authority":true,"idempotency_key":"i"}"#
            )
            .expect_err("sci"),
            OrchestratorLiveError::ScientificAuthorityRefused
        );
        assert_eq!(
            InterpretationRunRequest::from_json(
                &"x".repeat(super::DEFAULT_INTERPRETATION_BYTE_LIMIT + 1)
            )
            .expect_err("limit"),
            OrchestratorLiveError::LimitExceeded
        );
        assert_eq!(
            InterpretationRunRequest::from_json("not-json").expect_err("json"),
            OrchestratorLiveError::InvalidWirePayload
        );
        let accepted =
            InterpretationRunAccepted::new("run-1", OrchestrationMode::Conductor, "idem-1")
                .expect("accepted");
        assert_eq!(accepted.orchestration_mode(), OrchestrationMode::Conductor);
        let derived =
            InterpretationRunAccepted::from_validated_request("run-2".into(), &valid_request());
        assert_eq!(derived.interpretation_run_id(), "run-2");
        assert_eq!(valid_request().snapshot_id(), "snapshot-1");
        let invalid = InterpretationRunRequest {
            contract_version: 1,
            idempotency_key: String::new(),
            tenant_workspace_id: "t".into(),
            snapshot_id: "s".into(),
            knowledge_cutoff: "k".into(),
            orchestration_mode: OrchestrationMode::Direct,
            compute_budget_tokens: 1,
            evidence_span_ids: vec!["span".into()],
            scientific_authority: false,
        };
        assert_eq!(
            invalid.to_json().expect_err("to_json"),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunAccepted::new(" ", OrchestrationMode::Direct, "i").expect_err("id"),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            valid_request().orchestration_mode(),
            OrchestrationMode::Abstain
        );
        assert_eq!(
            InterpretationRunAccepted::from_json(
                r#"{"contract_version":1,"interpretation_run_id":"r","orchestration_mode":"direct","claim_status":"accepted","scientific_authority":false,"idempotency_key":"i"}"#
            )
            .expect_err("claim"),
            OrchestratorLiveError::ScientificAuthorityRefused
        );
        assert_eq!(
            to_json(&SerializationFailure),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            from_json::<u8>("["),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        require_nonempty("ok").expect("ok");
        assert_eq!(
            require_nonempty("  "),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        require_byte_limit("ab", 2).expect("ok");
        assert_eq!(
            require_byte_limit("abc", 2),
            Err(OrchestratorLiveError::LimitExceeded)
        );
        require_contract_version(1, 1).expect("ok");
        assert_eq!(
            require_contract_version(2, 1),
            Err(OrchestratorLiveError::UnsupportedContractVersion)
        );
        assert!(host_implies_table_access("db.postgres.local"));
        assert!(host_implies_table_access("jdbc.local"));
        assert!(host_implies_table_access("127.0.0.1/sql"));
        assert!(host_implies_table_access("127.0.0.1/tables/x"));
        assert!(host_implies_table_access("bad host"));
        assert!(host_implies_table_access("bad;host"));
        assert!(host_implies_table_access("bad'host"));
        assert!(host_implies_table_access("bad\\host"));
        assert!(host_implies_table_access("bad\u{0001}host"));
        assert!(!host_implies_table_access("127.0.0.1:43789"));
    }
}
