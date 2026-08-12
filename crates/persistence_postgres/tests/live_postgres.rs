//! Live `PostgreSQL` integration for the optional `live-sqlx` driver.
//!
//! Default and offline CI stay free of a database process. Exact-head live
//! evidence is produced only when `TEPP_LIVE_POSTGRES=1` and a validated
//! `DATABASE_URL` point at a reachable server (see the `live-postgres` CI job).

#![cfg(feature = "live-sqlx")]

use persistence_postgres::{
    AuditEvent, DocumentRecord, LiveDocumentRepository, LiveSqlxPoolOptions, MigrationCatalog,
    ReproducibilityManifestRecord, SqlSession, apply_sql_batch, assume_app_runtime_role_sql,
    clear_session_tenant_sql, open_live_sqlx_pool, require_live_sqlx_config,
    reset_app_runtime_role_sql, set_session_tenant_sql,
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
    // Single connection so SET ROLE / tenant GUC session state survives between
    // SqlSession::execute calls (pool acquire must reuse the same backend session).
    let options = LiveSqlxPoolOptions::new(1, 5_000).expect("pool options");
    let pool = open_live_sqlx_pool(&config, options)
        .expect("live-sqlx pool must open against the CI PostgreSQL service");
    assert!(pool.is_live());

    let mut repo = LiveDocumentRepository::new(pool);
    repo.session_mut()
        .execute("SELECT 1")
        .expect("SELECT 1 through live transport");

    let catalog = MigrationCatalog::from_embedded().expect("embedded foundation catalog");
    // Best-effort reset: empty service DBs lack tables/role; re-runs clean residual objects.
    let _ = apply_sql_batch(repo.session_mut(), catalog.down_sql());
    let _ = repo
        .session_mut()
        .execute("DROP ROLE IF EXISTS tepp_app_runtime");
    let applied = repo
        .apply_migrations(&catalog)
        .expect("foundation+RLS migrations must apply on live PostgreSQL");
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

    // Owner inserts a reproducibility manifest for the same tenant, then proves
    // digest lookup SQL remains executable on the live transport.
    let manifest = ReproducibilityManifestRecord {
        reproducibility_manifest_id: Uuid::now_v7(),
        tenant_record_id,
        knowledge_cutoff: available,
        evidence_digest: content_digest,
        code_commit_sha: "a".repeat(40),
        dependency_lock_digest: "b".repeat(32),
        system_time: SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("sys"),
        available_time: available,
    };
    repo.insert_reproducibility_manifest(&manifest)
        .expect("insert reproducibility_manifest");
    repo.submit_reproducibility_manifest_by_digests(
        &manifest.evidence_digest,
        &manifest.code_commit_sha,
        &manifest.dependency_lock_digest,
    )
    .expect("select reproducibility_manifest by digests");
    repo.submit_reproducibility_manifest_by_id(manifest.reproducibility_manifest_id)
        .expect("select reproducibility_manifest by id");

    prove_tenant_rls_isolation(&mut repo);
}

/// Superuser seeds two tenants; `tepp_app_runtime` must only see the bound tenant.
fn prove_tenant_rls_isolation(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
) {
    let tenant_a = Uuid::now_v7();
    let tenant_b = Uuid::now_v7();
    let artifact_a = Uuid::now_v7();
    let artifact_b = Uuid::now_v7();
    let digest_a = "11".repeat(32);
    let digest_b = "22".repeat(32);

    seed_tenant_and_artifact(repo, tenant_a, artifact_a, &digest_a);
    seed_tenant_and_artifact(repo, tenant_b, artifact_b, &digest_b);

    repo.session_mut()
        .execute(&assume_app_runtime_role_sql())
        .expect("SET ROLE tepp_app_runtime");
    repo.session_mut()
        .execute(&set_session_tenant_sql(tenant_a))
        .expect("bind tenant A session GUC");

    repo.session_mut()
        .execute(&format!(
            "SELECT 1 FROM source_artifact WHERE source_artifact_id = '{artifact_a}'::uuid \
             AND content_sha256 = '{digest_a}'"
        ))
        .expect("tenant A must read own source_artifact under RLS");

    repo.session_mut()
        .execute(&format!(
            "DO $tepp$ BEGIN \
               IF EXISTS ( \
                 SELECT 1 FROM source_artifact WHERE source_artifact_id = '{artifact_b}'::uuid \
               ) THEN \
                 RAISE EXCEPTION 'cross-tenant source_artifact visible under RLS'; \
               END IF; \
             END $tepp$"
        ))
        .expect("tenant A must not read tenant B source_artifact");

    repo.session_mut()
        .execute(&set_session_tenant_sql(tenant_b))
        .expect("bind tenant B session GUC");
    repo.session_mut()
        .execute(&format!(
            "DO $tepp$ BEGIN \
               IF EXISTS ( \
                 SELECT 1 FROM source_artifact WHERE source_artifact_id = '{artifact_a}'::uuid \
               ) THEN \
                 RAISE EXCEPTION 'cross-tenant source_artifact visible under RLS'; \
               END IF; \
             END $tepp$"
        ))
        .expect("tenant B must not read tenant A source_artifact");
    repo.session_mut()
        .execute(&format!(
            "SELECT 1 FROM source_artifact WHERE source_artifact_id = '{artifact_b}'::uuid \
             AND content_sha256 = '{digest_b}'"
        ))
        .expect("tenant B must read own source_artifact under RLS");

    repo.session_mut()
        .execute(&clear_session_tenant_sql())
        .expect("clear tenant GUC");
    repo.session_mut()
        .execute(
            "DO $tepp$ BEGIN \
               IF EXISTS (SELECT 1 FROM source_artifact LIMIT 1) THEN \
                 RAISE EXCEPTION 'unset tenant GUC must hide all source_artifact rows'; \
               END IF; \
             END $tepp$",
        )
        .expect("cleared tenant GUC must deny all rows");

    repo.session_mut()
        .execute(&reset_app_runtime_role_sql())
        .expect("RESET ROLE");
}
