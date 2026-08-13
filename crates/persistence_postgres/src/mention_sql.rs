//! SQL contracts for event mentions distinct from event instances.

use crate::PersistenceError;
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

/// One append-only event mention that cannot be treated as an instance.
///
/// Maps to `event_mention`. `event_mention_id` must differ from
/// `event_instance_id`; confidence must be finite and in `(0, 1]`.
#[derive(Clone, Debug, PartialEq)]
pub struct EventMentionRecord {
    /// Mention identity (never interchangeable with the instance).
    pub event_mention_id: Uuid,
    /// Promoted instance this mention supports.
    pub event_instance_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Mention confidence in `(0, 1]`.
    pub confidence_score: f64,
    /// System/record time when the mention was asserted.
    pub system_time: SystemTime,
    /// Availability time of the mention evidence.
    pub available_time: AvailableTime,
}

impl EventMentionRecord {
    /// Fail-closed mention/instance separation and confidence validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidEventMention`] when the mention is
    /// the instance identity or the confidence is not in `(0, 1]`.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        if self.event_mention_id == self.event_instance_id {
            return Err(PersistenceError::InvalidEventMention);
        }
        if !self.confidence_score.is_finite()
            || self.confidence_score <= 0.0
            || self.confidence_score > 1.0
        {
            return Err(PersistenceError::InvalidEventMention);
        }
        Ok(())
    }
}

/// Render insert SQL for a validated event mention.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidEventMention`] before any SQL is produced.
pub fn insert_event_mention_sql(record: &EventMentionRecord) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO event_mention (\
            event_mention_id, event_instance_id, tenant_record_id, \
            confidence_score, system_time, available_time\
        ) VALUES (\
            '{mention}'::uuid, '{instance}'::uuid, '{tenant}'::uuid, \
            {confidence}, '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        mention = record.event_mention_id,
        instance = record.event_instance_id,
        tenant = record.tenant_record_id,
        confidence = record.confidence_score,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}
