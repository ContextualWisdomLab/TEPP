//! Event-relation SQL must bind ERD transition vocabulary to `transition_edge`.

use persistence_postgres::{EventRelationRecord, PersistenceError, insert_event_relation_sql};
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

fn clocks() -> (AvailableTime, SystemTime) {
    (
        AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
        SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
    )
}

fn base_record(relation_type_code: &str, transition_edge: bool) -> EventRelationRecord {
    let (available, system) = clocks();
    EventRelationRecord {
        event_relation_id: Uuid::nil(),
        tenant_record_id: Uuid::nil(),
        source_event_id: Uuid::from_u128(1),
        target_event_id: Uuid::from_u128(2),
        relation_type_code: relation_type_code.into(),
        transition_edge,
        system_time: system,
        available_time: available,
    }
}

#[test]
fn forward_transition_inserts_and_provenance_is_not_a_transition() {
    let sql = insert_event_relation_sql(&base_record("causes", true)).expect("causes");
    assert!(sql.contains("INSERT INTO event_relation"));
    assert!(sql.contains("transition_edge"));
    assert!(sql.contains("TRUE"));

    let sql = insert_event_relation_sql(&base_record("references", false)).expect("references");
    assert!(sql.contains("FALSE"));
}

#[test]
fn mismatched_transition_flag_and_unknown_type_fail_closed() {
    assert_eq!(
        insert_event_relation_sql(&base_record("causes", false)),
        Err(PersistenceError::InvalidEventRelation)
    );
    assert_eq!(
        insert_event_relation_sql(&base_record("references", true)),
        Err(PersistenceError::InvalidEventRelation)
    );
    assert_eq!(
        insert_event_relation_sql(&base_record("invented_link", true)),
        Err(PersistenceError::InvalidEventRelation)
    );
}

#[test]
fn provenance_self_loop_is_allowed() {
    let mut record = base_record("references", false);
    record.target_event_id = record.source_event_id;

    let sql = insert_event_relation_sql(&record).expect("provenance self-loop must remain valid");
    assert!(sql.contains("references"));
    assert!(sql.contains("FALSE"));
}

#[test]
fn transition_self_loop_fails_closed() {
    let mut record = base_record("produces", true);
    record.target_event_id = record.source_event_id;
    assert_eq!(
        insert_event_relation_sql(&record),
        Err(PersistenceError::InvalidEventRelation)
    );
}
