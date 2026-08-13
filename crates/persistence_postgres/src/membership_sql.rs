//! SQL contracts for typed multiple-membership assignments (ADR 0013 / ERD).

use crate::PersistenceError;
use temporal_core::{AvailableTime, EventTime, SystemTime};
use uuid::Uuid;

/// One append-only membership assignment with typed exactly-one keys.
///
/// Maps to `membership_assignment` after migration `0006`. Exactly one observed
/// unit (`document_record_id` or `text_segment_id`) and exactly one target
/// (`target_entity_id` or `target_project_id`) must be present.
#[derive(Clone, Debug, PartialEq)]
pub struct MembershipAssignmentRecord {
    /// Primary key for this assignment identity.
    pub membership_assignment_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Document-level observed unit, exclusive of `text_segment_id`.
    pub document_record_id: Option<Uuid>,
    /// Exact-span observed unit, exclusive of `document_record_id`.
    pub text_segment_id: Option<Uuid>,
    /// Entity membership target, exclusive of `target_project_id`.
    pub target_entity_id: Option<Uuid>,
    /// Project membership target, exclusive of `target_entity_id`.
    pub target_project_id: Option<Uuid>,
    /// Contextual membership type (author, department, customer, project role).
    pub membership_type_code: String,
    /// Positive membership weight used by multilevel estimators.
    pub membership_weight: f64,
    /// Inclusive start window; an exact start is the singleton `[t,t]`.
    pub valid_from: EventTime,
    /// Inclusive end instant, or `None` for an open-ended assignment.
    pub valid_to: Option<EventTime>,
    /// Governed precision vocabulary used to construct both windows.
    pub valid_time_precision_code: String,
    /// System/record time when the assignment was asserted.
    pub system_time: SystemTime,
    /// Availability time of the assignment evidence.
    pub available_time: AvailableTime,
}

impl MembershipAssignmentRecord {
    /// Fail-closed exactly-one, weight, and label validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidMembershipAssignment`] when the
    /// observed unit, target, weight, or labels violate the ERD contract.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        if !exactly_one(self.document_record_id, self.text_segment_id) {
            return Err(PersistenceError::InvalidMembershipAssignment);
        }
        if !exactly_one(self.target_entity_id, self.target_project_id) {
            return Err(PersistenceError::InvalidMembershipAssignment);
        }
        if !self.membership_weight.is_finite() || self.membership_weight <= 0.0 {
            return Err(PersistenceError::InvalidMembershipAssignment);
        }
        validate_membership_label(&self.membership_type_code)?;
        validate_membership_label(&self.valid_time_precision_code)?;
        Ok(())
    }
}

/// Render insert SQL for a validated membership assignment.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidMembershipAssignment`] before any SQL
/// is produced when the record fails validation.
pub fn insert_membership_assignment_sql(
    record: &MembershipAssignmentRecord,
) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO membership_assignment (\
            membership_assignment_id, tenant_record_id, document_record_id, \
            text_segment_id, target_entity_id, target_project_id, \
            membership_type_code, membership_weight, valid_from_window, \
            valid_to_window, valid_time_precision_code, system_time, available_time\
        ) VALUES (\
            '{assignment_id}'::uuid, '{tenant_id}'::uuid, {document_id}, \
            {text_segment_id}, {target_entity_id}, {target_project_id}, \
            '{type_code}', {weight}, {from_window}, \
            {to_window}, '{precision}', '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        assignment_id = record.membership_assignment_id,
        tenant_id = record.tenant_record_id,
        document_id = optional_uuid(record.document_record_id),
        text_segment_id = optional_uuid(record.text_segment_id),
        target_entity_id = optional_uuid(record.target_entity_id),
        target_project_id = optional_uuid(record.target_project_id),
        type_code = record.membership_type_code,
        weight = record.membership_weight,
        from_window = singleton_window_sql(record.valid_from),
        to_window = match record.valid_to {
            Some(end) => singleton_window_sql(end),
            None => "NULL".to_owned(),
        },
        precision = record.valid_time_precision_code,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render selection SQL for document-level assignments of one document identity.
#[must_use]
pub fn select_membership_assignments_for_document_sql(document_record_id: Uuid) -> String {
    format!(
        "SELECT membership_assignment_id FROM membership_assignment \
         WHERE document_record_id = '{document_record_id}'::uuid \
         ORDER BY membership_assignment_id"
    )
}

fn exactly_one(left: Option<Uuid>, right: Option<Uuid>) -> bool {
    left.is_some() ^ right.is_some()
}

fn optional_uuid(value: Option<Uuid>) -> String {
    match value {
        Some(id) => format!("'{id}'::uuid"),
        None => "NULL".to_owned(),
    }
}

fn singleton_window_sql(instant: EventTime) -> String {
    let stamp = instant.to_rfc3339();
    format!("'[{stamp},{stamp}]'::tstzrange")
}

fn validate_membership_label(value: &str) -> Result<(), PersistenceError> {
    if value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_control() || ch == '\'' || ch == ';')
    {
        return Err(PersistenceError::InvalidMembershipAssignment);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        MembershipAssignmentRecord, insert_membership_assignment_sql,
        select_membership_assignments_for_document_sql,
    };
    use crate::PersistenceError;
    use temporal_core::{AvailableTime, EventTime, SystemTime};
    use uuid::Uuid;

    fn times() -> (AvailableTime, EventTime, SystemTime) {
        (
            AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
            EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("valid"),
            SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
        )
    }

    fn valid_document_entity() -> MembershipAssignmentRecord {
        let (available, valid, system) = times();
        MembershipAssignmentRecord {
            membership_assignment_id: Uuid::nil(),
            tenant_record_id: Uuid::nil(),
            document_record_id: Some(Uuid::nil()),
            text_segment_id: None,
            target_entity_id: Some(Uuid::nil()),
            target_project_id: None,
            membership_type_code: "author".into(),
            membership_weight: 1.0,
            valid_from: valid,
            valid_to: Some(valid),
            valid_time_precision_code: "second".into(),
            system_time: system,
            available_time: available,
        }
    }

    #[test]
    fn insert_sql_uses_typed_columns_and_singleton_windows() {
        let sql = insert_membership_assignment_sql(&valid_document_entity()).expect("sql");
        assert!(sql.contains("INSERT INTO membership_assignment"));
        assert!(sql.contains("target_entity_id"));
        assert!(sql.contains("valid_from_window"));
        assert!(sql.contains("::tstzrange"));
        assert!(sql.contains("NULL"));
    }

    #[test]
    fn both_or_neither_keys_fail_closed() {
        let mut both_targets = valid_document_entity();
        both_targets.target_project_id = Some(Uuid::nil());
        assert_eq!(
            insert_membership_assignment_sql(&both_targets),
            Err(PersistenceError::InvalidMembershipAssignment)
        );

        let mut neither_target = valid_document_entity();
        neither_target.target_entity_id = None;
        assert_eq!(
            insert_membership_assignment_sql(&neither_target),
            Err(PersistenceError::InvalidMembershipAssignment)
        );

        let mut both_units = valid_document_entity();
        both_units.text_segment_id = Some(Uuid::nil());
        assert_eq!(
            insert_membership_assignment_sql(&both_units),
            Err(PersistenceError::InvalidMembershipAssignment)
        );

        let mut neither_unit = valid_document_entity();
        neither_unit.document_record_id = None;
        assert_eq!(
            insert_membership_assignment_sql(&neither_unit),
            Err(PersistenceError::InvalidMembershipAssignment)
        );
    }

    #[test]
    fn non_positive_weight_and_hostile_labels_fail_closed() {
        let mut weight = valid_document_entity();
        weight.membership_weight = 0.0;
        assert_eq!(
            insert_membership_assignment_sql(&weight),
            Err(PersistenceError::InvalidMembershipAssignment)
        );
        weight.membership_weight = f64::NAN;
        assert_eq!(
            insert_membership_assignment_sql(&weight),
            Err(PersistenceError::InvalidMembershipAssignment)
        );

        let mut label = valid_document_entity();
        label.membership_type_code = "author'; DROP TABLE".into();
        assert_eq!(
            insert_membership_assignment_sql(&label),
            Err(PersistenceError::InvalidMembershipAssignment)
        );
        label.membership_type_code = String::new();
        assert_eq!(
            insert_membership_assignment_sql(&label),
            Err(PersistenceError::InvalidMembershipAssignment)
        );
        label.membership_type_code = "author".into();
        label.valid_time_precision_code = String::new();
        assert_eq!(
            insert_membership_assignment_sql(&label),
            Err(PersistenceError::InvalidMembershipAssignment)
        );
    }

    #[test]
    fn open_ended_assignment_and_document_lookup_render() {
        let mut open = valid_document_entity();
        open.valid_to = None;
        open.target_entity_id = None;
        open.target_project_id = Some(Uuid::nil());
        let sql = insert_membership_assignment_sql(&open).expect("open");
        assert!(sql.contains("NULL"));
        assert!(sql.contains("target_project_id"));
        let lookup = select_membership_assignments_for_document_sql(Uuid::nil());
        assert!(lookup.contains("WHERE document_record_id"));
    }
}
