//! Cutoff-safe project-history projection for `LineageWeave` buyer surfaces.
//!
//! TEPP owns temporal validation and deterministic ordering. `LineageWeave` owns
//! authorization and selects the bounded source evidence supplied here. The
//! projection reports explicit temporal associations only; it never upgrades
//! sequence into causality or emits a psychometric score.

use std::collections::{BTreeSet, HashSet};

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::ApiError;
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};

/// Supported project-history request and response contract version.
pub const PROJECT_HISTORY_CONTRACT_VERSION: u16 = 1;

/// Versioned project-history path exposed by a TEPP service adapter.
pub const PROJECT_HISTORY_PATH: &str = "/v1/project-histories";

/// Maximum serialized request size accepted by the project-history contract.
pub const DEFAULT_PROJECT_HISTORY_BYTE_LIMIT: usize = 256 * 1024;

/// Maximum event count accepted in one project-history request.
pub const DEFAULT_PROJECT_HISTORY_EVENT_LIMIT: usize = 128;

/// Explicit event evidence supplied by an authorized modular consumer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectHistoryEvent {
    /// Consumer-owned opaque event identity.
    pub event_id: String,
    /// Bounded machine event type, such as `voc_received`.
    pub event_type_code: String,
    /// Buyer-readable event title grounded in the source evidence.
    pub event_title: String,
    /// Event occurrence instant as RFC 3339.
    pub occurred_at: String,
    /// Instant at which this evidence was available to the analysis.
    pub available_at: String,
    /// Authorized `LineageWeave` source-post identity.
    pub source_post_id: String,
    /// Bounded evidence excerpt; never an instruction or causal conclusion.
    pub evidence_text: String,
    /// Opaque actor identities explicitly attached to this event.
    pub actor_ids: Vec<String>,
}

/// Versioned request for a deterministic TEPP project-history projection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectHistoryRequest {
    /// Semantic contract version.
    pub contract_version: u16,
    /// Caller-supplied opaque idempotency key.
    pub idempotency_key: String,
    /// Authorized tenant or workspace identity.
    pub tenant_workspace_id: String,
    /// Consumer-owned stable project key.
    pub project_key: String,
    /// Buyer-readable project label.
    pub project_name: String,
    /// Maximum evidence-availability instant as RFC 3339.
    pub knowledge_cutoff: String,
    /// Event around which before/after findings are evaluated.
    pub focus_event_id: String,
    /// Explicit source-grounded events.
    pub events: Vec<ProjectHistoryEvent>,
}

/// One evidence-grounded temporal association in a project history.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectHistoryFinding {
    /// Stable finding code interpreted by consumer UI copy.
    pub finding_code: String,
    /// Non-causal explanation of the explicit event ordering.
    pub summary: String,
    /// Event identities supporting this finding.
    pub related_event_ids: Vec<String>,
    /// Source-post identities supporting this finding.
    pub evidence_post_ids: Vec<String>,
}

/// Deterministically ordered project-history response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectHistoryProjection {
    /// Semantic contract version.
    pub contract_version: u16,
    /// Consumer-owned stable project key.
    pub project_key: String,
    /// Buyer-readable project label.
    pub project_name: String,
    /// Focus event echoed after validation.
    pub focus_event_id: String,
    /// Earliest event instant in the response.
    pub history_span_start: String,
    /// Latest event instant in the response.
    pub history_span_end: String,
    /// Distinct explicit actor count across the supplied events.
    pub participant_count: usize,
    /// Fixed claim boundary: sequence is association, not causation.
    pub inference_status: String,
    /// Events ordered by occurrence instant and stable identity.
    pub events: Vec<ProjectHistoryEvent>,
    /// Findings derived only from explicit known event types.
    pub findings: Vec<ProjectHistoryFinding>,
}

/// HTTP exchange for a project-history request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectHistoryHttpExchange {
    /// HTTP method, always `POST`.
    pub method: &'static str,
    /// Absolute HTTPS target ending in [`PROJECT_HISTORY_PATH`].
    pub target_url: String,
    /// Exact version, consumer, content, and idempotency headers.
    pub headers: Vec<(String, String)>,
    /// Validated JSON request body.
    pub body: String,
}

impl ProjectHistoryRequest {
    /// Parse and validate a project-history request using the default limit.
    ///
    /// # Errors
    ///
    /// Returns a version, size, JSON, timestamp, leakage, or field error.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)
    }

    /// Parse and validate a project-history request using a caller limit.
    ///
    /// # Errors
    ///
    /// Returns a version, size, JSON, timestamp, leakage, or field error.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        let request: Self = from_json(payload)?;
        request.validate()?;
        Ok(request)
    }

    /// Serialize a validated project-history request.
    ///
    /// # Errors
    ///
    /// Returns a field-validation or serialization error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, PROJECT_HISTORY_CONTRACT_VERSION)?;
        validate_bounded_text(&self.idempotency_key, 256)?;
        validate_bounded_text(&self.tenant_workspace_id, 256)?;
        validate_bounded_text(&self.project_key, 256)?;
        validate_bounded_text(&self.project_name, 512)?;
        validate_bounded_text(&self.focus_event_id, 256)?;
        if self.events.is_empty() || self.events.len() > DEFAULT_PROJECT_HISTORY_EVENT_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        let cutoff = parse_timestamp(&self.knowledge_cutoff)?;
        if cutoff > Timestamp::now() {
            return Err(ApiError::InvalidWirePayload);
        }
        let mut event_ids = HashSet::with_capacity(self.events.len());
        let mut focus_found = false;
        for event in &self.events {
            validate_event(event, &cutoff)?;
            if !event_ids.insert(event.event_id.as_str()) {
                return Err(ApiError::InvalidWirePayload);
            }
            focus_found |= event.event_id == self.focus_event_id;
        }
        if !focus_found {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

impl ProjectHistoryProjection {
    /// Parse and validate a serialized TEPP projection.
    ///
    /// # Errors
    ///
    /// Returns a JSON, version, field, or claim-boundary error.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        let projection: Self = from_json(payload)?;
        projection.validate()?;
        Ok(projection)
    }

    /// Serialize a validated TEPP projection.
    ///
    /// # Errors
    ///
    /// Returns a validation or serialization error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, PROJECT_HISTORY_CONTRACT_VERSION)?;
        validate_bounded_text(&self.project_key, 256)?;
        validate_bounded_text(&self.project_name, 512)?;
        validate_bounded_text(&self.focus_event_id, 256)?;
        if self.inference_status != "temporal_association_only" || self.events.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        let start = parse_timestamp(&self.history_span_start)?;
        let end = parse_timestamp(&self.history_span_end)?;
        if start > end {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

/// Build a deterministic, cutoff-safe project-history projection.
///
/// Findings are created only from explicit event type codes around the focus
/// event. The function does not infer causality, missing events, or latent
/// scores.
///
/// # Errors
///
/// Returns a fail-closed request validation error.
pub fn project_history_projection(
    request: &ProjectHistoryRequest,
) -> Result<ProjectHistoryProjection, ApiError> {
    request.validate()?;
    let mut ordered = request.events.clone();
    ordered.sort_by(|left, right| {
        let left_time = parse_timestamp(&left.occurred_at);
        let right_time = parse_timestamp(&right.occurred_at);
        match (left_time, right_time) {
            (Ok(left_time), Ok(right_time)) => left_time
                .cmp(&right_time)
                .then_with(|| left.event_id.cmp(&right.event_id)),
            _ => std::cmp::Ordering::Equal,
        }
    });
    let focus_index = ordered
        .iter()
        .position(|event| event.event_id == request.focus_event_id)
        .ok_or(ApiError::InvalidWirePayload)?;
    let findings = build_findings(&ordered, focus_index);
    let participant_count = ordered
        .iter()
        .flat_map(|event| event.actor_ids.iter().map(String::as_str))
        .collect::<BTreeSet<_>>()
        .len();
    let history_span_start = ordered
        .first()
        .map(|event| event.occurred_at.clone())
        .ok_or(ApiError::InvalidWirePayload)?;
    let history_span_end = ordered
        .last()
        .map(|event| event.occurred_at.clone())
        .ok_or(ApiError::InvalidWirePayload)?;
    Ok(ProjectHistoryProjection {
        contract_version: PROJECT_HISTORY_CONTRACT_VERSION,
        project_key: request.project_key.clone(),
        project_name: request.project_name.clone(),
        focus_event_id: request.focus_event_id.clone(),
        history_span_start,
        history_span_end,
        participant_count,
        inference_status: "temporal_association_only".into(),
        events: ordered,
        findings,
    })
}

pub(crate) fn build_project_history_exchange(
    origin: &str,
    consumer_code: &str,
    request: &ProjectHistoryRequest,
) -> Result<ProjectHistoryHttpExchange, ApiError> {
    validate_bounded_text(consumer_code, 64)?;
    let target_url = compose_https_target(origin)?;
    let body = request.to_json()?;
    Ok(ProjectHistoryHttpExchange {
        method: "POST",
        target_url,
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), consumer_code.to_owned()),
            (
                "tepp-contract-version".into(),
                PROJECT_HISTORY_CONTRACT_VERSION.to_string(),
            ),
            ("idempotency-key".into(), request.idempotency_key.clone()),
        ],
        body,
    })
}

fn validate_event(event: &ProjectHistoryEvent, cutoff: &Timestamp) -> Result<(), ApiError> {
    validate_bounded_text(&event.event_id, 256)?;
    validate_code(&event.event_type_code)?;
    validate_bounded_text(&event.event_title, 512)?;
    validate_bounded_text(&event.source_post_id, 256)?;
    validate_bounded_text(&event.evidence_text, 4096)?;
    if event.actor_ids.len() > 64 {
        return Err(ApiError::LimitExceeded);
    }
    for actor_id in &event.actor_ids {
        validate_bounded_text(actor_id, 256)?;
    }
    let occurred_at = parse_timestamp(&event.occurred_at)?;
    let available_at = parse_timestamp(&event.available_at)?;
    if occurred_at > *cutoff || available_at > *cutoff {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

fn validate_bounded_text(value: &str, maximum_bytes: usize) -> Result<(), ApiError> {
    require_nonempty(value)?;
    if value.len() > maximum_bytes {
        return Err(ApiError::LimitExceeded);
    }
    Ok(())
}

fn validate_code(value: &str) -> Result<(), ApiError> {
    validate_bounded_text(value, 64)?;
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

fn parse_timestamp(value: &str) -> Result<Timestamp, ApiError> {
    value
        .parse::<Timestamp>()
        .map_err(|_| ApiError::InvalidWirePayload)
}

fn build_findings(
    ordered: &[ProjectHistoryEvent],
    focus_index: usize,
) -> Vec<ProjectHistoryFinding> {
    let before = &ordered[..focus_index];
    let after = &ordered[focus_index + 1..];
    let specification = first_type(before, "specification_changed");
    let handoff = first_type(before, "handoff_recorded");
    let mut findings = Vec::new();
    append_single_finding(
        &mut findings,
        first_type(before, "contract_awarded"),
        "contract_award_before_focus",
        "An explicit contract-award event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        specification,
        "specification_change_before_focus",
        "An explicit specification-change event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        first_type(before, "delivered"),
        "delivery_before_focus",
        "An explicit delivery event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        handoff,
        "handoff_before_focus",
        "An explicit operational-handoff event precedes the focus event.",
    );
    append_single_finding(
        &mut findings,
        first_type(after, "rebid_started"),
        "rebid_after_focus",
        "An explicit rebid event follows the focus event.",
    );
    if let (Some(specification), Some(handoff)) = (specification, handoff) {
        findings.push(combined_finding(specification, handoff));
    }
    findings
}

fn first_type<'a>(
    events: &'a [ProjectHistoryEvent],
    event_type_code: &str,
) -> Option<&'a ProjectHistoryEvent> {
    events
        .iter()
        .find(|event| event.event_type_code == event_type_code)
}

fn append_single_finding(
    findings: &mut Vec<ProjectHistoryFinding>,
    event: Option<&ProjectHistoryEvent>,
    finding_code: &str,
    summary: &str,
) {
    if let Some(event) = event {
        findings.push(ProjectHistoryFinding {
            finding_code: finding_code.to_owned(),
            summary: summary.to_owned(),
            related_event_ids: vec![event.event_id.clone()],
            evidence_post_ids: vec![event.source_post_id.clone()],
        });
    }
}

fn combined_finding(
    specification: &ProjectHistoryEvent,
    handoff: &ProjectHistoryEvent,
) -> ProjectHistoryFinding {
    let evidence_post_ids = [
        specification.source_post_id.clone(),
        handoff.source_post_id.clone(),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect();
    ProjectHistoryFinding {
        finding_code: "specification_change_and_handoff_before_focus".into(),
        summary: "Explicit specification-change and handoff events precede the focus event; this is a temporal association, not a causal conclusion.".into(),
        related_event_ids: vec![
            specification.event_id.clone(),
            handoff.event_id.clone(),
        ],
        evidence_post_ids,
    }
}

fn compose_https_target(origin: &str) -> Result<String, ApiError> {
    validate_bounded_text(origin, 2048)?;
    let host = origin
        .strip_prefix("https://")
        .ok_or(ApiError::InvalidWirePayload)?;
    if host.is_empty()
        || host.starts_with('/')
        || host.contains('@')
        || host.contains('/')
        || host.contains('?')
        || host.contains('#')
        || host
            .chars()
            .any(|character| character.is_control() || matches!(character, '\'' | ';' | '\\' | ' '))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    let lowered = host.to_ascii_lowercase();
    if lowered.contains("postgres") || lowered.contains("jdbc") {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(format!("{origin}{PROJECT_HISTORY_PATH}"))
}

#[cfg(test)]
mod tests {
    use super::{
        PROJECT_HISTORY_CONTRACT_VERSION, ProjectHistoryEvent, ProjectHistoryProjection,
        ProjectHistoryRequest, project_history_projection,
    };
    use crate::ApiError;

    fn request_with_single_event() -> ProjectHistoryRequest {
        ProjectHistoryRequest {
            contract_version: PROJECT_HISTORY_CONTRACT_VERSION,
            idempotency_key: "idem".into(),
            tenant_workspace_id: "tenant".into(),
            project_key: "project".into(),
            project_name: "Project".into(),
            knowledge_cutoff: "2026-08-19T23:59:59Z".into(),
            focus_event_id: "focus".into(),
            events: vec![ProjectHistoryEvent {
                event_id: "focus".into(),
                event_type_code: "voc_received".into(),
                event_title: "VOC".into(),
                occurred_at: "2026-08-19T09:00:00Z".into(),
                available_at: "2026-08-19T10:00:00Z".into(),
                source_post_id: "post".into(),
                evidence_text: "explicit evidence".into(),
                actor_ids: Vec::new(),
            }],
        }
    }

    #[test]
    fn projection_round_trip_preserves_the_non_causal_claim_boundary() {
        let request = request_with_single_event();
        let projection = project_history_projection(&request).expect("projection");
        let json = projection.to_json().expect("json");
        assert_eq!(
            ProjectHistoryProjection::from_json(&json).expect("decode"),
            projection
        );
        assert!(projection.findings.is_empty());
        assert_eq!(projection.participant_count, 0);
    }

    #[test]
    fn request_refuses_missing_focus_bad_codes_and_excess_events() {
        let mut missing_focus = request_with_single_event();
        missing_focus.focus_event_id = "missing".into();
        assert_eq!(
            project_history_projection(&missing_focus),
            Err(ApiError::InvalidWirePayload)
        );

        let mut bad_code = request_with_single_event();
        bad_code.events[0].event_type_code = "VOC Received".into();
        assert_eq!(
            project_history_projection(&bad_code),
            Err(ApiError::InvalidWirePayload)
        );

        let mut excess = request_with_single_event();
        excess.events =
            vec![excess.events[0].clone(); super::DEFAULT_PROJECT_HISTORY_EVENT_LIMIT + 1];
        assert_eq!(
            project_history_projection(&excess),
            Err(ApiError::LimitExceeded)
        );
    }
}
