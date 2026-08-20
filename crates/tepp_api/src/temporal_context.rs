//! Versioned, cutoff-safe temporal evidence context for modular consumers.

use crate::ApiError;
use crate::lineageweave_http::LINEAGEWEAVE_CONSUMER_CODE;
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff, TemporalInstant};

/// Supported temporal-context contract version.
pub const TEMPORAL_CONTEXT_CONTRACT_VERSION: u16 = 1;

/// Versioned temporal-context HTTP path.
pub const TEMPORAL_CONTEXT_PATH: &str = "/v1/temporal-context";

/// Claim boundary for temporal association output.
pub const TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY: &str = "association_not_causal";

const DEFAULT_TEMPORAL_CONTEXT_BYTE_LIMIT: usize = 64 * 1024;
const MAXIMUM_TEMPORAL_CONTEXT_EVENTS: usize = 1024;

/// One opaque event offered for cutoff-safe temporal ordering.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalContextEvent {
    /// Opaque event identity.
    pub event_id: String,
    /// Opaque source-post identity.
    pub source_post_id: String,
    /// Stable event-type code.
    pub event_type_code: String,
    /// Bounded display label for the event.
    pub event_label: String,
    /// Event or valid time.
    pub event_time: String,
    /// Availability time for historical eligibility.
    pub available_time: String,
    /// Opaque project identity, when known.
    pub project_reference: Option<String>,
    /// Opaque actor identities participating in the event.
    pub actor_references: Vec<String>,
}

/// Request for one bounded temporal evidence context.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalContextRequest {
    /// Semantic contract version.
    pub contract_version: u16,
    /// Published modular-consumer identity.
    pub consumer_code: String,
    /// Latest availability time permitted in the context.
    pub knowledge_cutoff: String,
    /// Optional opaque post identity whose event is the subject.
    pub subject_post_id: Option<String>,
    /// Bounded source events.
    pub events: Vec<TemporalContextEvent>,
}

/// One ordered event in a temporal context response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalContextTimelineEvent {
    /// Opaque event identity.
    pub event_id: String,
    /// Opaque source-post identity.
    pub source_post_id: String,
    /// Stable event-type code.
    pub event_type_code: String,
    /// Bounded display label for the event.
    pub event_label: String,
    /// Original event-time representation.
    pub event_time: String,
    /// Optional opaque project identity.
    pub project_reference: Option<String>,
    /// Opaque actor identities.
    pub actor_references: Vec<String>,
    /// Zero-based deterministic temporal sequence position.
    pub sequence_ordinal: usize,
    /// Whether this event is associated with the requested subject post.
    pub is_subject: bool,
}

/// One non-causal forward temporal relation in a context response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalContextRelation {
    /// Earlier event identity.
    pub from_event_id: String,
    /// Later event identity.
    pub to_event_id: String,
    /// Stable relation code.
    pub relation_code: String,
}

/// One candidate transition gap that is not a causal claim.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalTransitionGapCandidate {
    /// Earlier event identity.
    pub from_event_id: String,
    /// Later event identity.
    pub to_event_id: String,
    /// Explicit non-causal evidence-status code.
    pub evidence_status_code: String,
}

/// Ordered temporal evidence context returned to a modular consumer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalContextResponse {
    /// Semantic contract version.
    pub contract_version: u16,
    /// Explicit claim boundary for every relation and gap candidate.
    pub claim_boundary: String,
    /// Events ordered by absolute event time.
    pub timeline_events: Vec<TemporalContextTimelineEvent>,
    /// Adjacent forward-only temporal relations.
    pub temporal_relations: Vec<TemporalContextRelation>,
    /// Adjacent candidate gaps that are not causal claims.
    pub transition_gap_candidates: Vec<TemporalTransitionGapCandidate>,
    /// Source-post identities in response order.
    pub source_post_ids: Vec<String>,
}

impl TemporalContextRequest {
    /// Parse and validate a temporal-context request with the default limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, version, limit, consumer, time, or shape
    /// error.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_TEMPORAL_CONTEXT_BYTE_LIMIT)
    }

    /// Parse and validate a temporal-context request with a caller limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, version, limit, consumer, time, or shape
    /// error.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        let request: Self = from_json(payload)?;
        request.validate()?;
        Ok(request)
    }

    /// Serialize a temporal-context request after validation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed validation or serialization error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        self.validated_ordered_events().map(|_| ())
    }

    fn validated_ordered_events(
        &self,
    ) -> Result<Vec<(TemporalContextEvent, TemporalInstant)>, ApiError> {
        require_contract_version(self.contract_version, TEMPORAL_CONTEXT_CONTRACT_VERSION)?;
        if self.consumer_code != LINEAGEWEAVE_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        let cutoff = KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff)
            .map_err(|_| ApiError::InvalidWirePayload)?;
        if self.events.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.events.len() > MAXIMUM_TEMPORAL_CONTEXT_EVENTS {
            return Err(ApiError::LimitExceeded);
        }
        if let Some(subject_post_id) = &self.subject_post_id {
            require_nonempty(subject_post_id)?;
        }
        let mut event_ids = HashSet::with_capacity(self.events.len());
        let mut ordered = Vec::with_capacity(self.events.len());
        for event in &self.events {
            let event_time = validate_event(event, cutoff.instant(), &mut event_ids)?;
            ordered.push((event.clone(), event_time));
        }
        if let Some(subject_post_id) = &self.subject_post_id
            && !self
                .events
                .iter()
                .any(|event| event.source_post_id == *subject_post_id)
        {
            return Err(ApiError::InvalidWirePayload);
        }
        ordered.sort_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| left.0.event_id.cmp(&right.0.event_id))
        });
        Ok(ordered)
    }
}

impl TemporalContextResponse {
    /// Parse and validate a temporal-context response.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, version, limit, or response-shape error.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_TEMPORAL_CONTEXT_BYTE_LIMIT)
    }

    /// Parse and validate a temporal-context response with a caller limit.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed wire, version, limit, or response-shape error.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        let response: Self = from_json(payload)?;
        response.validate()?;
        Ok(response)
    }

    /// Serialize a temporal-context response after validation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed validation or serialization error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, TEMPORAL_CONTEXT_CONTRACT_VERSION)?;
        if self.claim_boundary != TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.timeline_events.is_empty()
            || self.timeline_events.len() != self.source_post_ids.len()
            || self.temporal_relations.len().checked_add(1) != Some(self.timeline_events.len())
            || self.transition_gap_candidates.len().checked_add(1)
                != Some(self.timeline_events.len())
        {
            return Err(ApiError::InvalidWirePayload);
        }
        let mut event_ids = HashSet::with_capacity(self.timeline_events.len());
        let mut previous_key: Option<(TemporalInstant, &str)> = None;
        for (ordinal, event) in self.timeline_events.iter().enumerate() {
            if event.sequence_ordinal != ordinal
                || event.event_id.is_empty()
                || event.source_post_id.is_empty()
                || event.event_type_code.is_empty()
                || event.event_label.is_empty()
                || event.event_time.is_empty()
                || event.actor_references.is_empty()
                || self.source_post_ids[ordinal] != event.source_post_id
            {
                return Err(ApiError::InvalidWirePayload);
            }
            if !event_ids.insert(event.event_id.clone()) {
                return Err(ApiError::InvalidWirePayload);
            }
            let event_time = EventTime::parse_rfc3339(&event.event_time)
                .map_err(|_| ApiError::InvalidWirePayload)?
                .instant();
            if let Some((previous_time, previous_id)) = previous_key
                && (event_time < previous_time
                    || (event_time == previous_time && event.event_id.as_str() <= previous_id))
            {
                return Err(ApiError::InvalidWirePayload);
            }
            previous_key = Some((event_time, event.event_id.as_str()));
            if let Some(project_reference) = &event.project_reference {
                require_nonempty(project_reference)?;
            }
            for actor_reference in &event.actor_references {
                require_nonempty(actor_reference)?;
            }
        }
        for (index, relation) in self.temporal_relations.iter().enumerate() {
            if relation.from_event_id != self.timeline_events[index].event_id
                || relation.to_event_id != self.timeline_events[index + 1].event_id
                || relation.relation_code != "before"
            {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        for (index, candidate) in self.transition_gap_candidates.iter().enumerate() {
            if candidate.from_event_id != self.timeline_events[index].event_id
                || candidate.to_event_id != self.timeline_events[index + 1].event_id
                || candidate.evidence_status_code != "candidate_not_causal"
            {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        Ok(())
    }
}

/// Build an ordered, cutoff-safe temporal context without causal inference.
///
/// # Errors
///
/// Returns a fail-closed error when the request contains future-available,
/// duplicate, malformed, or otherwise invalid evidence.
pub fn build_temporal_context(
    request: &TemporalContextRequest,
) -> Result<TemporalContextResponse, ApiError> {
    let ordered = request.validated_ordered_events()?;
    let timeline_events = ordered
        .iter()
        .enumerate()
        .map(
            |(sequence_ordinal, (event, _))| TemporalContextTimelineEvent {
                event_id: event.event_id.clone(),
                source_post_id: event.source_post_id.clone(),
                event_type_code: event.event_type_code.clone(),
                event_label: event.event_label.clone(),
                event_time: event.event_time.clone(),
                project_reference: event.project_reference.clone(),
                actor_references: event.actor_references.clone(),
                sequence_ordinal,
                is_subject: request
                    .subject_post_id
                    .as_ref()
                    .is_some_and(|subject| subject == &event.source_post_id),
            },
        )
        .collect::<Vec<_>>();
    let temporal_relations = adjacent_relations(&ordered);
    let transition_gap_candidates = adjacent_gaps(&ordered);
    let response = TemporalContextResponse {
        contract_version: TEMPORAL_CONTEXT_CONTRACT_VERSION,
        claim_boundary: TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY.into(),
        source_post_ids: timeline_events
            .iter()
            .map(|event| event.source_post_id.clone())
            .collect(),
        timeline_events,
        temporal_relations,
        transition_gap_candidates,
    };
    response.validate()?;
    Ok(response)
}

fn validate_event(
    event: &TemporalContextEvent,
    cutoff: TemporalInstant,
    event_ids: &mut HashSet<String>,
) -> Result<TemporalInstant, ApiError> {
    for value in [
        &event.event_id,
        &event.source_post_id,
        &event.event_type_code,
        &event.event_label,
        &event.event_time,
        &event.available_time,
    ] {
        require_nonempty(value)?;
    }
    if !event_ids.insert(event.event_id.clone()) {
        return Err(ApiError::InvalidWirePayload);
    }
    if let Some(project_reference) = &event.project_reference {
        require_nonempty(project_reference)?;
    }
    if event.actor_references.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    for actor_reference in &event.actor_references {
        require_nonempty(actor_reference)?;
    }
    let event_time =
        EventTime::parse_rfc3339(&event.event_time).map_err(|_| ApiError::InvalidWirePayload)?;
    let available_time = AvailableTime::parse_rfc3339(&event.available_time)
        .map_err(|_| ApiError::InvalidWirePayload)?;
    if available_time.instant() > cutoff {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(event_time.instant())
}

fn adjacent_relations(
    ordered: &[(TemporalContextEvent, TemporalInstant)],
) -> Vec<TemporalContextRelation> {
    ordered
        .windows(2)
        .map(|events| TemporalContextRelation {
            from_event_id: events[0].0.event_id.clone(),
            to_event_id: events[1].0.event_id.clone(),
            relation_code: "before".into(),
        })
        .collect()
}

fn adjacent_gaps(
    ordered: &[(TemporalContextEvent, TemporalInstant)],
) -> Vec<TemporalTransitionGapCandidate> {
    ordered
        .windows(2)
        .map(|events| TemporalTransitionGapCandidate {
            from_event_id: events[0].0.event_id.clone(),
            to_event_id: events[1].0.event_id.clone(),
            evidence_status_code: "candidate_not_causal".into(),
        })
        .collect()
}
