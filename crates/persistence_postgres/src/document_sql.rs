//! Parameterized SQL contracts for bitemporal document rows.

use crate::PersistenceError;
use crate::document_store::{
    ACTION_AUDIT_EVENT_APPEND, AuditEvent, AuditSourceInspection, DocumentRecord,
};
use operational_log::{AnalyticalSubject, try_record};

/// Validate digest and render an insert for an open document version.
///
/// The rendered statement binds values as quoted `RFC 3339` / UUID literals so a
/// live `SQLx` transport can execute the same text contract after server-side
/// parameterization is layered on.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidContentDigest`] when the digest is not a
/// 64-character hexadecimal `SHA-256` string.
pub fn insert_document_sql(record: &DocumentRecord) -> Result<String, PersistenceError> {
    validate_digest(&record.content_digest)?;
    Ok(format!(
        "INSERT INTO document_record (\
            document_record_id, tenant_record_id, source_artifact_id, content_sha256, \
            language_profile_code, assertion_time, document_time, valid_from, valid_to, \
            system_from, system_to, available_time, revision_number\
        ) VALUES (\
            '{document_id}'::uuid, '{tenant_id}'::uuid, '{document_id}'::uuid, '{digest}', \
            'und', NULL, NULL, '{valid_from}'::timestamptz, {valid_to}, \
            '{system_from}'::timestamptz, NULL, '{available}'::timestamptz, {revision}\
        )",
        document_id = record.document_record_id,
        tenant_id = record.tenant_record_id,
        digest = record.content_digest,
        valid_from = record.valid_from.to_rfc3339(),
        valid_to = optional_timestamptz(record.valid_to.map(temporal_core::EventTime::to_rfc3339)),
        system_from = record.system_from.to_rfc3339(),
        available = record.available_time.to_rfc3339(),
        revision = record.revision_number,
    ))
}

/// Render the close + insert pair used when revising a document identity.
///
/// # Errors
///
/// Returns digest validation failures for the revised row.
pub fn revise_document_sqls(record: &DocumentRecord) -> Result<[String; 2], PersistenceError> {
    validate_digest(&record.content_digest)?;
    let close = format!(
        "UPDATE document_record SET system_to = '{system_from}'::timestamptz \
         WHERE document_record_id = '{document_id}'::uuid AND system_to IS NULL",
        system_from = record.system_from.to_rfc3339(),
        document_id = record.document_record_id,
    );
    let insert = insert_document_sql(record)?;
    Ok([close, insert])
}

/// Render one transactional revise that fails closed unless exactly one open row closes.
///
/// The `DO` block updates the current `system_to IS NULL` version, requires that
/// close to affect exactly one row, then inserts the successor. Concurrent
/// revisers serialize on the open-row lock; the loser raises
/// `serialization_failure` instead of leaving two open versions or a silent
/// no-op. Digest validation matches [`insert_document_sql`].
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidContentDigest`] when the digest is not a
/// 64-character hexadecimal `SHA-256` string.
pub fn revise_document_atomic_sql(record: &DocumentRecord) -> Result<String, PersistenceError> {
    let insert = insert_document_sql(record)?;
    Ok(format!(
        "DO $tepp$ \
         DECLARE closed_count integer; \
         BEGIN \
           PERFORM 1 FROM document_record \
            WHERE document_record_id = '{document_id}'::uuid AND system_to IS NULL \
            FOR UPDATE NOWAIT; \
           UPDATE document_record SET system_to = '{system_from}'::timestamptz \
            WHERE document_record_id = '{document_id}'::uuid AND system_to IS NULL; \
           GET DIAGNOSTICS closed_count = ROW_COUNT; \
           IF closed_count <> 1 THEN \
             RAISE EXCEPTION 'concurrent document revision conflict' \
               USING ERRCODE = 'serialization_failure'; \
           END IF; \
           {insert}; \
         END $tepp$",
        system_from = record.system_from.to_rfc3339(),
        document_id = record.document_record_id,
    ))
}

/// Render as-known-at selection for one document identity.
#[must_use]
pub fn as_known_at_sql(document_record_id: uuid::Uuid, known_at_rfc3339: &str) -> String {
    format!(
        "SELECT document_record_id, tenant_record_id, content_sha256, available_time, \
                valid_from, valid_to, system_from, system_to, revision_number \
         FROM document_record \
         WHERE document_record_id = '{document_record_id}'::uuid \
           AND system_from <= '{known_at_rfc3339}'::timestamptz \
           AND (system_to IS NULL OR '{known_at_rfc3339}'::timestamptz < system_to) \
         ORDER BY revision_number DESC \
         LIMIT 1"
    )
}

/// Render as-valid-at selection under a system-time as-of.
#[must_use]
pub fn as_valid_at_sql(
    document_record_id: uuid::Uuid,
    valid_at_rfc3339: &str,
    known_at_rfc3339: &str,
) -> String {
    format!(
        "SELECT document_record_id, tenant_record_id, content_sha256, available_time, \
                valid_from, valid_to, system_from, system_to, revision_number \
         FROM document_record \
         WHERE document_record_id = '{document_record_id}'::uuid \
           AND system_from <= '{known_at_rfc3339}'::timestamptz \
           AND (system_to IS NULL OR '{known_at_rfc3339}'::timestamptz < system_to) \
           AND valid_from <= '{valid_at_rfc3339}'::timestamptz \
           AND (valid_to IS NULL OR '{valid_at_rfc3339}'::timestamptz < valid_to) \
         ORDER BY revision_number DESC \
         LIMIT 1"
    )
}

/// Render append-only audit insert after `try_record` refuses forbidden payloads.
///
/// # Errors
///
/// Returns [`PersistenceError::InvalidAuditEvent`] when `action_code` is empty,
/// longer than 128 bytes, or contains a control character, `'`, `;`, or `\`.
/// Returns source-payload errors when `inspection` supplies source text, source
/// identity, or a blanket-mask grant.
pub fn append_audit_sql(
    event: &AuditEvent,
    inspection: AuditSourceInspection<'_>,
) -> Result<String, PersistenceError> {
    validate_audit_action(&event.action_code)?;
    inspect_audit_source(event, inspection)?;
    Ok(format!(
        "INSERT INTO audit_event (\
            audit_event_id, tenant_record_id, action_code, subject_record_id, recorded_system_time\
        ) VALUES (\
            '{audit_id}'::uuid, '{tenant_id}'::uuid, '{action}', '{subject}'::uuid, \
            '{recorded}'::timestamptz\
        )",
        audit_id = event.audit_event_id,
        tenant_id = event.tenant_record_id,
        action = escape_literal(&event.action_code),
        subject = event.subject_record_id,
        recorded = event.recorded_system_time.to_rfc3339(),
    ))
}

fn optional_timestamptz(value: Option<String>) -> String {
    match value {
        Some(stamp) => format!("'{stamp}'::timestamptz"),
        None => "NULL".to_owned(),
    }
}

fn escape_literal(value: &str) -> String {
    value.replace('\'', "''")
}

#[allow(clippy::cast_possible_truncation)]
fn unix_seconds(recorded: temporal_core::SystemTime) -> i64 {
    recorded.instant().as_nanosecond().div_euclid(1_000_000_000) as i64
}

fn inspect_audit_source(
    event: &AuditEvent,
    inspection: AuditSourceInspection<'_>,
) -> Result<(), PersistenceError> {
    try_record(
        ACTION_AUDIT_EVENT_APPEND,
        AnalyticalSubject::from_opaque(event.subject_record_id.as_u128()),
        unix_seconds(event.recorded_system_time),
        inspection.source_text,
        inspection.source_identity,
        inspection.blanket_mask,
    )?;
    Ok(())
}

fn validate_audit_action(value: &str) -> Result<(), PersistenceError> {
    if value.is_empty()
        || value.len() > 128
        || value
            .chars()
            .any(|ch| ch.is_control() || ch == '\'' || ch == ';' || ch == '\\')
    {
        return Err(PersistenceError::InvalidAuditEvent);
    }
    Ok(())
}

fn validate_digest(digest: &str) -> Result<(), PersistenceError> {
    let length_ok = digest.len() == 64;
    let hex_ok = digest.chars().all(|ch| ch.is_ascii_hexdigit());
    if length_ok & hex_ok {
        Ok(())
    } else {
        Err(PersistenceError::InvalidContentDigest)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        append_audit_sql, as_known_at_sql, as_valid_at_sql, escape_literal, insert_document_sql,
        optional_timestamptz, revise_document_atomic_sql, revise_document_sqls,
        validate_audit_action, validate_digest,
    };
    use crate::PersistenceError;
    use crate::document_store::{AuditEvent, AuditSourceInspection, DocumentRecord};
    use temporal_core::{AvailableTime, EventTime, SystemTime};

    fn sample_record() -> DocumentRecord {
        DocumentRecord {
            document_record_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            content_digest: "ab".repeat(32),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("a"),
            valid_from: EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("v"),
            valid_to: None,
            system_from: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
            system_to: None,
            revision_number: 1,
        }
    }

    #[test]
    fn document_sql_covers_insert_revise_and_queries() {
        let record = sample_record();
        let insert = insert_document_sql(&record).expect("insert");
        assert!(insert.contains("INSERT INTO document_record"));
        assert!(insert.contains("NULL"));

        let mut bounded = record.clone();
        bounded.valid_to = Some(EventTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("vt"));
        let insert_bounded = insert_document_sql(&bounded).expect("bounded");
        assert!(insert_bounded.contains("2026-02-01T00:00:00Z"));

        let [close, reopen] = revise_document_sqls(&record).expect("revise");
        assert!(close.contains("UPDATE document_record"));
        assert!(close.contains("system_to IS NULL"));
        assert!(reopen.contains("INSERT INTO document_record"));

        let atomic = revise_document_atomic_sql(&record).expect("atomic");
        assert!(atomic.contains("DO $tepp$"));
        assert!(atomic.contains("GET DIAGNOSTICS closed_count = ROW_COUNT"));
        assert!(atomic.contains("serialization_failure"));
        assert!(atomic.contains("INSERT INTO document_record"));
        assert_eq!(
            revise_document_atomic_sql(&DocumentRecord {
                content_digest: "nope".into(),
                ..sample_record()
            }),
            Err(PersistenceError::InvalidContentDigest)
        );

        let known = as_known_at_sql(uuid::Uuid::nil(), "2026-03-01T00:00:00Z");
        assert!(known.contains("system_from <="));
        let valid = as_valid_at_sql(
            uuid::Uuid::nil(),
            "2026-01-15T00:00:00Z",
            "2026-03-01T00:00:00Z",
        );
        assert!(valid.contains("valid_from <="));

        let audit = AuditEvent {
            audit_event_id: uuid::Uuid::nil(),
            tenant_record_id: uuid::Uuid::nil(),
            action_code: "revise".into(),
            subject_record_id: uuid::Uuid::nil(),
            recorded_system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("s"),
        };
        let audit_sql = append_audit_sql(&audit, AuditSourceInspection::CLEAR).expect("audit");
        assert!(audit_sql.contains("INSERT INTO audit_event"));
        assert!(audit_sql.contains("revise"));
        assert_eq!(
            append_audit_sql(
                &AuditEvent {
                    action_code: "revise'attempt".into(),
                    ..audit.clone()
                },
                AuditSourceInspection::CLEAR,
            ),
            Err(PersistenceError::InvalidAuditEvent)
        );
        assert_eq!(
            append_audit_sql(
                &audit,
                AuditSourceInspection {
                    source_text: Some("source"),
                    source_identity: None,
                    blanket_mask: false,
                },
            ),
            Err(PersistenceError::SourceTextNotAuditable)
        );
        assert!(validate_audit_action("revise").is_ok());
        assert!(validate_audit_action("").is_err());
        assert!(validate_audit_action("revise;x").is_err());
        assert!(validate_audit_action("revise\\").is_err());
        assert!(validate_audit_action("revise\n").is_err());
        assert!(validate_audit_action(&"x".repeat(129)).is_err());

        assert_eq!(
            insert_document_sql(&DocumentRecord {
                content_digest: "short".into(),
                ..record
            }),
            Err(PersistenceError::InvalidContentDigest)
        );
        assert_eq!(
            revise_document_sqls(&DocumentRecord {
                content_digest: "nope".into(),
                ..sample_record()
            }),
            Err(PersistenceError::InvalidContentDigest)
        );
        assert_eq!(optional_timestamptz(None), "NULL");
        assert_eq!(
            optional_timestamptz(Some("2026-01-01T00:00:00Z".into())),
            "'2026-01-01T00:00:00Z'::timestamptz"
        );
        assert_eq!(escape_literal("a'b"), "a''b");
        assert!(validate_digest(&"ff".repeat(32)).is_ok());
        assert!(validate_digest("x").is_err());
    }
}
