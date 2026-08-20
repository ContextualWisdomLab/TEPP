"""Close PR 159 project-history production line and branch coverage gaps."""

from __future__ import annotations

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    """Replace one exact source anchor or accept an already-applied edit."""
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    if new in text:
        return
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one anchor, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def append_once(path: str, marker: str, addition: str) -> None:
    """Append a Rust test module once."""
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    if marker in text:
        return
    target.write_text(f"{text.rstrip()}\n\n{addition.rstrip()}\n", encoding="utf-8")


def main() -> None:
    """Remove invariant-only error arms and add exhaustive contract tests."""
    source = "crates/tepp_api/src/project_history.rs"
    live_source = "crates/tepp_api/src/analysis_run_live.rs"

    replace_once(
        source,
        """    let focus_index = ordered
        .iter()
        .position(|event| event.event_id == request.focus_event_id)
        .ok_or(ApiError::InvalidWirePayload)?;
""",
        """    let focus_index = ordered
        .iter()
        .position(|event| event.event_id == request.focus_event_id)
        .expect("validated project-history request contains its focus event");
""",
    )
    replace_once(
        source,
        """    let history_span_start = ordered
        .first()
        .map(|event| event.event_time.clone())
        .ok_or(ApiError::InvalidWirePayload)?;
    let history_span_end = ordered
        .last()
        .map(|event| event.event_time.clone())
        .ok_or(ApiError::InvalidWirePayload)?;
""",
        """    let history_span_start = ordered
        .first()
        .expect("validated project-history request is non-empty")
        .event_time
        .clone();
    let history_span_end = ordered
        .last()
        .expect("validated project-history request is non-empty")
        .event_time
        .clone();
""",
    )

    append_once(
        source,
        "mod project_history_exhaustive_tests",
        r'''#[cfg(test)]
mod project_history_exhaustive_tests {
    use super::*;

    fn event(
        event_id: &str,
        event_type_code: &str,
        event_time: &str,
    ) -> ProjectHistoryEvent {
        ProjectHistoryEvent {
            event_id: event_id.into(),
            event_type_code: event_type_code.into(),
            event_title: format!("title {event_id}"),
            event_time: event_time.into(),
            available_at: "2026-08-19T12:00:00Z".into(),
            availability_basis: "source_post.created_at".into(),
            source_post_id: format!("post-{event_id}"),
            evidence_text: format!("evidence {event_id}"),
            actor_ids: vec![format!("actor-{event_id}")],
        }
    }

    fn request() -> ProjectHistoryRequest {
        ProjectHistoryRequest {
            contract_version: PROJECT_HISTORY_CONTRACT_VERSION,
            idempotency_key: "idem-exhaustive".into(),
            tenant_workspace_id: "tenant-exhaustive".into(),
            project_key: "project-exhaustive".into(),
            project_name: "Project exhaustive".into(),
            knowledge_cutoff: "2026-08-19T23:59:59Z".into(),
            focus_event_id: "focus".into(),
            events: vec![
                event("rebid", "rebid_started", "2026-08-19T18:00:00Z"),
                event("award", "contract_awarded", "2022-03-01T00:00:00Z"),
                event(
                    "specification",
                    "specification_changed",
                    "2023-06-01T00:00:00Z",
                ),
                event("delivery", "delivered", "2024-01-01T00:00:00Z"),
                event(
                    "handoff",
                    "operational_handoff",
                    "2024-02-01T00:00:00Z",
                ),
                event("focus", "voc_received", "2026-08-19T17:00:00Z"),
            ],
        }
    }

    #[test]
    fn request_json_limits_and_identity_guards_are_exhaustive() {
        let request = request();
        let json = request.to_json().expect("valid request json");
        assert_eq!(
            ProjectHistoryRequest::from_json(&json).expect("valid request"),
            request
        );
        assert_eq!(
            ProjectHistoryRequest::from_json_with_limit(&json, json.len()),
            Ok(request.clone())
        );
        assert_eq!(
            ProjectHistoryRequest::from_json_with_limit(&json, json.len() - 1),
            Err(ApiError::LimitExceeded)
        );

        let mut invalid = request.clone();
        invalid.contract_version += 1;
        assert_eq!(
            invalid.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );

        invalid = request.clone();
        invalid.idempotency_key.clear();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));

        invalid = request.clone();
        invalid.idempotency_key = "x".repeat(257);
        assert_eq!(invalid.to_json(), Err(ApiError::LimitExceeded));

        invalid = request.clone();
        invalid.events.clear();
        assert_eq!(invalid.to_json(), Err(ApiError::LimitExceeded));

        invalid = request.clone();
        invalid.events = vec![
            event("many", "event_observed", "2026-08-19T12:00:00Z");
            DEFAULT_PROJECT_HISTORY_EVENT_LIMIT + 1
        ];
        assert_eq!(invalid.to_json(), Err(ApiError::LimitExceeded));

        invalid = request.clone();
        invalid.knowledge_cutoff = "2999-01-01T00:00:00Z".into();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));

        invalid = request.clone();
        invalid.knowledge_cutoff = "not-a-time".into();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));

        invalid = request.clone();
        invalid.focus_event_id = "missing".into();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));

        invalid = request.clone();
        invalid.events[1].event_id = invalid.events[0].event_id.clone();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));
    }

    #[test]
    fn event_fields_actor_bounds_and_availability_are_exhaustive() {
        let request = request();
        let cutoff = parse_timestamp(&request.knowledge_cutoff).expect("cutoff");
        let base = request.events[0].clone();
        assert_eq!(validate_event(&base, &cutoff), Ok(()));

        let mut invalid = base.clone();
        invalid.event_type_code = "Event-Observed".into();
        assert_eq!(
            validate_event(&invalid, &cutoff),
            Err(ApiError::InvalidWirePayload)
        );

        invalid = base.clone();
        invalid.availability_basis.clear();
        assert_eq!(
            validate_event(&invalid, &cutoff),
            Err(ApiError::InvalidWirePayload)
        );

        invalid = base.clone();
        invalid.event_title = "x".repeat(513);
        assert_eq!(validate_event(&invalid, &cutoff), Err(ApiError::LimitExceeded));

        invalid = base.clone();
        invalid.source_post_id.clear();
        assert_eq!(
            validate_event(&invalid, &cutoff),
            Err(ApiError::InvalidWirePayload)
        );

        invalid = base.clone();
        invalid.evidence_text = "x".repeat(4097);
        assert_eq!(validate_event(&invalid, &cutoff), Err(ApiError::LimitExceeded));

        invalid = base.clone();
        invalid.actor_ids = (0..65).map(|index| format!("actor-{index}")).collect();
        assert_eq!(validate_event(&invalid, &cutoff), Err(ApiError::LimitExceeded));

        invalid = base.clone();
        invalid.actor_ids = vec![String::new()];
        assert_eq!(
            validate_event(&invalid, &cutoff),
            Err(ApiError::InvalidWirePayload)
        );

        invalid = base.clone();
        invalid.event_time = "not-a-time".into();
        assert_eq!(
            validate_event(&invalid, &cutoff),
            Err(ApiError::InvalidWirePayload)
        );

        invalid = base.clone();
        invalid.available_at = "not-a-time".into();
        assert_eq!(
            validate_event(&invalid, &cutoff),
            Err(ApiError::InvalidWirePayload)
        );

        invalid = base.clone();
        invalid.available_at = "2026-08-20T00:00:00Z".into();
        assert_eq!(
            validate_event(&invalid, &cutoff),
            Err(ApiError::InvalidWirePayload)
        );

        let mut scheduled = base;
        scheduled.event_time = "2027-01-01T00:00:00Z".into();
        assert_eq!(validate_event(&scheduled, &cutoff), Ok(()));

        assert_eq!(validate_bounded_text("x", 1), Ok(()));
        assert_eq!(validate_bounded_text("é", 1), Err(ApiError::LimitExceeded));
        assert_eq!(validate_code("abc_123"), Ok(()));
        assert_eq!(validate_code("ABC"), Err(ApiError::InvalidWirePayload));
        assert!(parse_timestamp("2026-08-19T00:00:00Z").is_ok());
        assert_eq!(parse_timestamp("bad"), Err(ApiError::InvalidWirePayload));
    }

    #[test]
    fn projection_validation_and_findings_cover_success_and_failure_arms() {
        let request = request();
        let projection = project_history_projection(&request).expect("projection");
        assert_eq!(projection.events.first().expect("first").event_id, "award");
        assert_eq!(projection.events.last().expect("last").event_id, "rebid");
        assert_eq!(projection.participant_count, 6);
        assert_eq!(projection.findings.len(), 6);
        assert!(projection.findings.iter().all(|finding| {
            finding.related_event_ids.contains(&"focus".to_owned())
                && finding.evidence_post_ids.contains(&"post-focus".to_owned())
                && finding.summary.contains("temporal association")
                && finding.summary.contains("not a causal conclusion")
        }));

        let json = projection.to_json().expect("projection json");
        assert_eq!(
            ProjectHistoryProjection::from_json(&json).expect("projection decode"),
            projection
        );

        let focus_only_request = ProjectHistoryRequest {
            events: vec![event("focus", "voc_received", "2026-08-19T17:00:00Z")],
            ..request.clone()
        };
        let focus_only = project_history_projection(&focus_only_request).expect("focus only");
        assert!(focus_only.findings.is_empty());

        let mut invalid = projection.clone();
        invalid.contract_version += 1;
        assert_eq!(
            invalid.to_json(),
            Err(ApiError::UnsupportedContractVersion)
        );

        invalid = projection.clone();
        invalid.project_key.clear();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));

        invalid = projection.clone();
        invalid.project_name = "x".repeat(513);
        assert_eq!(invalid.to_json(), Err(ApiError::LimitExceeded));

        invalid = projection.clone();
        invalid.inference_status = "causal".into();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));

        invalid = projection.clone();
        invalid.events.clear();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));

        invalid = projection.clone();
        invalid.history_span_start = "not-a-time".into();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));

        invalid = projection;
        invalid.history_span_start = "2026-08-20T00:00:00Z".into();
        invalid.history_span_end = "2026-08-19T00:00:00Z".into();
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));
    }

    #[test]
    fn origin_validation_exercises_every_fail_closed_boundary() {
        assert_eq!(
            compose_https_target("https://tepp.example.test"),
            Ok(format!("https://tepp.example.test{PROJECT_HISTORY_PATH}"))
        );
        for hostile in [
            "",
            "http://tepp.example.test",
            "https://",
            "https:///path",
            "https://user@host",
            "https://host/path",
            "https://host?query",
            "https://host#fragment",
            "https://host\n",
            "https://ho'st",
            "https://host;drop",
            "https://host\\path",
            "https://host name",
            "https://postgres.example.test",
            "https://jdbc.example.test",
        ] {
            assert!(compose_https_target(hostile).is_err(), "accepted {hostile:?}");
        }
        let overlong = format!("https://{}", "a".repeat(2049));
        assert_eq!(compose_https_target(&overlong), Err(ApiError::LimitExceeded));
    }
}''',
    )

    append_once(
        live_source,
        "mod project_history_live_exhaustive_tests",
        r'''#[cfg(test)]
mod project_history_live_exhaustive_tests {
    use std::io::Cursor;

    use super::{
        LIVE_BODY_BYTE_LIMIT, PROJECT_HISTORY_PATH, read_http_request, require_request_line,
        split_request,
    };
    use crate::{ApiError, NARUON_ANALYSIS_RUN_PATH};

    #[test]
    fn request_line_accepts_only_the_two_published_post_routes() {
        assert_eq!(
            require_request_line(&format!("POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1")),
            Ok(NARUON_ANALYSIS_RUN_PATH)
        );
        assert_eq!(
            require_request_line(&format!("POST {PROJECT_HISTORY_PATH} HTTP/1.1")),
            Ok(PROJECT_HISTORY_PATH)
        );
        for hostile in [
            "GET /v1/analysis-runs HTTP/1.1",
            "POST",
            "POST /v1/unknown HTTP/1.1",
            "POST /v1/analysis-runs HTTP/2",
            "POST /v1/analysis-runs HTTP/1.1 extra",
        ] {
            assert_eq!(
                require_request_line(hostile),
                Err(ApiError::InvalidWirePayload)
            );
        }
    }

    #[test]
    fn live_body_limit_is_enforced_before_body_allocation_or_dispatch() {
        let declared = LIVE_BODY_BYTE_LIMIT + 1;
        let header = format!(
            "POST {PROJECT_HISTORY_PATH} HTTP/1.1\r\ncontent-length: {declared}\r\n\r\n"
        );
        assert_eq!(
            read_http_request(&mut Cursor::new(header.into_bytes())),
            Err(ApiError::LimitExceeded)
        );

        let body = "x".repeat(declared);
        let request = format!(
            "POST {PROJECT_HISTORY_PATH} HTTP/1.1\r\ncontent-length: {declared}\r\n\r\n{body}"
        );
        assert_eq!(split_request(&request), Err(ApiError::LimitExceeded));
    }
}''',
    )


if __name__ == "__main__":
    main()
