//! Entity and project target SQL must refuse hostile labels before INSERT.

use persistence_postgres::{
    EntityRecord, PersistenceError, ProjectRecord, insert_entity_record_sql,
    insert_project_record_sql, select_entity_record_by_id_sql, select_project_record_by_id_sql,
};
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

fn clocks() -> (AvailableTime, SystemTime) {
    (
        AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
        SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
    )
}

fn entity() -> EntityRecord {
    let (available, system) = clocks();
    EntityRecord {
        entity_record_id: Uuid::from_u128(1),
        tenant_record_id: Uuid::nil(),
        entity_type_code: "author".into(),
        system_time: system,
        available_time: available,
    }
}

fn project() -> ProjectRecord {
    let (available, system) = clocks();
    ProjectRecord {
        project_record_id: Uuid::from_u128(2),
        tenant_record_id: Uuid::nil(),
        project_status_code: "active".into(),
        system_time: system,
        available_time: available,
    }
}

#[test]
fn insert_sql_renders_typed_entity_and_project_columns() {
    let entity_sql = insert_entity_record_sql(&entity()).expect("entity");
    assert!(entity_sql.contains("INSERT INTO entity_record"));
    assert!(entity_sql.contains("entity_type_code"));
    assert!(entity_sql.contains("'author'"));
    assert!(entity_sql.contains("00000000-0000-0000-0000-000000000001"));

    let project_sql = insert_project_record_sql(&project()).expect("project");
    assert!(project_sql.contains("INSERT INTO project_record"));
    assert!(project_sql.contains("project_status_code"));
    assert!(project_sql.contains("'active'"));
    assert!(project_sql.contains("00000000-0000-0000-0000-000000000002"));
}

#[test]
fn lookup_sql_selects_by_primary_key() {
    let entity_lookup = select_entity_record_by_id_sql(Uuid::from_u128(1));
    assert!(entity_lookup.contains("FROM entity_record"));
    assert!(entity_lookup.contains("entity_record_id"));
    let project_lookup = select_project_record_by_id_sql(Uuid::from_u128(2));
    assert!(project_lookup.contains("FROM project_record"));
    assert!(project_lookup.contains("project_record_id"));
}

#[test]
fn empty_oversized_and_hostile_entity_labels_fail_closed() {
    let mut empty = entity();
    empty.entity_type_code.clear();
    assert_eq!(
        insert_entity_record_sql(&empty),
        Err(PersistenceError::InvalidEntityRecord)
    );

    let mut quoted = entity();
    quoted.entity_type_code = "author'; DROP TABLE".into();
    assert_eq!(
        insert_entity_record_sql(&quoted),
        Err(PersistenceError::InvalidEntityRecord)
    );

    let mut semicolon = entity();
    semicolon.entity_type_code = "author;role".into();
    assert_eq!(
        insert_entity_record_sql(&semicolon),
        Err(PersistenceError::InvalidEntityRecord)
    );

    let mut backslash = entity();
    backslash.entity_type_code = "author\\".into();
    assert_eq!(
        insert_entity_record_sql(&backslash),
        Err(PersistenceError::InvalidEntityRecord)
    );

    let mut control = entity();
    control.entity_type_code = "author\nrole".into();
    assert_eq!(
        insert_entity_record_sql(&control),
        Err(PersistenceError::InvalidEntityRecord)
    );

    let mut oversized = entity();
    oversized.entity_type_code = "a".repeat(129);
    assert_eq!(
        insert_entity_record_sql(&oversized),
        Err(PersistenceError::InvalidEntityRecord)
    );
}

#[test]
fn empty_oversized_and_hostile_project_labels_fail_closed() {
    let mut empty = project();
    empty.project_status_code.clear();
    assert_eq!(
        insert_project_record_sql(&empty),
        Err(PersistenceError::InvalidProjectRecord)
    );

    let mut quoted = project();
    quoted.project_status_code = "active'; DROP TABLE".into();
    assert_eq!(
        insert_project_record_sql(&quoted),
        Err(PersistenceError::InvalidProjectRecord)
    );

    let mut semicolon = project();
    semicolon.project_status_code = "active;closed".into();
    assert_eq!(
        insert_project_record_sql(&semicolon),
        Err(PersistenceError::InvalidProjectRecord)
    );

    let mut backslash = project();
    backslash.project_status_code = "active\\".into();
    assert_eq!(
        insert_project_record_sql(&backslash),
        Err(PersistenceError::InvalidProjectRecord)
    );

    let mut control = project();
    control.project_status_code = "active\nclosed".into();
    assert_eq!(
        insert_project_record_sql(&control),
        Err(PersistenceError::InvalidProjectRecord)
    );

    let mut oversized = project();
    oversized.project_status_code = "s".repeat(129);
    assert_eq!(
        insert_project_record_sql(&oversized),
        Err(PersistenceError::InvalidProjectRecord)
    );
}
