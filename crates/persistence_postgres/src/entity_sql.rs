//! SQL contracts for typed entity membership targets (ADR 0003 / ADR 0013).

use crate::PersistenceError;
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

/// One append-only entity target that membership rows may reference.
///
/// Maps to `entity_record` after migration `0006`. The type label is a
/// fail-closed ASCII snake-case contextual code (`author`, `department`,
/// `customer`) and is not a direct identity string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityRecord {
    /// Primary key for this entity identity.
    pub entity_record_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Contextual entity type used by multiple-membership estimators.
    pub entity_type_code: String,
    /// System/record time when the entity identity was asserted.
    pub system_time: SystemTime,
    /// Availability time of the entity evidence.
    pub available_time: AvailableTime,
}

impl EntityRecord {
    /// Fail-closed type-label validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidEntityRecord`] when the type code is
    /// empty, longer than 128 bytes, or contains a character outside the ASCII
    /// letters, digits, and underscore allowlist used by the rendered SQL
    /// transport.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        validate_entity_label(&self.entity_type_code)
    }
}

/// Render insert SQL for a validated entity record.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidEntityRecord`] before any SQL is produced.
pub fn insert_entity_record_sql(record: &EntityRecord) -> Result<String, PersistenceError> {
    record.validate()?;
    // The current SqlSession contract accepts rendered SQL, so the label is
    // restricted to an SQL-literal-safe identifier token before interpolation.
    Ok(format!(
        "INSERT INTO entity_record (\
            entity_record_id, tenant_record_id, entity_type_code, \
            system_time, available_time\
        ) VALUES (\
            '{entity}'::uuid, '{tenant}'::uuid, '{type_code}', \
            '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        entity = record.entity_record_id,
        tenant = record.tenant_record_id,
        type_code = record.entity_type_code,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render selection of an entity record by primary key.
#[must_use]
pub fn select_entity_record_by_id_sql(entity_record_id: Uuid) -> String {
    format!(
        "SELECT entity_record_id, tenant_record_id, entity_type_code, \
                system_time, available_time \
         FROM entity_record \
         WHERE entity_record_id = '{entity_record_id}'::uuid \
         LIMIT 1"
    )
}

fn validate_entity_label(value: &str) -> Result<(), PersistenceError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(PersistenceError::InvalidEntityRecord);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{EntityRecord, insert_entity_record_sql, select_entity_record_by_id_sql};
    use crate::PersistenceError;
    use temporal_core::{AvailableTime, SystemTime};
    use uuid::Uuid;

    fn sample() -> EntityRecord {
        EntityRecord {
            entity_record_id: Uuid::nil(),
            tenant_record_id: Uuid::nil(),
            entity_type_code: "author".into(),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    #[test]
    fn entity_sql_covers_valid_and_fail_closed_paths() {
        sample().validate().expect("valid");
        let sql = insert_entity_record_sql(&sample()).expect("insert");
        assert!(sql.contains("INSERT INTO entity_record"));
        assert!(sql.contains("entity_type_code"));
        for label in [
            String::new(),
            "author'; DROP TABLE".into(),
            "author;role".into(),
            "author\\".into(),
            "author\nrole".into(),
            "a".repeat(129),
        ] {
            assert_eq!(
                insert_entity_record_sql(&EntityRecord {
                    entity_type_code: label,
                    ..sample()
                }),
                Err(PersistenceError::InvalidEntityRecord)
            );
        }
        assert!(select_entity_record_by_id_sql(Uuid::nil()).contains("FROM entity_record"));
    }
}
