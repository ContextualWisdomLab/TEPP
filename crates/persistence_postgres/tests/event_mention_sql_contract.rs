//! Event-mention SQL must refuse treating a mention as an instance.

use persistence_postgres::{EventMentionRecord, PersistenceError, insert_event_mention_sql};
use temporal_core::{AvailableTime, SystemTime};
use uuid::Uuid;

fn clocks() -> (AvailableTime, SystemTime) {
    (
        AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
        SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
    )
}

fn mention(instance: Uuid, mention: Uuid, confidence: f64) -> EventMentionRecord {
    let (available, system) = clocks();
    EventMentionRecord {
        event_mention_id: mention,
        event_instance_id: instance,
        tenant_record_id: Uuid::nil(),
        confidence_score: confidence,
        system_time: system,
        available_time: available,
    }
}

#[test]
fn insert_sql_keeps_mention_and_instance_distinct() {
    let sql = insert_event_mention_sql(&mention(Uuid::from_u128(1), Uuid::from_u128(2), 0.8))
        .expect("sql");
    assert!(sql.contains("INSERT INTO event_mention"));
    assert!(sql.contains("event_instance_id"));
    assert!(sql.contains("0.8"));
}

#[test]
fn mention_as_instance_and_invalid_confidence_fail_closed() {
    let same = Uuid::from_u128(7);
    assert_eq!(
        insert_event_mention_sql(&mention(same, same, 0.8)),
        Err(PersistenceError::InvalidEventMention)
    );
    assert_eq!(
        insert_event_mention_sql(&mention(Uuid::from_u128(1), Uuid::from_u128(2), 0.0)),
        Err(PersistenceError::InvalidEventMention)
    );
    assert_eq!(
        insert_event_mention_sql(&mention(Uuid::from_u128(1), Uuid::from_u128(2), 1.01)),
        Err(PersistenceError::InvalidEventMention)
    );
    assert_eq!(
        insert_event_mention_sql(&mention(Uuid::from_u128(1), Uuid::from_u128(2), f64::NAN)),
        Err(PersistenceError::InvalidEventMention)
    );
    assert_eq!(
        insert_event_mention_sql(&mention(
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            f64::INFINITY
        )),
        Err(PersistenceError::InvalidEventMention)
    );
    assert!(
        insert_event_mention_sql(&mention(Uuid::from_u128(1), Uuid::from_u128(2), 1.0)).is_ok()
    );
}
