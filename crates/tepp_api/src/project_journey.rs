//! Posterior Project Journey graph without a fabricated first stage.
//!
//! Events retain record time separately from TDT/CHRONOS event-time draws.
//! Directed relations are supplied as posterior draws, so multiple
//! predecessors, branches, transitions, and exact ties remain representable.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::ApiError;
use crate::wire::{from_json, require_byte_limit, to_json_with_limit};

/// Exact Project Journey posterior graph schema.
pub const PROJECT_JOURNEY_POSTERIOR_SCHEMA: &str = "tepp.project_journey_posterior.v1";
/// Maximum serialized Project Journey artifact size.
pub const DEFAULT_PROJECT_JOURNEY_BYTE_LIMIT: usize = 16 * 1024 * 1024;

/// One evidence-grounded journey event with uncertain event time.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectJourneyEventPosterior {
    /// Opaque event identity.
    pub event_id: String,
    /// Event family from the producer ontology.
    pub event_type_code: String,
    /// Record creation instant, not an event-time substitute.
    pub record_created_at: String,
    /// Instant at which this evidence became available to the analysis.
    pub available_at: String,
    /// Posterior event-time draws in the artifact's common draw order.
    pub event_time_draws: Vec<String>,
    /// Opaque authorized source-evidence identities.
    pub evidence_record_ids: Vec<String>,
}

/// One posterior temporal dependency, branch, or transition.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectJourneyRelationPosterior {
    /// Opaque relation identity.
    pub relation_id: String,
    /// Predecessor event identity.
    pub predecessor_event_id: String,
    /// Successor event identity.
    pub successor_event_id: String,
    /// Producer ontology relation code.
    pub relation_type_code: String,
    /// Relation-presence draw for every common posterior draw.
    pub relation_draws: Vec<bool>,
    /// Opaque evidence identities supporting the relation posterior.
    pub evidence_record_ids: Vec<String>,
}

/// Complete posterior Project Journey graph.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectJourneyPosteriorArtifact {
    /// Exact schema identity.
    pub schema_version: String,
    /// TEPP producer run identity.
    pub tepp_run_id: String,
    /// Immutable source snapshot digest.
    pub source_snapshot_sha256: String,
    /// Historical knowledge cutoff.
    pub knowledge_cutoff: String,
    /// Common posterior draw count.
    pub draw_count: usize,
    /// Explicit inference boundary.
    pub inference_status: String,
    /// Evidence-grounded events in stable identity order, not date order.
    pub events: Vec<ProjectJourneyEventPosterior>,
    /// Posterior temporal dependencies, branches, and transitions.
    pub relations: Vec<ProjectJourneyRelationPosterior>,
}

impl ProjectJourneyPosteriorArtifact {
    /// Parse and validate one bounded posterior journey graph.
    ///
    /// # Errors
    ///
    /// Rejects missing evidence, duplicate identities, mixed draw counts,
    /// backward transition draws, and unsupported certainty.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        require_byte_limit(payload, DEFAULT_PROJECT_JOURNEY_BYTE_LIMIT)?;
        let artifact: Self = from_json(payload)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize only a fully validated journey graph.
    ///
    /// # Errors
    ///
    /// Returns a redacted validation or resource-bound error.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json_with_limit(self, DEFAULT_PROJECT_JOURNEY_BYTE_LIMIT)
    }

    fn validate(&self) -> Result<(), ApiError> {
        if self.schema_version != PROJECT_JOURNEY_POSTERIOR_SCHEMA
            || !identifier(&self.tepp_run_id)
            || !digest(&self.source_snapshot_sha256)
            || parse_time(&self.knowledge_cutoff).is_none()
            || self.draw_count < 2
            || self.inference_status != "posterior_temporal_relation_not_causal"
            || self.events.is_empty()
        {
            return Err(ApiError::InvalidWirePayload);
        }
        let ids = self
            .events
            .iter()
            .map(|event| event.event_id.as_str())
            .collect::<BTreeSet<_>>();
        if ids.len() != self.events.len() {
            return Err(ApiError::InvalidWirePayload);
        }
        for event in &self.events {
            let available_at = parse_time(&event.available_at);
            let cutoff = parse_time(&self.knowledge_cutoff);
            if !identifier(&event.event_id)
                || !allowed_event_type(&event.event_type_code)
                || parse_time(&event.record_created_at).is_none()
                || available_at.is_none()
                || available_at > cutoff
                || event.event_time_draws.len() != self.draw_count
                || event
                    .event_time_draws
                    .iter()
                    .any(|value| parse_time(value).is_none())
                || event.evidence_record_ids.is_empty()
                || event
                    .evidence_record_ids
                    .iter()
                    .any(|value| !identifier(value))
            {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        let relation_ids = self
            .relations
            .iter()
            .map(|edge| edge.relation_id.as_str())
            .collect::<BTreeSet<_>>();
        if relation_ids.len() != self.relations.len() {
            return Err(ApiError::InvalidWirePayload);
        }
        for edge in &self.relations {
            let predecessor = self
                .events
                .iter()
                .find(|event| event.event_id == edge.predecessor_event_id);
            let successor = self
                .events
                .iter()
                .find(|event| event.event_id == edge.successor_event_id);
            if !identifier(&edge.relation_id)
                || predecessor.is_none()
                || successor.is_none()
                || edge.predecessor_event_id == edge.successor_event_id
                || !identifier(&edge.relation_type_code)
                || edge.relation_draws.len() != self.draw_count
                || edge.evidence_record_ids.is_empty()
                || edge
                    .evidence_record_ids
                    .iter()
                    .any(|value| !identifier(value))
            {
                return Err(ApiError::InvalidWirePayload);
            }
            let predecessor = predecessor.expect("checked");
            let successor = successor.expect("checked");
            for draw in 0..self.draw_count {
                if edge.relation_draws[draw]
                    && parse_time(&predecessor.event_time_draws[draw])
                        > parse_time(&successor.event_time_draws[draw])
                {
                    return Err(ApiError::InvalidWirePayload);
                }
            }
        }
        if (0..self.draw_count).any(|draw| !draw_is_acyclic(&self.events, &self.relations, draw)) {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

fn draw_is_acyclic(
    events: &[ProjectJourneyEventPosterior],
    relations: &[ProjectJourneyRelationPosterior],
    draw: usize,
) -> bool {
    let indices = events
        .iter()
        .enumerate()
        .map(|(index, event)| (event.event_id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut indegree = vec![0_usize; events.len()];
    let mut successors = vec![Vec::new(); events.len()];
    for relation in relations
        .iter()
        .filter(|relation| relation.relation_draws[draw])
    {
        let predecessor = indices[relation.predecessor_event_id.as_str()];
        let successor = indices[relation.successor_event_id.as_str()];
        successors[predecessor].push(successor);
        indegree[successor] += 1;
    }
    let mut queue = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect::<VecDeque<_>>();
    let mut visited = 0_usize;
    while let Some(node) = queue.pop_front() {
        visited += 1;
        for successor in &successors[node] {
            indegree[*successor] -= 1;
            if indegree[*successor] == 0 {
                queue.push_back(*successor);
            }
        }
    }
    visited == events.len()
}

fn identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && value.trim() == value
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn parse_time(value: &str) -> Option<Timestamp> {
    value.parse::<Timestamp>().ok()
}

fn allowed_event_type(value: &str) -> bool {
    matches!(
        value,
        "prior_project"
            | "customer_request"
            | "procurement_notice"
            | "direct_bid"
            | "negotiated_bid"
            | "external_sensing"
            | "internal_discussion"
            | "lead"
            | "design"
            | "production"
            | "delivery"
            | "trial_operation"
            | "operation"
            | "claim"
            | "rebid"
            | "other_evidence_grounded_event"
    )
}

#[cfg(test)]
mod branch_coverage_tests {
    use super::{allowed_event_type, digest, identifier, parse_time};

    #[test]
    fn guard_functions_cover_each_arm() {
        assert!(allowed_event_type("prior_project"));
        assert!(allowed_event_type("customer_request"));
        assert!(!allowed_event_type("telepathy"));
        assert!(!identifier(""));
        assert!(!identifier(&"x".repeat(257)));
        assert!(!identifier(" padded "));
        assert!(!digest(&"Z".repeat(64)));
        assert!(!digest(&"a".repeat(63)));
        assert!(digest(&"0".repeat(64)));
        assert!(digest(&"0123456789abcdef".repeat(4)));
        assert!(parse_time("2026-08-25T00:00:00Z").is_some());
        assert!(parse_time("not-a-time").is_none());
    }
}
