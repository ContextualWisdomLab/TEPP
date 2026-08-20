use tepp_api::{
    ANALYSIS_RESULT_CONTRACT_VERSION, ANALYSIS_RUN_CONTRACT_VERSION, AnalysisResultSummary,
    AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult, AnalysisRunTerminalState,
    ApiError, DEFAULT_ANALYSIS_RESULT_BYTE_LIMIT, require_terminal_binding,
    terminal_result_matches_accepted, terminal_result_matches_request,
};

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "idem-1".into(),
        tenant_workspace_id: "tenant-ws-1".into(),
        snapshot_id: "snapshot-1".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "temporal-model-v1".into(),
        output_profile: "validation-report".into(),
    }
}

fn accepted() -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-1", "accepted", "idem-1").expect("accepted")
}

fn summary() -> AnalysisResultSummary {
    AnalysisResultSummary::new("temporal_topic_measurement", 120, 42, "validated")
        .expect("summary")
}

fn succeeded() -> AnalysisRunTerminalResult {
    AnalysisRunTerminalResult::succeeded(
        &request(),
        &accepted(),
        "artifact-1",
        DIGEST,
        "tepp-result-v1",
        "2026-08-02T03:04:05Z",
        summary(),
    )
    .expect("succeeded")
}

fn failed() -> AnalysisRunTerminalResult {
    AnalysisRunTerminalResult::failed(
        &request(),
        &accepted(),
        "2026-08-02T03:04:05Z",
        "estimation_failed",
    )
    .expect("failed")
}

#[test]
fn terminal_success_and_failure_round_trip_without_receipt_confusion() {
    let success = succeeded();
    assert_eq!(success.run_state, AnalysisRunTerminalState::Succeeded);
    assert!(terminal_result_matches_request(&request(), &success));
    assert!(terminal_result_matches_accepted(&accepted(), &success));
    assert_eq!(
        require_terminal_binding(&request(), &accepted(), &success),
        Ok(())
    );
    let json = success.to_json().expect("json");
    assert!(json.len() <= DEFAULT_ANALYSIS_RESULT_BYTE_LIMIT);
    assert_eq!(
        AnalysisRunTerminalResult::from_json(&json).expect("decoded"),
        success
    );

    let failure = failed();
    assert_eq!(failure.run_state, AnalysisRunTerminalState::Failed);
    assert_eq!(failure.result_artifact_id, None);
    assert_eq!(failure.summary, None);
    let json = failure.to_json().expect("json");
    assert_eq!(
        AnalysisRunTerminalResult::from_json(&json).expect("decoded"),
        failure
    );

    let accepted_json = accepted().to_json().expect("accepted json");
    assert_eq!(
        AnalysisRunTerminalResult::from_json(&accepted_json),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn wire_version_limit_extension_and_time_validation_fail_closed() {
    let mut value: serde_json::Value =
        serde_json::from_str(&succeeded().to_json().expect("json")).expect("value");
    value["extra"] = serde_json::json!(true);
    assert_eq!(
        AnalysisRunTerminalResult::from_json(&value.to_string()),
        Err(ApiError::InvalidWirePayload)
    );

    let json = succeeded().to_json().expect("json");
    assert_eq!(
        AnalysisRunTerminalResult::from_json_with_limit(&json, 8),
        Err(ApiError::LimitExceeded)
    );

    let mut value = succeeded();
    value.contract_version = ANALYSIS_RESULT_CONTRACT_VERSION + 1;
    assert_eq!(
        value.to_json(),
        Err(ApiError::UnsupportedContractVersion)
    );

    let mut value = succeeded();
    value.knowledge_cutoff = "yesterday".into();
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    let mut value = succeeded();
    value.completed_at = "2026-99-99T25:00:00Z".into();
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));
}

#[test]
fn every_required_binding_field_is_nonempty() {
    for index in 0..8 {
        let mut value = succeeded();
        match index {
            0 => value.run_id.clear(),
            1 => value.idempotency_key.clear(),
            2 => value.tenant_workspace_id.clear(),
            3 => value.snapshot_id.clear(),
            4 => value.knowledge_cutoff.clear(),
            5 => value.model_contract_version.clear(),
            6 => value.output_profile.clear(),
            7 => value.completed_at.clear(),
            _ => unreachable!(),
        }
        assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));
    }
}

#[test]
fn succeeded_shape_requires_complete_digest_bound_result() {
    let mut value = succeeded();
    value.result_artifact_id = None;
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    let mut value = succeeded();
    value.result_artifact_id = Some(String::new());
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    let mut value = succeeded();
    value.result_sha256 = None;
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    for digest in [
        String::new(),
        "abcd".into(),
        DIGEST.to_uppercase(),
        format!("{DIGEST}0"),
        format!("g{}", &DIGEST[1..]),
    ] {
        let mut value = succeeded();
        value.result_sha256 = Some(digest);
        assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));
    }

    let mut value = succeeded();
    value.result_schema_version = None;
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    let mut value = succeeded();
    value.result_schema_version = Some(String::new());
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    let mut value = succeeded();
    value.summary = None;
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    let mut value = succeeded();
    value.failure_code = Some("unexpected_failure".into());
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));
}

#[test]
fn summary_is_nonempty_and_bounded_in_constructor_and_wire_shape() {
    assert_eq!(
        AnalysisResultSummary::new("", 0, 0, "validated"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisResultSummary::new("family", 0, 0, ""),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisResultSummary::new("family", 1_000_000_001, 0, "validated"),
        Err(ApiError::LimitExceeded)
    );
    assert_eq!(
        AnalysisResultSummary::new("family", 0, 1_000_000_001, "validated"),
        Err(ApiError::LimitExceeded)
    );

    let invalid_summaries = [
        AnalysisResultSummary {
            analysis_family: String::new(),
            evidence_count: 0,
            statistic_count: 0,
            validation_status: "validated".into(),
        },
        AnalysisResultSummary {
            analysis_family: "family".into(),
            evidence_count: 0,
            statistic_count: 0,
            validation_status: String::new(),
        },
        AnalysisResultSummary {
            analysis_family: "family".into(),
            evidence_count: 1_000_000_001,
            statistic_count: 0,
            validation_status: "validated".into(),
        },
        AnalysisResultSummary {
            analysis_family: "family".into(),
            evidence_count: 0,
            statistic_count: 1_000_000_001,
            validation_status: "validated".into(),
        },
    ];
    for invalid_summary in invalid_summaries {
        let mut value = succeeded();
        value.summary = Some(invalid_summary);
        assert!(matches!(
            value.to_json(),
            Err(ApiError::InvalidWirePayload | ApiError::LimitExceeded)
        ));
    }
}

#[test]
fn failed_shape_refuses_measurement_fields_and_invalid_failure_codes() {
    let base = failed();

    let mut value = base.clone();
    value.result_artifact_id = Some("artifact".into());
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    let mut value = base.clone();
    value.result_sha256 = Some(DIGEST.into());
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    let mut value = base.clone();
    value.result_schema_version = Some("schema".into());
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    let mut value = base.clone();
    value.summary = Some(summary());
    assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));

    for code in [
        None,
        Some(String::new()),
        Some("UPPER_CASE".into()),
        Some("_leading".into()),
        Some("contains-hyphen".into()),
        Some("x".repeat(65)),
    ] {
        let mut value = base.clone();
        value.failure_code = code;
        assert_eq!(value.to_json(), Err(ApiError::InvalidWirePayload));
    }
}

#[test]
fn every_request_binding_dimension_and_receipt_identity_is_checked() {
    let result = succeeded();

    for index in 0..6 {
        let mut mismatched = request();
        match index {
            0 => mismatched.idempotency_key = "other".into(),
            1 => mismatched.tenant_workspace_id = "other".into(),
            2 => mismatched.snapshot_id = "other".into(),
            3 => mismatched.knowledge_cutoff = "2026-07-31T00:00:00Z".into(),
            4 => mismatched.model_contract_version = "other".into(),
            5 => mismatched.output_profile = "other".into(),
            _ => unreachable!(),
        }
        assert!(!terminal_result_matches_request(&mismatched, &result));
        assert_eq!(
            require_terminal_binding(&mismatched, &accepted(), &result),
            Err(ApiError::InvalidWirePayload)
        );
    }

    let other_run =
        AnalysisRunAccepted::new("other-run", "accepted", "idem-1").expect("accepted");
    assert!(!terminal_result_matches_accepted(&other_run, &result));
    assert_eq!(
        require_terminal_binding(&request(), &other_run, &result),
        Err(ApiError::InvalidWirePayload)
    );

    let other_key =
        AnalysisRunAccepted::new("run-1", "accepted", "other-key").expect("accepted");
    assert!(!terminal_result_matches_accepted(&other_key, &result));
    assert_eq!(
        require_terminal_binding(&request(), &other_key, &result),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunTerminalResult::succeeded(
            &request(),
            &other_key,
            "artifact",
            DIGEST,
            "schema",
            "2026-08-02T03:04:05Z",
            summary(),
        ),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunTerminalResult::failed(
            &request(),
            &other_key,
            "2026-08-02T03:04:05Z",
            "provider_timeout",
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
