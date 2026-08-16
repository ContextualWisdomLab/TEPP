//! SQL contracts for typed project membership targets (ADR 0003 / ADR 0013).

use crate::PersistenceError;
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

/// One append-only project target that membership rows may reference.
///
/// Maps to `project_record` after migration `0006`. The status label is a
/// fail-closed contextual code (`active`, `closed`) and is not a project
/// display name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRecord {
    /// Primary key for this project identity.
    pub project_record_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Contextual project lifecycle status.
    pub project_status_code: String,
    /// System/record time when the project identity was asserted.
    pub system_time: SystemTime,
    /// Availability time of the project evidence.
    pub available_time: AvailableTime,
}

impl ProjectRecord {
    /// Fail-closed status-label validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidProjectRecord`] when the status code
    /// is empty, longer than 128 bytes, or contains control, quote, semicolon,
    /// or backslash characters.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_project_label(&self.project_status_code)
    }
}

/// Render insert SQL for a validated project record.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidProjectRecord`] before any SQL is produced.
pub fn insert_project_record_sql(record: &ProjectRecord) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO project_record (\
            project_record_id, tenant_record_id, project_status_code, \
            system_time, available_time\
        ) VALUES (\
            '{project}'::uuid, '{tenant}'::uuid, '{status}', \
            '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        project = record.project_record_id,
        tenant = record.tenant_record_id,
        status = record.project_status_code,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render selection of a project record by primary key.
#[must_use]
pub fn select_project_record_by_id_sql(project_record_id: Uuid) -> String {
    format!(
        "SELECT project_record_id, tenant_record_id, project_status_code, \
                system_time, available_time \
         FROM project_record \
         WHERE project_record_id = '{project_record_id}'::uuid \
         LIMIT 1"
    )
}

fn validate_project_label(value: &str) -> Result<(), PersistenceError> {
    if value.is_empty()
        || value.len() > 128
        || value
            .chars()
            .any(|ch| ch.is_control() || ch == '\'' || ch == ';' || ch == '\\')
    {
        return Err(PersistenceError::InvalidProjectRecord);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ProjectRecord, insert_project_record_sql, select_project_record_by_id_sql};
    use crate::PersistenceError;
    use temporal_core::{AvailableTime, SystemTime};
    use uuid::Uuid;

    fn sample() -> ProjectRecord {
        ProjectRecord {
            project_record_id: Uuid::nil(),
            tenant_record_id: Uuid::nil(),
            project_status_code: "active".into(),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    #[test]
    fn project_sql_covers_valid_and_fail_closed_paths() {
        sample().validate().expect("valid");
        let sql = insert_project_record_sql(&sample()).expect("insert");
        assert!(sql.contains("INSERT INTO project_record"));
        assert!(sql.contains("project_status_code"));
        for label in [
            String::new(),
            "active'; DROP TABLE".into(),
            "active;closed".into(),
            "active\\".into(),
            "active\nclosed".into(),
            "s".repeat(129),
        ] {
            assert_eq!(
                insert_project_record_sql(&ProjectRecord {
                    project_status_code: label,
                    ..sample()
                }),
                Err(PersistenceError::InvalidProjectRecord)
            );
        }
        assert!(select_project_record_by_id_sql(Uuid::nil()).contains("FROM project_record"));
    }
}
