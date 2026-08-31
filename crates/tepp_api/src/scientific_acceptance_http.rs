//! Loopback HTTP gates for `tepp.scientific_acceptance.v1` status reads.
//!
//! GAP-003A third slice: `POST /v1/analysis-runs` stays a metric-free receipt.
//! `GET /v1/analysis-runs/{run_id}` returns an accepted or running status with
//! no scientific-acceptance object. Only a succeeded status whose request
//! profile is `scientific_acceptance_v1` may carry `tepp.scientific_acceptance.v1`.
//! This module does not define the terminal-result DTO; that wire belongs to
//! the separate GAP-003A API slice. Persistence remains GAP-003B.

use crate::wire::require_nonempty;
use crate::{
    AnalysisRunRequest, AnalysisRunStatus, AnalysisRunStatusState, AnalysisRunTerminalState,
    ApiError,
};
use sha2::{Digest, Sha256};

/// Schema identity returned on a succeeded scientific-acceptance GET.
pub const SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA: &str = "tepp.scientific_acceptance.v1";
/// Output profile that authorizes the scientific-acceptance HTTP attachment.
pub const SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE: &str = "scientific_acceptance_v1";

const FORBIDDEN_RECEIPT_KEYS: [&str; 12] = [
    "rmse",
    "rmse_standard_error",
    "mean_bias",
    "bias_standard_error",
    "interval_coverage",
    "coverage_wilson_lower",
    "coverage_wilson_upper",
    "temporal_order_accuracy",
    "se_gate_accepted",
    "se_gate_k",
    "scientific_acceptance",
    "report",
];

/// Return whether a receipt JSON object carries scientific-metric keys.
///
/// Request and accepted receipts, and accepted/running status bodies, must
/// remain metric-free. Unknown-field denial is the DTO gate; this helper names
/// the forbidden keys for the HTTP boundary.
#[must_use]
pub fn receipt_json_carries_scientific_metrics(payload: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    FORBIDDEN_RECEIPT_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
}

/// Refuse a request, accepted receipt, or non-terminal status that already
/// carries scientific-metric keys.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric key is
/// present on a receipt object.
pub fn refuse_metrics_on_receipt(payload: &str) -> Result<(), ApiError> {
    if receipt_json_carries_scientific_metrics(payload) {
        Err(ApiError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

/// Serialize one loopback status GET body with HTTP-layer scientific-acceptance
/// gates.
///
/// Accepted and running statuses stay metric-free. A failed status cannot carry
/// the artifact. A succeeded status may carry `tepp.scientific_acceptance.v1`
/// only when the request profile is `scientific_acceptance_v1`, the binding
/// digest is a non-zero canonical SHA-256, and the artifact bytes match
/// `result_sha256`.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for profile, state, digest, or
/// metric-key violations, and [`ApiError::LimitExceeded`] is not used here.
pub(crate) fn status_http_json(
    status: &AnalysisRunStatus,
    request: &AnalysisRunRequest,
    artifact_json: Option<&str>,
) -> Result<String, ApiError> {
    request.validate()?;
    if status.idempotency_key != request.idempotency_key {
        return Err(ApiError::InvalidWirePayload);
    }
    match status.run_state {
        AnalysisRunStatusState::Accepted
        | AnalysisRunStatusState::Running
        | AnalysisRunStatusState::Failed => {
            if artifact_json.is_some() {
                return Err(ApiError::InvalidWirePayload);
            }
            let status_json = status.to_json()?;
            refuse_metrics_on_receipt(&status_json)?;
            Ok(status_json)
        }
        AnalysisRunStatusState::Succeeded => match (
            request.output_profile.as_str() == SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
            artifact_json,
            status.terminal_result.as_ref(),
        ) {
            (false, None, Some(_)) => {
                let status_json = status.to_json()?;
                refuse_metrics_on_receipt(&status_json)?;
                Ok(status_json)
            }
            (true, Some(artifact), Some(terminal)) => {
                validate_scientific_acceptance_terminal(terminal, artifact)?;
                let status_json = status.to_json()?;
                refuse_metrics_on_receipt(&status_json)?;
                inject_scientific_acceptance_http(&status_json, artifact)
            }
            _ => Err(ApiError::InvalidWirePayload),
        },
    }
}

fn validate_scientific_acceptance_terminal(
    terminal: &crate::AnalysisRunTerminalResult,
    artifact_json: &str,
) -> Result<(), ApiError> {
    if terminal.run_state != AnalysisRunTerminalState::Succeeded {
        return Err(ApiError::InvalidWirePayload);
    }
    let schema = terminal
        .result_schema_version
        .as_deref()
        .ok_or(ApiError::InvalidWirePayload)?;
    let digest = terminal
        .result_sha256
        .as_deref()
        .ok_or(ApiError::InvalidWirePayload)?;
    if schema != SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA
        || terminal.output_profile != SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE
    {
        return Err(ApiError::InvalidWirePayload);
    }
    require_nonzero_canonical_sha256(digest)?;
    let _artifact = parse_scientific_acceptance_http_artifact(artifact_json)?;
    if sha256_hex(artifact_json.as_bytes()) != digest {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

pub(crate) fn inject_scientific_acceptance_http(
    status_json: &str,
    artifact_json: &str,
) -> Result<String, ApiError> {
    let artifact = parse_scientific_acceptance_http_artifact(artifact_json)?;
    let mut status_value: serde_json::Value =
        serde_json::from_str(status_json).map_err(|_| ApiError::InvalidWirePayload)?;
    let terminal_value = status_value
        .get_mut("terminal_result")
        .ok_or(ApiError::InvalidWirePayload)?;
    let terminal_object = terminal_value
        .as_object_mut()
        .ok_or(ApiError::InvalidWirePayload)?;
    terminal_object.insert("scientific_acceptance".to_owned(), artifact);
    serde_json::to_string(&status_value).map_err(|_| ApiError::InvalidWirePayload)
}

fn parse_scientific_acceptance_http_artifact(
    artifact_json: &str,
) -> Result<serde_json::Value, ApiError> {
    require_nonempty(artifact_json)?;
    let value: serde_json::Value =
        serde_json::from_str(artifact_json).map_err(|_| ApiError::InvalidWirePayload)?;
    let object = value.as_object().ok_or(ApiError::InvalidWirePayload)?;
    let schema = object
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .ok_or(ApiError::InvalidWirePayload)?;
    let profile = object
        .get("output_profile")
        .and_then(serde_json::Value::as_str)
        .ok_or(ApiError::InvalidWirePayload)?;
    let binding = object
        .get("binding_sha256")
        .and_then(serde_json::Value::as_str)
        .ok_or(ApiError::InvalidWirePayload)?;
    if schema != SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA || profile != SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE
    {
        return Err(ApiError::InvalidWirePayload);
    }
    require_nonzero_canonical_sha256(binding)?;
    Ok(value)
}

fn require_nonzero_canonical_sha256(value: &str) -> Result<(), ApiError> {
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid || value.bytes().all(|byte| byte == b'0') {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{
        SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA, encode_hex,
        inject_scientific_acceptance_http, receipt_json_carries_scientific_metrics,
        refuse_metrics_on_receipt, require_nonzero_canonical_sha256, sha256_hex, status_http_json,
    };
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisResultSummary, AnalysisRunAccepted,
        AnalysisRunRequest, AnalysisRunStatus, AnalysisRunStatusState, AnalysisRunTerminalResult,
        AnalysisRunTerminalState, ApiError,
    };

    fn request(profile: &str) -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "idem-http-1".into(),
            tenant_workspace_id: "tenant-http-1".into(),
            snapshot_id: "snapshot-http-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "validation_cpu_f64_v1".into(),
            output_profile: profile.into(),
        }
    }

    fn accepted() -> AnalysisRunAccepted {
        AnalysisRunAccepted::new("tepp-run-1", "accepted", "idem-http-1").expect("accepted")
    }

    fn summary() -> AnalysisResultSummary {
        AnalysisResultSummary::new("scientific_acceptance", 4, 8, "validated").expect("summary")
    }

    fn artifact_json(binding: &str) -> String {
        format!(
            r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}","output_profile":"{SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE}","binding_sha256":"{binding}","run_id":"tepp-run-1"}}"#
        )
    }

    fn succeeded(profile: &str, digest: &str, schema: &str) -> AnalysisRunStatus {
        let request = request(profile);
        let accepted = accepted();
        let terminal = AnalysisRunTerminalResult::succeeded(
            &request,
            &accepted,
            "artifact-http-1",
            digest,
            schema,
            "2026-08-02T03:04:05Z",
            summary(),
        )
        .expect("succeeded");
        AnalysisRunStatus::terminal(&request, &accepted, terminal).expect("status")
    }

    #[test]
    fn receipt_metric_keys_and_nonzero_digests_fail_closed() {
        assert!(!receipt_json_carries_scientific_metrics("{"));
        assert!(!receipt_json_carries_scientific_metrics("[]"));
        assert!(!receipt_json_carries_scientific_metrics("{}"));
        assert!(refuse_metrics_on_receipt(r#"{"run_id":"tepp-run-1"}"#).is_ok());
        for key in [
            "rmse",
            "rmse_standard_error",
            "mean_bias",
            "bias_standard_error",
            "interval_coverage",
            "coverage_wilson_lower",
            "coverage_wilson_upper",
            "temporal_order_accuracy",
            "se_gate_accepted",
            "se_gate_k",
            "scientific_acceptance",
            "report",
        ] {
            let payload = format!(r#"{{"{key}":1}}"#);
            assert!(receipt_json_carries_scientific_metrics(&payload), "{key}");
            assert_eq!(
                refuse_metrics_on_receipt(&payload),
                Err(ApiError::InvalidWirePayload),
                "{key}"
            );
        }
        assert_eq!(
            require_nonzero_canonical_sha256(&"0".repeat(64)),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            require_nonzero_canonical_sha256("not-a-digest"),
            Err(ApiError::InvalidWirePayload)
        );
        assert!(require_nonzero_canonical_sha256(&"ab".repeat(32)).is_ok());
        assert_eq!(encode_hex(&[0x0f, 0xa0]), "0fa0");
        assert_eq!(sha256_hex(b"").len(), 64);
    }

    #[test]
    fn accepted_running_and_failed_status_bodies_stay_metric_free() {
        let request = request("calibrated_event_measurement");
        let accepted = accepted();
        let accepted_status = AnalysisRunStatus::accepted(&accepted).expect("accepted");
        let running_status = AnalysisRunStatus::running(&accepted).expect("running");
        let accepted_json =
            status_http_json(&accepted_status, &request, None).expect("accepted json");
        let running_json = status_http_json(&running_status, &request, None).expect("running json");
        assert!(!accepted_json.contains("scientific_acceptance"));
        assert!(!running_json.contains("rmse"));
        assert_eq!(
            status_http_json(&accepted_status, &request, Some("{}")),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            status_http_json(&running_status, &request, Some("{}")),
            Err(ApiError::InvalidWirePayload)
        );

        let failed = AnalysisRunTerminalResult::failed(
            &request,
            &accepted,
            "2026-08-02T03:04:05Z",
            "estimation_failed",
        )
        .expect("failed");
        let failed_status =
            AnalysisRunStatus::terminal(&request, &accepted, failed).expect("failed status");
        assert!(
            status_http_json(&failed_status, &request, None)
                .expect("failed json")
                .contains("\"failed\"")
        );
        assert_eq!(
            status_http_json(
                &failed_status,
                &request,
                Some(&artifact_json(&"ab".repeat(32)))
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn only_succeeded_scientific_acceptance_profile_may_return_schema() {
        let binding = "ab".repeat(32);
        let artifact = artifact_json(&binding);
        let digest = sha256_hex(artifact.as_bytes());
        let other = succeeded("calibrated_event_measurement", &digest, "tepp-result-v1");
        let other_request = request("calibrated_event_measurement");
        let other_json = status_http_json(&other, &other_request, None).expect("other");
        assert!(!other_json.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
        assert_eq!(
            status_http_json(&other, &other_request, Some(&artifact)),
            Err(ApiError::InvalidWirePayload)
        );

        let profile_request = request(SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE);
        let missing = succeeded(
            SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
            &digest,
            SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
        );
        assert_eq!(
            status_http_json(&missing, &profile_request, None),
            Err(ApiError::InvalidWirePayload)
        );

        let body = status_http_json(&missing, &profile_request, Some(&artifact)).expect("attached");
        assert!(body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
        assert!(body.contains("scientific_acceptance"));
        assert_eq!(
            status_http_json(
                &missing,
                &request("calibrated_event_measurement"),
                Some(&artifact)
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn zero_digest_mismatch_and_hostile_artifacts_fail_closed() {
        let binding = "cd".repeat(32);
        let artifact = artifact_json(&binding);
        let digest = sha256_hex(artifact.as_bytes());
        let profile_request = request(SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE);
        let status = succeeded(
            SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
            &digest,
            SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
        );
        let zero_binding = artifact_json(&"0".repeat(64));
        assert_eq!(
            status_http_json(&status, &profile_request, Some(&zero_binding)),
            Err(ApiError::InvalidWirePayload)
        );
        let mismatched = artifact.replace(&binding, &"ef".repeat(32));
        assert_eq!(
            status_http_json(&status, &profile_request, Some(&mismatched)),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            status_http_json(&status, &profile_request, Some("[]")),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            status_http_json(&status, &profile_request, Some("")),
            Err(ApiError::InvalidWirePayload)
        );
        let wrong_schema = artifact.replace(
            SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
            "tepp.scientific_acceptance.v0",
        );
        assert_eq!(
            status_http_json(&status, &profile_request, Some(&wrong_schema)),
            Err(ApiError::InvalidWirePayload)
        );
        let wrong_profile = artifact.replace(SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, "other_profile");
        assert_eq!(
            status_http_json(&status, &profile_request, Some(&wrong_profile)),
            Err(ApiError::InvalidWirePayload)
        );
        let missing_binding = r#"{"schema_version":"tepp.scientific_acceptance.v1","output_profile":"scientific_acceptance_v1"}"#;
        assert_eq!(
            status_http_json(&status, &profile_request, Some(missing_binding)),
            Err(ApiError::InvalidWirePayload)
        );
        let numeric_schema = r#"{"schema_version":1,"output_profile":"scientific_acceptance_v1","binding_sha256":"abababababababababababababababababababababababababababababababab"}"#;
        assert_eq!(
            status_http_json(&status, &profile_request, Some(numeric_schema)),
            Err(ApiError::InvalidWirePayload)
        );
        let zero_digest_status = succeeded(
            SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
            &"0".repeat(64),
            SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
        );
        assert_eq!(
            status_http_json(&zero_digest_status, &profile_request, Some(&artifact)),
            Err(ApiError::InvalidWirePayload)
        );
        let wrong_schema_status = succeeded(
            SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
            &digest,
            "tepp-result-v1",
        );
        assert_eq!(
            status_http_json(&wrong_schema_status, &profile_request, Some(&artifact)),
            Err(ApiError::InvalidWirePayload)
        );
        let mut mismatched_request = profile_request.clone();
        mismatched_request.idempotency_key = "other-key".into();
        assert_eq!(
            status_http_json(&status, &mismatched_request, Some(&artifact)),
            Err(ApiError::InvalidWirePayload)
        );

        let mut missing_terminal = status.clone();
        missing_terminal.terminal_result = None;
        assert_eq!(
            status_http_json(&missing_terminal, &profile_request, Some(&artifact)),
            Err(ApiError::InvalidWirePayload)
        );
        let mut failed_terminal = status.clone();
        failed_terminal
            .terminal_result
            .as_mut()
            .expect("terminal")
            .run_state = AnalysisRunTerminalState::Failed;
        assert_eq!(
            status_http_json(&failed_terminal, &profile_request, Some(&artifact)),
            Err(ApiError::InvalidWirePayload)
        );
        let mut missing_digest = status.clone();
        missing_digest
            .terminal_result
            .as_mut()
            .expect("terminal")
            .result_sha256 = None;
        assert_eq!(
            status_http_json(&missing_digest, &profile_request, Some(&artifact)),
            Err(ApiError::InvalidWirePayload)
        );
        let mut missing_schema = status.clone();
        missing_schema
            .terminal_result
            .as_mut()
            .expect("terminal")
            .result_schema_version = None;
        assert_eq!(
            status_http_json(&missing_schema, &profile_request, Some(&artifact)),
            Err(ApiError::InvalidWirePayload)
        );
        let mut other_profile_terminal = status.clone();
        other_profile_terminal
            .terminal_result
            .as_mut()
            .expect("terminal")
            .output_profile = "calibrated_event_measurement".into();
        assert_eq!(
            status_http_json(&other_profile_terminal, &profile_request, Some(&artifact)),
            Err(ApiError::InvalidWirePayload)
        );
        let mut succeeded_without_terminal =
            AnalysisRunStatus::accepted(&accepted()).expect("base");
        succeeded_without_terminal.run_state = AnalysisRunStatusState::Succeeded;
        assert_eq!(
            status_http_json(
                &succeeded_without_terminal,
                &request("calibrated_event_measurement"),
                None,
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            inject_scientific_acceptance_http("not-json", &artifact),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            inject_scientific_acceptance_http("{}", &artifact),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            inject_scientific_acceptance_http(r#"{"terminal_result":null}"#, &artifact),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            inject_scientific_acceptance_http(r#"{"terminal_result":[]}"#, &artifact),
            Err(ApiError::InvalidWirePayload)
        );
        let injected =
            inject_scientific_acceptance_http(&status.to_json().expect("status json"), &artifact)
                .expect("inject");
        assert!(injected.contains("scientific_acceptance"));
    }
}
