//! SQL contracts for exact-span `text_segment` rows (ADR 0008 / ADR 0013).

use crate::PersistenceError;
use temporal_core::{AvailableTime, KnowledgeCutoff, SystemTime};
use uuid::Uuid;

/// One append-only exact-span observation on a document.
///
/// Maps to physical `text_segment` from migration `0006`. Byte offsets are
/// half-open `[start_byte, end_byte)` over the document UTF-8 bytes.
/// `document_record_id` is required; a foreign key remains a later migration
/// (`#45` owns `0007`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextSegmentRecord {
    /// Segment identity used by membership and mention observed units.
    pub text_segment_id: Uuid,
    /// Owning tenant boundary.
    pub tenant_record_id: Uuid,
    /// Document whose UTF-8 bytes this span indexes.
    pub document_record_id: Uuid,
    /// Inclusive start offset in UTF-8 bytes; must be `>= 0`.
    pub start_byte: i64,
    /// Exclusive end offset in UTF-8 bytes; must be `> start_byte`.
    pub end_byte: i64,
    /// System/record time when the span was asserted.
    pub system_time: SystemTime,
    /// Availability time of the span evidence.
    pub available_time: AvailableTime,
}

impl TextSegmentRecord {
    /// Fail-closed half-open byte-span validation.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::InvalidTextSegment`] when `start_byte` is
    /// negative or `end_byte` is not strictly greater than `start_byte`.
    pub fn validate(&self) -> Result<(), PersistenceError> {
        if self.start_byte < 0 || self.end_byte <= self.start_byte {
            return Err(PersistenceError::InvalidTextSegment);
        }
        Ok(())
    }
}

/// Render insert SQL for a validated text segment.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidTextSegment`] before any SQL is produced.
pub fn insert_text_segment_sql(record: &TextSegmentRecord) -> Result<String, PersistenceError> {
    record.validate()?;
    Ok(format!(
        "INSERT INTO text_segment (\
            text_segment_id, tenant_record_id, document_record_id, \
            start_byte, end_byte, system_time, available_time\
        ) VALUES (\
            '{segment}'::uuid, '{tenant}'::uuid, '{document}'::uuid, \
            {start_byte}, {end_byte}, '{system}'::timestamptz, '{available}'::timestamptz\
        )",
        segment = record.text_segment_id,
        tenant = record.tenant_record_id,
        document = record.document_record_id,
        start_byte = record.start_byte,
        end_byte = record.end_byte,
        system = record.system_time.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
    ))
}

/// Render selection of one text segment by primary key.
#[must_use]
pub fn select_text_segment_by_id_sql(text_segment_id: Uuid) -> String {
    format!(
        "SELECT text_segment_id, tenant_record_id, document_record_id, \
                start_byte, end_byte, system_time, available_time \
         FROM text_segment \
         WHERE text_segment_id = '{text_segment_id}'::uuid \
         LIMIT 1"
    )
}

/// Render cutoff-eligible segments for one document identity.
///
/// Enforces `available_time <= knowledge_cutoff` so a historical fit cannot
/// consume a span that was unavailable at the declared cutoff.
#[must_use]
pub fn select_text_segments_for_document_as_of_sql(
    document_record_id: Uuid,
    knowledge_cutoff: &KnowledgeCutoff,
) -> String {
    format!(
        "SELECT text_segment_id, tenant_record_id, document_record_id, \
                start_byte, end_byte, system_time, available_time \
         FROM text_segment \
         WHERE document_record_id = '{document_record_id}'::uuid \
           AND available_time <= '{cutoff}'::timestamptz \
         ORDER BY start_byte, text_segment_id",
        cutoff = knowledge_cutoff.to_rfc3339(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        TextSegmentRecord, insert_text_segment_sql, select_text_segment_by_id_sql,
        select_text_segments_for_document_as_of_sql,
    };
    use crate::PersistenceError;
    use temporal_core::{AvailableTime, KnowledgeCutoff, SystemTime};
    use uuid::Uuid;

    fn sample() -> TextSegmentRecord {
        TextSegmentRecord {
            text_segment_id: Uuid::from_u128(1),
            tenant_record_id: Uuid::from_u128(2),
            document_record_id: Uuid::from_u128(3),
            start_byte: 0,
            end_byte: 5,
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
        }
    }

    #[test]
    fn validate_and_select_helpers_cover_local_branches() {
        let record = sample();
        record.validate().expect("valid");
        let insert = insert_text_segment_sql(&record).expect("insert");
        assert!(insert.contains("0, 5"));
        assert_eq!(
            insert_text_segment_sql(&TextSegmentRecord {
                start_byte: 0,
                end_byte: 0,
                ..record.clone()
            }),
            Err(PersistenceError::InvalidTextSegment)
        );
        let by_id = select_text_segment_by_id_sql(record.text_segment_id);
        assert!(by_id.contains("LIMIT 1"));
        let cutoff = KnowledgeCutoff::parse_rfc3339("2026-01-01T00:00:00Z").expect("c");
        let as_of = select_text_segments_for_document_as_of_sql(record.document_record_id, &cutoff);
        assert!(as_of.contains("available_time <="));
    }
}
