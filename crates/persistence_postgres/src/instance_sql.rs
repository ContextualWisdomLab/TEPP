//! SQL contracts for bitemporal event instances distinct from mentions.

use crate::PersistenceError;
use temporal_core::{AvailableTime, EventTime, SystemTime};
use uuid::Uuid;

/// One bitemporal event-instance version.
///
/// Maps to `event_instance`. Valid and system windows must be ordered
/// (`end` is `NULL` or `>= start`). Labels are fail-closed.
#[derive(Clone, Debug, PartialEq)]
pub struct EventInstanceRecord {
    /// Instance identity (never interchangeable with a mention).
    pub event_instance_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Closed event-type vocabulary token.
    pub event_type_code: String,
    /// Inclusive event-time start.
    pub valid_from: EventTime,
    /// Inclusive event-time end, or `None` for an open interval.
    pub valid_to: Option<EventTime>,
    /// Inclusive system-time start of this version.
    pub system_from: SystemTime,
    /// Inclusive system-time end, or `None` for the open version.
    pub system_to: Option<SystemTime>,
    /// Availability time of the instance assertion.
    pub available_time: AvailableTime,
    /// Lifecycle token (`asserted`, `revised`, `retracted`).
    pub lifecycle_status_code: String,
}

impl EventInstanceRecord {
    /// Fail-closed window order and label validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidEventInstance`] when a window is
    /// inverted or a label is empty/hostile.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        if let Some(end) = self.valid_to
            && end.instant() < self.valid_from.instant()
        {
            return Err(PersistenceError::InvalidEventInstance);
        }
        if let Some(end) = self.system_to
            && end.instant() < self.system_from.instant()
        {
            return Err(PersistenceError::InvalidEventInstance);
        }
        validate_instance_label(&self.event_type_code)?;
        validate_instance_label(&self.lifecycle_status_code)?;
        Ok(())
    }
}

/// Render insert SQL for a validated event instance version.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidEventInstance`] before any SQL is produced.
pub fn insert_event_instance_sql(record: &EventInstanceRecord) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO event_instance (\
            event_instance_id, tenant_record_id, event_type_code, \
            valid_from, valid_to, system_from, system_to, available_time, \
            lifecycle_status_code\
        ) VALUES (\
            '{instance}'::uuid, '{tenant}'::uuid, '{event_type}', \
            '{valid_from}'::timestamptz, {valid_to}, '{system_from}'::timestamptz, \
            {system_to}, '{available}'::timestamptz, '{lifecycle}'\
        )",
        instance = record.event_instance_id,
        tenant = record.tenant_record_id,
        event_type = record.event_type_code,
        valid_from = record.valid_from.to_rfc3339(),
        valid_to = optional_timestamptz(record.valid_to.map(EventTime::to_rfc3339)),
        system_from = record.system_from.to_rfc3339(),
        system_to = optional_timestamptz(record.system_to.map(SystemTime::to_rfc3339)),
        available = record.available_time.to_rfc3339(),
        lifecycle = record.lifecycle_status_code,
    ))
}

/// Render as-known-at selection for one event-instance identity.
#[must_use]
pub fn select_event_instance_as_known_at_sql(
    event_instance_id: Uuid,
    known_at_rfc3339: &str,
) -> String {
    format!(
        "SELECT event_instance_id, tenant_record_id, event_type_code, \
                valid_from, valid_to, system_from, system_to, available_time, \
                lifecycle_status_code \
         FROM event_instance \
         WHERE event_instance_id = '{event_instance_id}'::uuid \
           AND system_from <= '{known_at_rfc3339}'::timestamptz \
           AND (system_to IS NULL OR '{known_at_rfc3339}'::timestamptz < system_to) \
         ORDER BY system_from DESC \
         LIMIT 1"
    )
}

fn optional_timestamptz(value: Option<String>) -> String {
    match value {
        Some(stamp) => format!("'{stamp}'::timestamptz"),
        None => "NULL".to_owned(),
    }
}

fn validate_instance_label(value: &str) -> Result<(), PersistenceError> {
    if value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_control() || ch == '\'' || ch == ';' || ch == '\\')
    {
        return Err(PersistenceError::InvalidEventInstance);
    }
    Ok(())
}
