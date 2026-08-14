//! contextual-orchestrator HTTP bodies must be bounded JSON without secret-name channels.

use tepp_api::{
    ApiError, MAX_ORCHESTRATOR_BODY_BYTES, MAX_ORCHESTRATOR_IDEMPOTENCY_KEY_BYTES,
    orchestrator_interpretation_exchange,
};

#[test]
fn request_body_must_be_a_bounded_json_object() {
    for invalid in ["not-json", "[]", "null", "\"text\""] {
        assert_eq!(
            orchestrator_interpretation_exchange("orchestrator.example", "idem-1", invalid),
            Err(ApiError::InvalidWirePayload)
        );
    }

    let oversized = format!(r#"{{"payload":"{}"}}"#, "x".repeat(MAX_ORCHESTRATOR_BODY_BYTES));
    assert_eq!(
        orchestrator_interpretation_exchange("orchestrator.example", "idem-1", &oversized),
        Err(ApiError::LimitExceeded)
    );
}

#[test]
fn normalized_nested_secret_names_fail_closed() {
    for body in [
        r#"{"copilot_github_token":"x"}"#,
        r#"{"nested":{"review-agent-github-token":"x"}}"#,
        r#"{"credential_name":"GITHUB_TOKEN"}"#,
        r#"{"credential_name":"copilot github token"}"#,
    ] {
        assert_eq!(
            orchestrator_interpretation_exchange("orchestrator.example", "idem-1", body),
            Err(ApiError::AuthorizationDenied)
        );
    }

    orchestrator_interpretation_exchange(
        "orchestrator.example",
        "idem-1",
        r#"{"credential_name":"NVIDIA_NIM_API_KEY","task":"interpret"}"#,
    )
    .expect("the only allowed model credential name remains admissible");
}

#[test]
fn idempotency_key_is_bounded_and_header_safe() {
    let oversized = "i".repeat(MAX_ORCHESTRATOR_IDEMPOTENCY_KEY_BYTES + 1);
    assert_eq!(
        orchestrator_interpretation_exchange("orchestrator.example", &oversized, "{}"),
        Err(ApiError::LimitExceeded)
    );
    assert_eq!(
        orchestrator_interpretation_exchange(
            "orchestrator.example",
            "idem\r\ninjected: true",
            "{}",
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
