//! SQL contracts for typed event relations (ADR 0013 / ERD relation vocabulary).

use crate::PersistenceError;
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

const TRANSITION_TYPES: [&str; 8] = [
    "causes",
    "enables",
    "intervenes_on",
    "leads_to",
    "produces",
    "transitions_to",
    "input_to",
    "process_to",
];

const PROVENANCE_TYPES: [&str; 8] = [
    "references",
    "summarizes",
    "revises",
    "translates",
    "retrospectively_reports",
    "supports",
    "contradicts",
    "outcome_of",
];

/// One append-only event relation with ERD-bound transition classification.
///
/// Maps to `event_relation`. `transition_edge` must match the closed
/// transition/provenance vocabulary; unknown types fail closed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRelationRecord {
    /// Primary key for this relation identity.
    pub event_relation_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Source event identity.
    pub source_event_id: Uuid,
    /// Target event identity.
    pub target_event_id: Uuid,
    /// Closed ERD relation type code.
    pub relation_type_code: String,
    /// Whether this row is a forward state-transition edge.
    pub transition_edge: bool,
    /// System/record time when the relation was asserted.
    pub system_time: SystemTime,
    /// Availability time of the relation evidence.
    pub available_time: AvailableTime,
}

impl EventRelationRecord {
    /// Fail-closed vocabulary, flag, and self-loop validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidEventRelation`] when the type is
    /// unknown, the transition flag disagrees with the vocabulary, or a
    /// transition is a self-loop.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        let is_transition = TRANSITION_TYPES.contains(&self.relation_type_code.as_str());
        let is_provenance = PROVENANCE_TYPES.contains(&self.relation_type_code.as_str());
        if !is_transition && !is_provenance {
            return Err(PersistenceError::InvalidEventRelation);
        }
        if is_transition != self.transition_edge {
            return Err(PersistenceError::InvalidEventRelation);
        }
        if is_transition && self.source_event_id == self.target_event_id {
            return Err(PersistenceError::InvalidEventRelation);
        }
        Ok(())
    }
}

/// Render insert SQL for a validated event relation.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidEventRelation`] before any SQL is produced.
pub fn insert_event_relation_sql(record: &EventRelationRecord) -> Result<String, PersistenceError> {
    record.validate()?;
    let flag = if record.transition_edge {
        "TRUE"
    } else {
        "FALSE"
    };
    Ok(format!(
        "INSERT INTO event_relation (\
            event_relation_id, tenant_record_id, source_event_id, target_event_id, \
            relation_type_code, transition_edge, system_time, available_time\
        ) VALUES (\
            '{relation_id}'::uuid, '{tenant_id}'::uuid, '{source}'::uuid, '{target}'::uuid, \
            '{kind}', {flag}, '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        relation_id = record.event_relation_id,
        tenant_id = record.tenant_record_id,
        source = record.source_event_id,
        target = record.target_event_id,
        kind = record.relation_type_code,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}
