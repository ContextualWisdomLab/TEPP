//! Event-instance SQL must refuse inverted valid/system windows.

use persistence_postgres::{EventInstanceRecord, PersistenceError, insert_event_instance_sql};
use temporal_core::{AvailableTime, EventTime, SystemTime};
use uuid::Uuid;

fn clocks() -> (AvailableTime, EventTime, SystemTime) {
    (
        AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
        EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("valid"),
        SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
    )
}

fn instance() -> EventInstanceRecord {
    let (available, valid, system) = clocks();
    EventInstanceRecord {
        event_instance_id: Uuid::from_u128(1),
        tenant_record_id: Uuid::nil(),
        event_type_code: "occurrence".into(),
        valid_from: valid,
        valid_to: None,
        system_from: system,
        system_to: None,
        available_time: available,
        lifecycle_status_code: "asserted".into(),
    }
}

#[test]
fn insert_sql_renders_open_ended_instance() {
    let sql = insert_event_instance_sql(&instance()).expect("sql");
    assert!(sql.contains("INSERT INTO event_instance"));
    assert!(sql.contains("event_type_code"));
    assert!(sql.contains("lifecycle_status_code"));
    assert!(sql.contains("NULL"));
}

#[test]
fn inverted_windows_and_hostile_labels_fail_closed() {
    let mut inverted_valid = instance();
    inverted_valid.valid_to =
        Some(EventTime::parse_rfc3339("2025-12-31T00:00:00Z").expect("earlier valid"));
    assert_eq!(
        insert_event_instance_sql(&inverted_valid),
        Err(PersistenceError::InvalidEventInstance)
    );

    let mut inverted_system = instance();
    inverted_system.system_to =
        Some(SystemTime::parse_rfc3339("2025-12-31T00:00:00Z").expect("earlier system"));
    assert_eq!(
        insert_event_instance_sql(&inverted_system),
        Err(PersistenceError::InvalidEventInstance)
    );

    let mut empty_type = instance();
    empty_type.event_type_code.clear();
    assert_eq!(
        insert_event_instance_sql(&empty_type),
        Err(PersistenceError::InvalidEventInstance)
    );

    let mut hostile = instance();
    hostile.lifecycle_status_code = "asserted'; DROP TABLE".into();
    assert_eq!(
        insert_event_instance_sql(&hostile),
        Err(PersistenceError::InvalidEventInstance)
    );
    hostile.lifecycle_status_code = "asserted".into();
    hostile.event_type_code = "occurrence;role".into();
    assert_eq!(
        insert_event_instance_sql(&hostile),
        Err(PersistenceError::InvalidEventInstance)
    );
    hostile.event_type_code = "occurrence\\".into();
    assert_eq!(
        insert_event_instance_sql(&hostile),
        Err(PersistenceError::InvalidEventInstance)
    );
    hostile.event_type_code = "occurrence\nrole".into();
    assert_eq!(
        insert_event_instance_sql(&hostile),
        Err(PersistenceError::InvalidEventInstance)
    );
}

#[test]
fn equal_point_bounds_and_lookup_render() {
    let mut point = instance();
    point.valid_to = Some(point.valid_from);
    point.system_to = Some(point.system_from);
    let sql = insert_event_instance_sql(&point).expect("point");
    assert!(!sql.contains("NULL"));
    let lookup = persistence_postgres::select_event_instance_as_known_at_sql(
        Uuid::from_u128(1),
        "2026-01-01T00:00:00Z",
    );
    assert!(lookup.contains("FROM event_instance"));
    assert!(lookup.contains("system_from"));
}
