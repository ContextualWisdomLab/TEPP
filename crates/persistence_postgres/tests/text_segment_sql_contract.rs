//! Typed `text_segment` SQL must recover known byte spans and refuse inverted ones.

use persistence_postgres::{
    PersistenceError, TextSegmentRecord, insert_text_segment_sql, select_text_segment_by_id_sql,
    select_text_segments_for_document_as_of_sql,
};
use temporal_core::{AvailableTime, KnowledgeCutoff, SystemTime};
use uuid::Uuid;

/// UTF-8 `hello world` is 11 bytes; the true `hello` span is `[0, 5)`.
const TRUTH_START_BYTE: i64 = 0;
const TRUTH_END_BYTE: i64 = 5;
const DOCUMENT_UTF8: &str = "hello world";

fn clocks() -> (AvailableTime, SystemTime) {
    (
        AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
        SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
    )
}

fn segment(start_byte: i64, end_byte: i64) -> TextSegmentRecord {
    let (available, system) = clocks();
    TextSegmentRecord {
        text_segment_id: Uuid::from_u128(7),
        tenant_record_id: Uuid::from_u128(3),
        document_record_id: Uuid::from_u128(11),
        start_byte,
        end_byte,
        system_time: system,
        available_time: available,
    }
}

#[test]
fn insert_sql_recovers_the_known_hello_byte_span() {
    assert_eq!(DOCUMENT_UTF8.len(), 11);
    assert_eq!(&DOCUMENT_UTF8.as_bytes()[0..5], b"hello");
    let sql = insert_text_segment_sql(&segment(TRUTH_START_BYTE, TRUTH_END_BYTE)).expect("sql");
    assert!(sql.contains("INSERT INTO text_segment"));
    assert!(sql.contains("start_byte"));
    assert!(sql.contains("end_byte"));
    assert!(
        sql.contains(&format!("{TRUTH_START_BYTE}, {TRUTH_END_BYTE}")),
        "rendered SQL must carry the known-truth span: {sql}"
    );
    assert!(
        !sql.contains("byte_start"),
        "physical 0006 column is start_byte"
    );
}

#[test]
fn inverted_empty_and_negative_spans_fail_closed_before_sql() {
    assert_eq!(
        insert_text_segment_sql(&segment(5, 5)),
        Err(PersistenceError::InvalidTextSegment)
    );
    assert_eq!(
        insert_text_segment_sql(&segment(5, 4)),
        Err(PersistenceError::InvalidTextSegment)
    );
    assert_eq!(
        insert_text_segment_sql(&segment(-1, 4)),
        Err(PersistenceError::InvalidTextSegment)
    );
}

#[test]
fn cutoff_select_binds_available_time_and_document_identity() {
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff");
    let document = Uuid::from_u128(11);
    let sql = select_text_segments_for_document_as_of_sql(document, &cutoff);
    assert!(sql.contains("FROM text_segment"));
    assert!(sql.contains(&format!("document_record_id = '{document}'::uuid")));
    assert!(sql.contains("available_time <= '2026-02-01T00:00:00Z'::timestamptz"));
    let by_id = select_text_segment_by_id_sql(Uuid::from_u128(7));
    assert!(by_id.contains("text_segment_id = '00000000-0000-0000-0000-000000000007'::uuid"));
}
