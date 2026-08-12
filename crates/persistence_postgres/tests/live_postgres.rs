//! Live `PostgreSQL` integration for the optional `live-sqlx` driver.
//!
//! Default and offline CI stay free of a database process. Exact-head live
//! evidence is produced only when `TEPP_LIVE_POSTGRES=1` and a validated
//! `DATABASE_URL` point at a reachable server (see the `live-postgres` CI job).

#![cfg(feature = "live-sqlx")]

use persistence_postgres::{
    AuditEvent, DocumentRecord, LiveDocumentRepository, LiveSqlxPoolOptions, MigrationCatalog,
    SqlSession, apply_sql_batch, open_live_sqlx_pool, require_live_sqlx_config,
};
use temporal_core::{AvailableTime, EventTime, SystemTime};
use uuid::Uuid;

const LIVE_GATE_ENV: &str = "TEPP_LIVE_POSTGRES";

fn live_postgres_requested() -> bool {
    match std::env::var(LIVE_GATE_ENV) {
        Ok(value) => value == "1",
        Err(_) => false,
    }
}

fn sample_times() -> (AvailableTime, EventTime, SystemTime) {
    (
        AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available"),
        EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("valid"),
        SystemTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("system"),
    )
}

fn seed_tenant_and_artifact(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    source_artifact_id: Uuid,
    content_digest: &str,
) {
    let (available, _valid, system) = sample_times();
    let tenant_sql = format!(
        "INSERT INTO tenant_record (tenant_record_id, tenant_status_code, system_time) \
         VALUES ('{tenant_record_id}'::uuid, 'active', '{system}'::timestamptz)",
        system = system.to_rfc3339(),
    );
    repo.session_mut()
        .execute(&tenant_sql)
        .expect("insert tenant_record");

    let artifact_sql = format!(
        "INSERT INTO source_artifact (\
            source_artifact_id, tenant_record_id, content_sha256, source_size_bytes, \
            media_type_code, protected_object_ref, system_time, available_time\
         ) VALUES (\
            '{source_artifact_id}'::uuid, '{tenant_record_id}'::uuid, '{content_digest}', 4, \
            'text/plain', NULL, '{system}'::timestamptz, '{available}'::timestamptz\
         )",
        system = system.to_rfc3339(),
        available = available.to_rfc3339(),
    );
    repo.session_mut()
        .execute(&artifact_sql)
        .expect("insert source_artifact");
}

#[test]
fn live_postgres_applies_migrations_and_document_sql() {
    if !live_postgres_requested() {
        // Offline default CI and local unit lanes stay database-free.
        return;
    }

    let config = require_live_sqlx_config().expect(
        "DATABASE_URL must be set and valid when TEPP_LIVE_POSTGRES=1 (live Postgres CI gate)",
    );
    let options = LiveSqlxPoolOptions::new(2, 5_000).expect("pool options");
    let pool = open_live_sqlx_pool(&config, options)
        .expect("live-sqlx pool must open against the CI PostgreSQL service");
    assert!(pool.is_live());

    let mut repo = LiveDocumentRepository::new(pool);
    repo.session_mut()
        .execute("SELECT 1")
        .expect("SELECT 1 through live transport");

    let catalog = MigrationCatalog::from_embedded().expect("embedded foundation catalog");
    // Re-run safe: down is IF EXISTS, then apply the authoritative up contract.
    apply_sql_batch(repo.session_mut(), catalog.down_sql())
        .expect("foundation down migration must apply (IF EXISTS)");
    let applied = repo
        .apply_migrations(&catalog)
        .expect("foundation migrations must apply on live PostgreSQL");
    assert!(applied >= 1);

    let tenant_record_id = Uuid::now_v7();
    let document_record_id = Uuid::now_v7();
    // document_sql contracts bind source_artifact_id to the document identity.
    let source_artifact_id = document_record_id;
    let content_digest = "ab".repeat(32);
    seed_tenant_and_artifact(
        &mut repo,
        tenant_record_id,
        source_artifact_id,
        &content_digest,
    );

    let (available, valid, system) = sample_times();
    let record = DocumentRecord {
        document_record_id,
        tenant_record_id,
        content_digest: content_digest.clone(),
        available_time: available,
        valid_from: valid,
        valid_to: None,
        system_from: system,
        system_to: None,
        revision_number: 1,
    };
    repo.insert(&record).expect("insert document_record");

    let mut revised = record.clone();
    revised.revision_number = 2;
    revised.system_from = SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("later system");
    revised.content_digest = "cd".repeat(32);
    repo.revise(&revised).expect("revise document_record");

    repo.submit_as_known_at(
        document_record_id,
        &SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("known_at"),
    )
    .expect("as-known-at select");
    repo.submit_as_valid_at(
        document_record_id,
        &EventTime::parse_rfc3339("2026-01-15T00:00:00Z").expect("valid_at"),
        &SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("known_at"),
    )
    .expect("as-valid-at select");

    let audit = AuditEvent {
        audit_event_id: Uuid::now_v7(),
        tenant_record_id,
        action_code: "live_postgres_ci".into(),
        subject_record_id: document_record_id,
        recorded_system_time: SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("audit"),
    };
    repo.append_audit(&audit).expect("append audit_event");
}
