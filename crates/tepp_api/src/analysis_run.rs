//! Versioned analysis-run request and accepted-run response contracts.

use crate::ApiError;
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use temporal_core::KnowledgeCutoff;

/// Supported analysis-run contract version.
pub const ANALYSIS_RUN_CONTRACT_VERSION: u16 = 1;

/// Default maximum analysis-run JSON payload size in bytes.
pub const DEFAULT_ANALYSIS_RUN_BYTE_LIMIT: usize = 64 * 1024;

/// Request to create a durable analysis run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunRequest {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Client-supplied idempotency key (opaque).
    pub idempotency_key: String,
    /// Authorized tenant or workspace opaque identity.
    pub tenant_workspace_id: String,
    /// Immutable corpus/evidence snapshot identity.
    pub snapshot_id: String,
    /// Knowledge cutoff instant as an ISO-8601 / RFC 3339 string.
    pub knowledge_cutoff: String,
    /// Versioned model/backend contract identity.
    pub model_contract_version: String,
    /// Requested output profile name.
    pub output_profile: String,
}

/// Server-accepted analysis run identity and lifecycle state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunAccepted {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Server-assigned opaque run identity.
    pub run_id: String,
    /// Canonical lifecycle state for the accepted run.
    pub run_state: String,
    /// Echo of the validated idempotency key.
    pub idempotency_key: String,
}

impl AnalysisRunRequest {
    /// Parse and validate a JSON analysis-run request with default size limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate a JSON analysis-run request.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        let request: Self = from_json(payload)?;
        request.validate()?;
        Ok(request)
    }

    /// Serialize this request to JSON after validation.
    ///
    /// # Errors
    ///
    /// Returns field-validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, ANALYSIS_RUN_CONTRACT_VERSION)?;
        require_nonempty(&self.idempotency_key)?;
        require_nonempty(&self.tenant_workspace_id)?;
        require_nonempty(&self.snapshot_id)?;
        require_rfc3339_knowledge_cutoff(&self.knowledge_cutoff)?;
        require_nonempty(&self.model_contract_version)?;
        require_nonempty(&self.output_profile)?;
        Ok(())
    }
}

/// Parse `knowledge_cutoff` as a TEPP clock and refuse a cutoff after now.
///
/// A buyer cannot claim analysis of evidence that is not yet available. The
/// request receipt instant is treated as availability of the command itself.
fn require_rfc3339_knowledge_cutoff(knowledge_cutoff: &str) -> Result<(), ApiError> {
    require_nonempty(knowledge_cutoff)?;
    let cutoff = KnowledgeCutoff::parse_rfc3339(knowledge_cutoff)
        .map_err(|_| ApiError::InvalidWirePayload)?;
    let receipt = KnowledgeCutoff::parse_rfc3339(&Timestamp::now().to_string())
        .map_err(|_| ApiError::InvalidWirePayload)?;
    if cutoff.instant() > receipt.instant() {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

impl AnalysisRunAccepted {
    /// Construct a validated accepted-run response.
    ///
    /// # Errors
    ///
    /// Returns version or field-validation errors.
    pub fn new(
        run_id: impl Into<String>,
        run_state: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let accepted = Self {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            run_id: run_id.into(),
            run_state: run_state.into(),
            idempotency_key: idempotency_key.into(),
        };
        accepted.validate()?;
        Ok(accepted)
    }

    /// Parse a JSON accepted-run payload.
    ///
    /// # Errors
    ///
    /// Returns wire, version, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        let accepted: Self = from_json(payload)?;
        accepted.validate()?;
        Ok(accepted)
    }

    /// Serialize to JSON.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, ANALYSIS_RUN_CONTRACT_VERSION)?;
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.run_state)?;
        require_nonempty(&self.idempotency_key)?;
        Ok(())
    }
}

/// Compare two requests for idempotent-retry semantic equality.
#[must_use]
pub fn requests_are_idempotent_matches(
    left: &AnalysisRunRequest,
    right: &AnalysisRunRequest,
) -> bool {
    left == right
}

#[cfg(test)]
mod tests {
    use super::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunRequest,
        DEFAULT_ANALYSIS_RUN_BYTE_LIMIT, requests_are_idempotent_matches,
    };
    use crate::ApiError;

    fn sample_request() -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "idem-1".into(),
            tenant_workspace_id: "tenant-ws-1".into(),
            snapshot_id: "snap-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "topic-v1".into(),
            output_profile: "validation-report".into(),
        }
    }

    #[test]
    fn analysis_run_contract_rejects_hostile_and_oversized_payloads() {
        let request = sample_request();
        let json = request.to_json().expect("json");
        let decoded = AnalysisRunRequest::from_json(&json).expect("decode");
        assert_eq!(decoded, request);
        assert!(requests_are_idempotent_matches(&request, &decoded));

        let mut other = request.clone();
        other.snapshot_id = "snap-2".into();
        assert!(!requests_are_idempotent_matches(&request, &other));

        assert_eq!(
            AnalysisRunRequest::from_json(
                r#"{"contract_version":1,"idempotency_key":"a","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"k","model_contract_version":"m","output_profile":"o","extra":true}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRequest::from_json(
                r#"{"contract_version":9,"idempotency_key":"a","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"k","model_contract_version":"m","output_profile":"o"}"#
            ),
            Err(ApiError::UnsupportedContractVersion)
        );
        assert_eq!(
            AnalysisRunRequest::from_json(
                r#"{"contract_version":1,"idempotency_key":"","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"k","model_contract_version":"m","output_profile":"o"}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRequest::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert!(DEFAULT_ANALYSIS_RUN_BYTE_LIMIT >= json.len());

        // empty required fields fail validation
        let mut bad = request.clone();
        bad.tenant_workspace_id.clear();
        assert_eq!(bad.to_json(), Err(ApiError::InvalidWirePayload));
        bad = request.clone();
        bad.snapshot_id.clear();
        assert_eq!(bad.to_json(), Err(ApiError::InvalidWirePayload));
        bad = request.clone();
        bad.knowledge_cutoff.clear();
        assert_eq!(bad.to_json(), Err(ApiError::InvalidWirePayload));
        bad = request.clone();
        bad.model_contract_version.clear();
        assert_eq!(bad.to_json(), Err(ApiError::InvalidWirePayload));
        bad = request.clone();
        bad.output_profile.clear();
        assert_eq!(bad.to_json(), Err(ApiError::InvalidWirePayload));

        assert_eq!(
            AnalysisRunRequest::from_json(
                r#"{"contract_version":1,"idempotency_key":"a\u001fb","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"m","output_profile":"o"}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRequest::from_json(
                r#"{"contract_version":1,"idempotency_key":"a","tenant_workspace_id":"t\u001fb","snapshot_id":"s","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"m","output_profile":"o"}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );

        assert_eq!(
            AnalysisRunRequest::from_json(
                r#"{"contract_version":1,"idempotency_key":"a","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"k","model_contract_version":"m","output_profile":"o"}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunRequest::from_json(
                r#"{"contract_version":1,"idempotency_key":"a","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"2099-01-01T00:00:00Z","model_contract_version":"m","output_profile":"o"}"#
            ),
            Err(ApiError::InvalidWirePayload)
        );

        let accepted = AnalysisRunAccepted::new("run-1", "accepted", "idem-1").expect("acc");
        let accepted_json = accepted.to_json().expect("aj");
        assert_eq!(
            AnalysisRunAccepted::from_json(&accepted_json).expect("d"),
            accepted
        );
        assert_eq!(
            AnalysisRunAccepted::new("", "accepted", "idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunAccepted::new("run-1", "", "idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunAccepted::new("run-1", "accepted", ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunAccepted::from_json(
                r#"{"contract_version":2,"run_id":"r","run_state":"accepted","idempotency_key":"i"}"#
            ),
            Err(ApiError::UnsupportedContractVersion)
        );
    }
}
