//! Cutoff-safe project-history projection for `LineageWeave` buyer surfaces.
//!
//! TEPP owns temporal validation and deterministic ordering. `LineageWeave` owns
//! authorization and selects the bounded source evidence supplied here. The
//! projection reports explicit temporal associations only; it never upgrades
//! sequence into causality or emits a psychometric score.

use std::collections::{BTreeSet, HashSet};
use std::net::{IpAddr, Ipv6Addr};

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use temporal_core::{KnowledgeCutoff, TemporalInstant};

use crate::ApiError;
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json_with_limit,
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
    /// Knowledge cutoff applied to every event in the response.
    pub knowledge_cutoff: String,
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
        to_json_with_limit(self, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, PROJECT_HISTORY_CONTRACT_VERSION)?;
        validate_http_field_value(&self.idempotency_key, 256)?;
        validate_bounded_text(&self.tenant_workspace_id, 256)?;
        validate_bounded_text(&self.project_key, 256)?;
        validate_bounded_text(&self.project_name, 512)?;
        validate_bounded_text(&self.focus_event_id, 256)?;
        if self.events.is_empty() || self.events.len() > DEFAULT_PROJECT_HISTORY_EVENT_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        let cutoff = parse_knowledge_cutoff(&self.knowledge_cutoff)?;
        if cutoff_is_in_future(cutoff)? {
            return Err(ApiError::InvalidWirePayload);
        }
        let mut event_ids = HashSet::with_capacity(self.events.len());
        let mut focus_found = false;
        for event in &self.events {
            validate_event(event, cutoff.instant())?;
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
        Self::from_json_with_limit(payload, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)
    }

    /// Parse and validate a serialized TEPP projection with a caller limit.
    ///
    /// # Errors
    ///
    /// Returns a size, JSON, version, field, or claim-boundary error.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
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
        to_json_with_limit(self, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, PROJECT_HISTORY_CONTRACT_VERSION)?;
        validate_bounded_text(&self.project_key, 256)?;
        validate_bounded_text(&self.project_name, 512)?;
        validate_bounded_text(&self.focus_event_id, 256)?;
        if self.inference_status != "temporal_association_only" || self.events.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.events.len() > DEFAULT_PROJECT_HISTORY_EVENT_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        let cutoff = parse_knowledge_cutoff(&self.knowledge_cutoff)?;
        if cutoff_is_in_future(cutoff)? {
            return Err(ApiError::InvalidWirePayload);
        }
        let mut event_ids = HashSet::with_capacity(self.events.len());
        let mut focus_index = None;
        for (index, event) in self.events.iter().enumerate() {
            validate_event(event, cutoff.instant())?;
            if !event_ids.insert(event.event_id.as_str()) {
                return Err(ApiError::InvalidWirePayload);
            }
            if event.event_id == self.focus_event_id {
                focus_index = Some(index);
            }
        }
        let focus_index = focus_index.ok_or(ApiError::InvalidWirePayload)?;
        let mut previous = None;
        for event in &self.events {
            let occurred_at = parse_timestamp(&event.occurred_at)?;
            if let Some((previous_time, previous_id)) = previous
                && (occurred_at < previous_time
                    || (occurred_at == previous_time && event.event_id.as_str() <= previous_id))
            {
                return Err(ApiError::InvalidWirePayload);
            }
            previous = Some((occurred_at, event.event_id.as_str()));
        }
        let start = parse_timestamp(&self.history_span_start)?;
        let end = parse_timestamp(&self.history_span_end)?;
        let first_event_time = parse_timestamp(&self.events[0].occurred_at)?;
        // The non-empty guard above makes this index safe and removes an
        // unreachable second empty-events error path from the response contract.
        let last_event_time = parse_timestamp(&self.events[self.events.len() - 1].occurred_at)?;
        if start > end || start != first_event_time || end != last_event_time {
            return Err(ApiError::InvalidWirePayload);
        }
        let participant_count = self
            .events
            .iter()
            .flat_map(|event| event.actor_ids.iter().map(String::as_str))
            .collect::<BTreeSet<_>>()
            .len();
        if self.participant_count != participant_count
            || self.findings != build_findings(&self.events, focus_index)
        {
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
    ordered.sort_by_key(|event| {
        (
            parse_timestamp(&event.occurred_at).ok(),
            event.event_id.clone(),
        )
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
    let projection = ProjectHistoryProjection {
        contract_version: PROJECT_HISTORY_CONTRACT_VERSION,
        project_key: request.project_key.clone(),
        project_name: request.project_name.clone(),
        focus_event_id: request.focus_event_id.clone(),
        knowledge_cutoff: request.knowledge_cutoff.clone(),
        history_span_start,
        history_span_end,
        participant_count,
        inference_status: "temporal_association_only".into(),
        events: ordered,
        findings,
    };
    projection.to_json()?;
    Ok(projection)
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

fn validate_event(event: &ProjectHistoryEvent, cutoff: TemporalInstant) -> Result<(), ApiError> {
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
    parse_timestamp(&event.occurred_at)?;
    let available_at = parse_timestamp(&event.available_at)?;
    if available_at > cutoff {
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

fn validate_http_field_value(value: &str, maximum_bytes: usize) -> Result<(), ApiError> {
    validate_bounded_text(value, maximum_bytes)?;
    (!value.chars().any(char::is_control))
        .then_some(())
        .ok_or(ApiError::InvalidWirePayload)
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

fn parse_knowledge_cutoff(value: &str) -> Result<KnowledgeCutoff, ApiError> {
    KnowledgeCutoff::parse_rfc3339(value).map_err(|_| ApiError::InvalidWirePayload)
}

fn parse_timestamp(value: &str) -> Result<TemporalInstant, ApiError> {
    TemporalInstant::parse_rfc3339(value).map_err(|_| ApiError::InvalidWirePayload)
}

fn cutoff_is_in_future(cutoff: KnowledgeCutoff) -> Result<bool, ApiError> {
    let now = KnowledgeCutoff::parse_rfc3339(&Timestamp::now().to_string())
        .map_err(|_| ApiError::InvalidWirePayload)?;
    Ok(cutoff > now)
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
    let authority = origin
        .strip_prefix("https://")
        .ok_or(ApiError::InvalidWirePayload)?;
    validate_https_authority(authority)?;
    let lowered = authority.to_ascii_lowercase();
    if lowered.contains("postgres") || lowered.contains("jdbc") {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(format!("{origin}{PROJECT_HISTORY_PATH}"))
}

fn validate_https_authority(authority: &str) -> Result<(), ApiError> {
    if authority.is_empty()
        || authority.contains('@')
        || authority.contains('/')
        || authority.contains('?')
        || authority.contains('#')
        || authority
            .chars()
            .any(|character| matches!(character, '\'' | ';' | '\\' | ' ') || character.is_control())
    {
        return Err(ApiError::InvalidWirePayload);
    }

    if let Some(bracketed) = authority.strip_prefix('[') {
        let close = bracketed.find(']').ok_or(ApiError::InvalidWirePayload)?;
        let host = &bracketed[..close];
        host.parse::<Ipv6Addr>()
            .map_err(|_| ApiError::InvalidWirePayload)?;
        let suffix = &bracketed[close + 1..];
        if suffix.is_empty() {
            return Ok(());
        }
        let port = suffix
            .strip_prefix(':')
            .ok_or(ApiError::InvalidWirePayload)?;
        return validate_https_port(port);
    }

    if authority.contains(']') || authority.matches(':').count() > 1 {
        return Err(ApiError::InvalidWirePayload);
    }
    let (host, port) = authority
        .rsplit_once(':')
        .map_or((authority, None), |(host, port)| (host, Some(port)));
    validate_https_host(host)?;
    if let Some(port) = port {
        validate_https_port(port)?;
    }
    Ok(())
}

fn validate_https_host(host: &str) -> Result<(), ApiError> {
    if host.is_empty() || host.len() > 253 {
        return Err(ApiError::InvalidWirePayload);
    }
    if host.parse::<IpAddr>().is_ok() {
        return Ok(());
    }
    for label in host.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    Ok(())
}

fn validate_https_port(port: &str) -> Result<(), ApiError> {
    if port.is_empty() || port.parse::<u16>().map_or(true, |value| value == 0) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        PROJECT_HISTORY_CONTRACT_VERSION, ProjectHistoryEvent, ProjectHistoryProjection,
        ProjectHistoryRequest, build_project_history_exchange, compose_https_target,
        project_history_projection, validate_code, validate_http_field_value,
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
    fn projection_counts_memberships_and_emits_explicit_findings() {
        let mut request = request_with_single_event();
        let event = |event_id: &str, event_type_code: &str, occurred_at: &str, actor_id: &str| {
            ProjectHistoryEvent {
                event_id: event_id.into(),
                event_type_code: event_type_code.into(),
                event_title: event_type_code.into(),
                occurred_at: occurred_at.into(),
                available_at: occurred_at.into(),
                source_post_id: format!("post-{event_id}"),
                evidence_text: "explicit evidence".into(),
                actor_ids: vec![actor_id.into()],
            }
        };
        request.events = vec![
            event(
                "award",
                "contract_awarded",
                "2026-08-19T08:00:00Z",
                "actor-1",
            ),
            event(
                "specification",
                "specification_changed",
                "2026-08-19T09:00:00Z",
                "actor-1",
            ),
            event("delivery", "delivered", "2026-08-19T10:00:00Z", "actor-2"),
            event(
                "handoff",
                "handoff_recorded",
                "2026-08-19T11:00:00Z",
                "actor-2",
            ),
            event("focus", "voc_received", "2026-08-19T12:00:00Z", "actor-3"),
            event("rebid", "rebid_started", "2026-08-19T13:00:00Z", "actor-3"),
        ];
        let projection = project_history_projection(&request).expect("projection");
        assert_eq!(projection.participant_count, 3);
        assert_eq!(projection.findings.len(), 6);
        let payload = projection.to_json().expect("projection json");
        assert_eq!(
            ProjectHistoryProjection::from_json(&payload),
            Ok(projection)
        );
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

        let mut injected = request_with_single_event();
        injected.idempotency_key = "safe\r\nx-api-key: secret".into();
        assert_eq!(injected.to_json(), Err(ApiError::InvalidWirePayload));
        assert_eq!(
            validate_http_field_value("safe\0value", 256),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn validation_edges_cover_cutoffs_bounds_projection_and_origins() {
        let mut empty = request_with_single_event();
        empty.events.clear();
        assert_eq!(
            project_history_projection(&empty),
            Err(ApiError::LimitExceeded)
        );

        let mut future_cutoff = request_with_single_event();
        future_cutoff.knowledge_cutoff = "2999-01-01T00:00:00Z".into();
        assert_eq!(
            project_history_projection(&future_cutoff),
            Err(ApiError::InvalidWirePayload)
        );

        let mut occurred_after_cutoff = request_with_single_event();
        occurred_after_cutoff.events[0].occurred_at = "2026-08-20T00:00:00Z".into();
        assert!(project_history_projection(&occurred_after_cutoff).is_ok());

        let mut available_after_cutoff = request_with_single_event();
        available_after_cutoff.events[0].available_at = "2026-08-20T00:00:00Z".into();
        assert_eq!(
            project_history_projection(&available_after_cutoff),
            Err(ApiError::InvalidWirePayload)
        );

        let mut too_many_actors = request_with_single_event();
        too_many_actors.events[0].actor_ids = vec!["actor".into(); 65];
        assert_eq!(
            project_history_projection(&too_many_actors),
            Err(ApiError::LimitExceeded)
        );

        let mut oversized_title = request_with_single_event();
        oversized_title.events[0].event_title = "x".repeat(513);
        assert_eq!(
            project_history_projection(&oversized_title),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(validate_code("_"), Ok(()));
        assert_eq!(validate_code("1"), Ok(()));

        let projection =
            project_history_projection(&request_with_single_event()).expect("projection");
        let mut wrong_status = projection.clone();
        wrong_status.inference_status = "causal_score".into();
        assert_eq!(wrong_status.to_json(), Err(ApiError::InvalidWirePayload));

        let mut empty_projection = projection.clone();
        empty_projection.events.clear();
        assert_eq!(
            empty_projection.to_json(),
            Err(ApiError::InvalidWirePayload)
        );

        let mut inverted_span = projection;
        inverted_span.history_span_start = "2026-08-20T00:00:00Z".into();
        inverted_span.history_span_end = "2026-08-19T00:00:00Z".into();
        assert_eq!(inverted_span.to_json(), Err(ApiError::InvalidWirePayload));

        let request = request_with_single_event();
        assert_eq!(
            build_project_history_exchange("", "lineageweave", &request),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            build_project_history_exchange("https://example.test", &"x".repeat(65), &request),
            Err(ApiError::LimitExceeded)
        );
        assert!(
            build_project_history_exchange("https://example.test", "lineageweave", &request)
                .is_ok()
        );
    }

    #[test]
    fn malformed_https_authorities_are_rejected() {
        for origin in [
            "http://example.test",
            "https://",
            "https://:",
            "https:///path",
            "https://user@example.test",
            "https://example.test/path",
            "https://example.test?query",
            "https://example.test#fragment",
            "https://example.test:",
            "https://example.test:not-a-port",
            "https://example.test:65536",
            "https://[::1",
            "https://[not-ipv6]",
            "https://::1",
            "https://example]test",
            "https://example test",
            "https://example'test",
            "https://example;test",
            "https://example\\test",
            "https://example\ntest",
            "https://postgres.example.test",
            "https://jdbc.example.test",
        ] {
            assert_eq!(
                compose_https_target(origin),
                Err(ApiError::InvalidWirePayload),
                "origin must be rejected: {origin:?}"
            );
        }
        assert_eq!(
            compose_https_target("https://example.test").expect("origin"),
            "https://example.test/v1/project-histories"
        );
        assert!(compose_https_target("https://example.test:443").is_ok());
        assert!(compose_https_target("https://127.0.0.1").is_ok());
        assert!(compose_https_target("https://[::1]").is_ok());
        assert!(compose_https_target("https://[::1]:443").is_ok());
        for origin in [
            "https://-example.test",
            "https://example-.test",
            "https://example..test",
            "https://example_.test",
        ] {
            assert_eq!(
                compose_https_target(origin),
                Err(ApiError::InvalidWirePayload)
            );
        }
        let long_host = format!("https://{}", "a.".repeat(127) + "a");
        assert_eq!(
            compose_https_target(&long_host),
            Err(ApiError::InvalidWirePayload)
        );
        let long_label = format!("https://{}.test", "a".repeat(64));
        assert_eq!(
            compose_https_target(&long_label),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
