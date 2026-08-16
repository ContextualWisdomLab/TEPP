//! Live `PostgreSQL` integration for the optional `live-sqlx` driver.
//!
//! Default and offline CI stay free of a database process. Exact-head live
//! evidence is produced only when `TEPP_LIVE_POSTGRES=1` and a validated
//! `DATABASE_URL` point at a reachable server (see the `live-postgres` CI job).

#![cfg(feature = "live-sqlx")]

use persistence_postgres::{
    AuditEvent, CorpusSplitManifestRecord, DocumentRecord, LiveDocumentRepository,
    LiveSqlxPoolOptions, MembershipAssignmentRecord, MigrationCatalog, ModelArtifactRecord,
    ModelRunRecord, PersistenceError, ReproducibilityManifestRecord, SqlSession, TextSegmentRecord,
    apply_sql_batch, assume_app_runtime_role_sql, clear_session_tenant_sql, open_live_sqlx_pool,
    require_live_sqlx_config, reset_app_runtime_role_sql, set_session_tenant_sql,
};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff, SystemTime};
use uuid::Uuid;

const CONCURRENT_WRITERS: usize = 2;
/// Wall-clock budget for concurrent proofs; hang rather than block the live job forever.
const CONCURRENT_PROOF_TIMEOUT: Duration = Duration::from_secs(90);

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
    let (_available, _valid, system) = sample_times();
    let tenant_sql = format!(
        "INSERT INTO tenant_record (tenant_record_id, tenant_status_code, system_time) \
         VALUES ('{tenant_record_id}'::uuid, 'active', '{system}'::timestamptz)",
        system = system.to_rfc3339(),
    );
    repo.session_mut()
        .execute(&tenant_sql)
        .expect("insert tenant_record");
    seed_source_artifact(repo, tenant_record_id, source_artifact_id, content_digest);
}

fn seed_source_artifact(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    source_artifact_id: Uuid,
    content_digest: &str,
) {
    let (available, _valid, system) = sample_times();
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
    apply_sql_timeouts(&mut repo, "5s", "60s");

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
    repo.assert_restore_integrity()
        .expect("empty restored catalog must pass integrity probes");

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
        // SHA-256 hex is always 64 characters (same contract as content_digest).
        dependency_lock_digest: "b".repeat(64),
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

    exercise_model_run_artifact_chain(&mut repo, tenant_record_id, &manifest, available);
    exercise_typed_membership_assignments(&mut repo, tenant_record_id, available, system);
    prove_text_segment_known_span(&mut repo, tenant_record_id, available, system);
    prove_append_only_immutability(&mut repo, &manifest);
    prove_temporal_interval_ordering(&mut repo, tenant_record_id, source_artifact_id);
    apply_sql_timeouts(&mut repo, "3s", "30s");
    prove_concurrent_document_writes(&mut repo);
    prove_tenant_rls_isolation(&mut repo);
}

fn apply_sql_timeouts(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    lock_timeout: &str,
    statement_timeout: &str,
) {
    // Fail closed instead of hanging the live job on lock or statement stalls.
    repo.session_mut()
        .execute(&format!("SET lock_timeout = '{lock_timeout}'"))
        .expect("lock_timeout");
    repo.session_mut()
        .execute(&format!("SET statement_timeout = '{statement_timeout}'"))
        .expect("statement_timeout");
}

fn open_writer_repo() -> LiveDocumentRepository<persistence_postgres::LiveSqlxPool> {
    let config = require_live_sqlx_config().expect("DATABASE_URL");
    let options = LiveSqlxPoolOptions::new(1, 5_000).expect("writer pool");
    let pool = open_live_sqlx_pool(&config, options).expect("writer pool open");
    let mut repo = LiveDocumentRepository::new(pool);
    apply_sql_timeouts(&mut repo, "3s", "15s");
    repo
}

fn is_closed_write_failure(error: PersistenceError) -> bool {
    matches!(
        error,
        PersistenceError::DuplicateDocumentRecord
            | PersistenceError::ConcurrentWriteConflict
            | PersistenceError::SqlExecutionFailed
    )
}

fn assert_single_winner(results: Vec<Result<(), PersistenceError>>, context: &str) {
    let successes = results.iter().filter(|result| result.is_ok()).count();
    assert_eq!(successes, 1, "{context}: expected exactly one winner");
    assert!(
        results
            .into_iter()
            .filter_map(Result::err)
            .all(is_closed_write_failure),
        "{context}: losers must fail closed"
    );
}

fn document_row_guard(document_record_id: Uuid, expected_rows: u64, expected_open: u64) -> String {
    format!(
        "DO $tepp$ BEGIN \
           IF (SELECT COUNT(*) FROM document_record \
               WHERE document_record_id = '{document_record_id}'::uuid) <> {expected_rows} THEN \
             RAISE EXCEPTION 'unexpected document_record row count'; \
           END IF; \
           IF (SELECT COUNT(*) FROM document_record \
               WHERE document_record_id = '{document_record_id}'::uuid \
                 AND system_to IS NULL) <> {expected_open} THEN \
             RAISE EXCEPTION 'unexpected open document_record count'; \
           END IF; \
         END $tepp$"
    )
}

fn sample_document(
    document_record_id: Uuid,
    tenant_record_id: Uuid,
    content_digest: String,
    revision_number: u64,
    system: SystemTime,
) -> DocumentRecord {
    let (available, valid, _) = sample_times();
    DocumentRecord {
        document_record_id,
        tenant_record_id,
        content_digest,
        available_time: available,
        valid_from: valid,
        valid_to: None,
        system_from: system,
        system_to: None,
        revision_number,
    }
}

fn join_with_timeout<T: Send + 'static>(
    handle: thread::JoinHandle<T>,
    budget: Duration,
    context: &'static str,
) -> T {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(handle.join());
    });
    match rx.recv_timeout(budget) {
        Ok(Ok(value)) => value,
        Ok(Err(panic_payload)) => {
            panic!("{context}: worker thread panicked: {panic_payload:?}")
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("{context}: worker exceeded wall-clock budget")
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("{context}: worker channel disconnected")
        }
    }
}

fn race_identical_writes(
    record: &DocumentRecord,
    revise: bool,
    context: &'static str,
) -> Vec<Result<(), PersistenceError>> {
    // Open pools before the barrier so a failed open cannot leave peers waiting forever.
    let writers: Vec<_> = (0..CONCURRENT_WRITERS)
        .map(|_| open_writer_repo())
        .collect();
    let barrier = Arc::new(Barrier::new(CONCURRENT_WRITERS));
    let handles: Vec<_> = writers
        .into_iter()
        .map(|mut writer| {
            let barrier = Arc::clone(&barrier);
            let record = record.clone();
            thread::spawn(move || {
                barrier.wait();
                if revise {
                    writer.revise(&record)
                } else {
                    writer.insert(&record)
                }
            })
        })
        .collect();
    handles
        .into_iter()
        .map(|handle| join_with_timeout(handle, CONCURRENT_PROOF_TIMEOUT, context))
        .collect()
}

/// Concurrent first inserts and revises must leave exactly one open version.
fn prove_concurrent_document_writes(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
) {
    let tenant_record_id = Uuid::now_v7();
    let first_document_id = Uuid::now_v7();
    let first_digest = "a1".repeat(32);
    seed_tenant_and_artifact(repo, tenant_record_id, first_document_id, &first_digest);
    let (_, _, system) = sample_times();
    let first = sample_document(first_document_id, tenant_record_id, first_digest, 1, system);
    assert_single_winner(
        race_identical_writes(&first, false, "insert thread"),
        "concurrent first insert",
    );
    repo.session_mut()
        .execute(&document_row_guard(first_document_id, 1, 1))
        .expect("exactly one first insert row");

    let revised = sample_document(
        first_document_id,
        tenant_record_id,
        "b2".repeat(32),
        2,
        SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("later"),
    );
    assert_single_winner(
        race_identical_writes(&revised, true, "revise thread"),
        "concurrent revise",
    );
    repo.session_mut()
        .execute(&document_row_guard(first_document_id, 2, 1))
        .expect("closed first version plus one open revision");

    prove_missing_open_row_fails(repo, tenant_record_id);
    prove_distinct_concurrent_inserts(repo, tenant_record_id, system);
    prove_concurrent_append_only_reject(first_document_id);
}

fn prove_missing_open_row_fails(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
) {
    let missing = sample_document(
        Uuid::now_v7(),
        tenant_record_id,
        "c3".repeat(32),
        2,
        SystemTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("missing"),
    );
    let missing_error = repo.revise(&missing).expect_err("missing open row");
    assert!(
        is_closed_write_failure(missing_error),
        "revise without an open row must fail closed"
    );
}

fn prove_distinct_concurrent_inserts(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    system: SystemTime,
) {
    let pairs: Vec<(Uuid, String)> = (0..CONCURRENT_WRITERS)
        .map(|index| (Uuid::now_v7(), format!("{index:02x}") + &"e5".repeat(31)))
        .collect();
    for (document_record_id, digest) in &pairs {
        seed_source_artifact(repo, tenant_record_id, *document_record_id, digest);
    }
    let writers: Vec<_> = (0..CONCURRENT_WRITERS)
        .map(|_| open_writer_repo())
        .collect();
    let barrier = Arc::new(Barrier::new(CONCURRENT_WRITERS));
    let handles: Vec<_> = writers
        .into_iter()
        .zip(pairs)
        .map(|(mut writer, (document_record_id, digest))| {
            let barrier = Arc::clone(&barrier);
            let record = sample_document(document_record_id, tenant_record_id, digest, 1, system);
            thread::spawn(move || {
                barrier.wait();
                writer.insert(&record)
            })
        })
        .collect();
    for handle in handles {
        join_with_timeout(handle, CONCURRENT_PROOF_TIMEOUT, "distinct insert thread")
            .expect("independent document inserts must all succeed");
    }
}

fn prove_concurrent_append_only_reject(source_artifact_id: Uuid) {
    let artifact_update = format!(
        "UPDATE source_artifact SET media_type_code = 'text/hostile' \
         WHERE source_artifact_id = '{source_artifact_id}'::uuid"
    );
    let writers: Vec<_> = (0..CONCURRENT_WRITERS)
        .map(|_| open_writer_repo())
        .collect();
    let barrier = Arc::new(Barrier::new(CONCURRENT_WRITERS));
    let handles: Vec<_> = writers
        .into_iter()
        .map(|mut writer| {
            let barrier = Arc::clone(&barrier);
            let sql = artifact_update.clone();
            thread::spawn(move || {
                barrier.wait();
                writer.session_mut().execute(&sql)
            })
        })
        .collect();
    for handle in handles {
        assert!(
            join_with_timeout(handle, CONCURRENT_PROOF_TIMEOUT, "mutation thread").is_err(),
            "concurrent append-only UPDATE must fail"
        );
    }
}

/// Append-only triggers must reject UPDATE/DELETE on identity tables.
fn prove_append_only_immutability(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    manifest: &ReproducibilityManifestRecord,
) {
    let update = format!(
        "UPDATE reproducibility_manifest SET code_commit_sha = 'deadbeef' \
         WHERE reproducibility_manifest_id = '{}'::uuid",
        manifest.reproducibility_manifest_id
    );
    assert!(
        repo.session_mut().execute(&update).is_err(),
        "UPDATE must fail on append-only reproducibility_manifest"
    );
    let delete = format!(
        "DELETE FROM reproducibility_manifest \
         WHERE reproducibility_manifest_id = '{}'::uuid",
        manifest.reproducibility_manifest_id
    );
    assert!(
        repo.session_mut().execute(&delete).is_err(),
        "DELETE must fail on append-only reproducibility_manifest"
    );
}

/// Inverted valid windows and non-positive revisions must fail closed.
fn prove_temporal_interval_ordering(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    source_artifact_id: Uuid,
) {
    let document_record_id = Uuid::now_v7();
    let inverted = format!(
        "INSERT INTO document_record (\
            document_record_id, tenant_record_id, source_artifact_id, content_sha256, \
            language_profile_code, assertion_time, document_time, valid_from, valid_to, \
            system_from, system_to, available_time, revision_number\
         ) VALUES (\
            '{document_record_id}'::uuid, '{tenant_record_id}'::uuid, '{source_artifact_id}'::uuid, \
            '{digest}', 'und', NULL, NULL, \
            '2026-02-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz, \
            '2026-01-01T00:00:00Z'::timestamptz, NULL, \
            '2026-01-01T00:00:00Z'::timestamptz, 1\
         )",
        digest = "a".repeat(64),
    );
    assert!(
        repo.session_mut().execute(&inverted).is_err(),
        "inverted valid_from/valid_to must fail document_record_valid_order"
    );

    let bad_revision = format!(
        "INSERT INTO document_record (\
            document_record_id, tenant_record_id, source_artifact_id, content_sha256, \
            language_profile_code, assertion_time, document_time, valid_from, valid_to, \
            system_from, system_to, available_time, revision_number\
         ) VALUES (\
            '{document_record_id}'::uuid, '{tenant_record_id}'::uuid, '{source_artifact_id}'::uuid, \
            '{digest}', 'und', NULL, NULL, \
            '2026-01-01T00:00:00Z'::timestamptz, NULL, \
            '2026-01-01T00:00:00Z'::timestamptz, NULL, \
            '2026-01-01T00:00:00Z'::timestamptz, 0\
         )",
        digest = "b".repeat(64),
    );
    assert!(
        repo.session_mut().execute(&bad_revision).is_err(),
        "revision_number 0 must fail document_record_revision_positive"
    );
}

fn exercise_model_run_artifact_chain(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    manifest: &ReproducibilityManifestRecord,
    available: AvailableTime,
) {
    let system = SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("sys");
    let split = CorpusSplitManifestRecord {
        corpus_split_manifest_id: Uuid::now_v7(),
        tenant_record_id,
        split_manifest_digest: "c".repeat(64),
        knowledge_cutoff: available,
        system_time: system,
        available_time: available,
    };
    repo.insert_corpus_split_manifest(&split)
        .expect("insert corpus_split_manifest");
    let model_run = ModelRunRecord {
        model_run_id: Uuid::now_v7(),
        tenant_record_id,
        reproducibility_manifest_id: manifest.reproducibility_manifest_id,
        corpus_split_manifest_id: Some(split.corpus_split_manifest_id),
        configuration_digest: "d".repeat(64),
        random_seed_manifest_digest: "e".repeat(64),
        engine_version_label: "tepp-estimator/0.1.0".into(),
        compute_backend_code: "cpu_f64".into(),
        knowledge_cutoff: available,
        system_time: system,
        available_time: available,
    };
    repo.insert_model_run(&model_run).expect("insert model_run");
    let artifact = ModelArtifactRecord {
        model_artifact_id: Uuid::now_v7(),
        tenant_record_id,
        model_run_id: model_run.model_run_id,
        artifact_type_code: "checkpoint".into(),
        artifact_content_digest: "f".repeat(64),
        protected_object_ref: None,
        system_time: system,
        available_time: available,
    };
    repo.insert_model_artifact(&artifact)
        .expect("insert model_artifact");
    repo.submit_model_run_by_id(model_run.model_run_id)
        .expect("select model_run by id");
    repo.submit_model_artifacts_by_run(model_run.model_run_id)
        .expect("select model_artifact by run");
}

fn live_membership(
    ids: (Uuid, Uuid, Option<Uuid>, Option<Uuid>),
    membership_type_code: &str,
    membership_weight: f64,
    clocks: (AvailableTime, SystemTime),
) -> MembershipAssignmentRecord {
    let (tenant_record_id, document_record_id, target_entity_id, target_project_id) = ids;
    let (available, system) = clocks;
    let valid = EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("valid");
    MembershipAssignmentRecord {
        membership_assignment_id: Uuid::now_v7(),
        tenant_record_id,
        document_record_id: Some(document_record_id),
        text_segment_id: None,
        target_entity_id,
        target_project_id,
        membership_type_code: membership_type_code.into(),
        membership_weight,
        valid_from: valid,
        valid_to: Some(valid),
        valid_time_precision_code: "second".into(),
        system_time: system,
        available_time: available,
    }
}

fn entity_insert_sql(
    entity_id: Uuid,
    tenant_record_id: Uuid,
    entity_type: &str,
    available: AvailableTime,
    system: SystemTime,
) -> String {
    format!(
        "INSERT INTO entity_record (\
            entity_record_id, tenant_record_id, entity_type_code, \
            system_time, available_time\
         ) VALUES (\
            '{entity_id}'::uuid, '{tenant_record_id}'::uuid, '{entity_type}', \
            '{system}'::timestamptz, '{available}'::timestamptz\
         )",
        system = system.to_rfc3339(),
        available = available.to_rfc3339(),
    )
}

fn seed_membership_targets(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    entity_a: Uuid,
    entity_b: Uuid,
    project: Uuid,
    available: AvailableTime,
    system: SystemTime,
) {
    repo.session_mut()
        .execute(&assume_app_runtime_role_sql())
        .expect("SET ROLE tepp_app_runtime for membership seed");
    repo.session_mut()
        .execute(&set_session_tenant_sql(Uuid::nil()))
        .expect("bind wrong tenant GUC");
    assert!(
        repo.session_mut()
            .execute(&entity_insert_sql(
                entity_a,
                tenant_record_id,
                "author",
                available,
                system,
            ))
            .is_err(),
        "wrong tenant GUC must reject entity_record insert under FORCE RLS"
    );
    repo.session_mut()
        .execute(&set_session_tenant_sql(tenant_record_id))
        .expect("bind membership tenant GUC");
    for (entity_id, entity_type) in [(entity_a, "author"), (entity_b, "department")] {
        repo.session_mut()
            .execute(&entity_insert_sql(
                entity_id,
                tenant_record_id,
                entity_type,
                available,
                system,
            ))
            .expect("insert entity_record");
    }
    repo.session_mut()
        .execute(&format!(
            "INSERT INTO project_record (\
                project_record_id, tenant_record_id, project_status_code, \
                system_time, available_time\
             ) VALUES (\
                '{project}'::uuid, '{tenant_record_id}'::uuid, 'active', \
                '{system}'::timestamptz, '{available}'::timestamptz\
             )",
            system = system.to_rfc3339(),
            available = available.to_rfc3339(),
        ))
        .expect("insert project_record");
    repo.session_mut()
        .execute(&reset_app_runtime_role_sql())
        .expect("RESET ROLE after membership seed");
    repo.session_mut()
        .execute(&clear_session_tenant_sql())
        .expect("clear tenant GUC after membership seed");
}

fn exercise_typed_membership_assignments(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    available: AvailableTime,
    system: SystemTime,
) {
    let entity_a = Uuid::now_v7();
    let entity_b = Uuid::now_v7();
    let project = Uuid::now_v7();
    let document_record_id = Uuid::now_v7();
    seed_membership_targets(
        repo,
        tenant_record_id,
        entity_a,
        entity_b,
        project,
        available,
        system,
    );
    let clocks = (available, system);
    repo.insert_membership_assignment(&live_membership(
        (tenant_record_id, document_record_id, Some(entity_a), None),
        "author",
        1.0,
        clocks,
    ))
    .expect("insert author membership");
    repo.insert_membership_assignment(&live_membership(
        (tenant_record_id, document_record_id, Some(entity_b), None),
        "department",
        0.5,
        clocks,
    ))
    .expect("insert department membership");
    repo.insert_membership_assignment(&live_membership(
        (tenant_record_id, document_record_id, None, Some(project)),
        "project",
        1.0,
        clocks,
    ))
    .expect("insert project membership");
    repo.submit_membership_assignments_for_document(document_record_id)
        .expect("select document memberships");
    prove_persisted_membership_rows(repo, document_record_id, entity_a, entity_b, project);
    prove_membership_exactly_one_rejections(
        repo,
        tenant_record_id,
        document_record_id,
        entity_a,
        project,
        available,
        system,
    );
}

fn prove_text_segment_known_span(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    available: AvailableTime,
    system: SystemTime,
) {
    const DOCUMENT_UTF8: &str = "hello world";
    assert_eq!(DOCUMENT_UTF8.len(), 11);
    assert_eq!(&DOCUMENT_UTF8.as_bytes()[0..5], b"hello");
    let document_record_id = Uuid::now_v7();
    let hello = Uuid::now_v7();
    let world = Uuid::now_v7();
    let later = AvailableTime::parse_rfc3339("2026-06-01T00:00:00Z").expect("later");
    repo.insert_text_segment(&TextSegmentRecord {
        text_segment_id: hello,
        tenant_record_id,
        document_record_id,
        start_byte: 0,
        end_byte: 5,
        system_time: system,
        available_time: available,
    })
    .expect("insert known hello span");
    repo.insert_text_segment(&TextSegmentRecord {
        text_segment_id: world,
        tenant_record_id,
        document_record_id,
        start_byte: 6,
        end_byte: 11,
        system_time: system,
        available_time: later,
    })
    .expect("insert later world span");
    let inverted = TextSegmentRecord {
        text_segment_id: Uuid::now_v7(),
        tenant_record_id,
        document_record_id,
        start_byte: 5,
        end_byte: 0,
        system_time: system,
        available_time: available,
    };
    assert_eq!(
        repo.insert_text_segment(&inverted),
        Err(PersistenceError::InvalidTextSegment)
    );
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff");
    repo.submit_text_segment_by_id(hello)
        .expect("select text_segment by id");
    repo.submit_text_segments_for_document_as_of(document_record_id, &cutoff)
        .expect("select cutoff-eligible text_segment rows");
    let recovery = format!(
        "DO $tepp_text_segment$ BEGIN \
           IF (SELECT start_byte FROM text_segment \
               WHERE text_segment_id = '{hello}'::uuid) <> 0 THEN \
             RAISE EXCEPTION 'hello start_byte did not recover'; \
           END IF; \
           IF (SELECT end_byte FROM text_segment \
               WHERE text_segment_id = '{hello}'::uuid) <> 5 THEN \
             RAISE EXCEPTION 'hello end_byte did not recover'; \
           END IF; \
           IF (SELECT COUNT(*) FROM text_segment \
               WHERE document_record_id = '{document_record_id}'::uuid \
                 AND available_time <= '2026-02-01T00:00:00Z'::timestamptz) <> 1 THEN \
             RAISE EXCEPTION 'cutoff must keep only the available hello span'; \
           END IF; \
         END $tepp_text_segment$"
    );
    repo.session_mut()
        .execute(&recovery)
        .expect("known hello span and cutoff eligibility must recover");
}

fn prove_persisted_membership_rows(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    document_record_id: Uuid,
    entity_a: Uuid,
    entity_b: Uuid,
    project: Uuid,
) {
    let count_proof = format!(
        "DO $tepp$ BEGIN \
           IF (SELECT COUNT(*) FROM membership_assignment \
               WHERE document_record_id = '{document_record_id}'::uuid) <> 3 THEN \
             RAISE EXCEPTION 'expected three membership rows for one document'; \
           END IF; \
           IF (SELECT COUNT(*) FROM membership_assignment \
               WHERE document_record_id = '{document_record_id}'::uuid \
                 AND target_entity_id = '{entity_a}'::uuid) <> 1 THEN \
             RAISE EXCEPTION 'missing entity_a membership'; \
           END IF; \
           IF (SELECT COUNT(*) FROM membership_assignment \
               WHERE document_record_id = '{document_record_id}'::uuid \
                 AND target_entity_id = '{entity_b}'::uuid) <> 1 THEN \
             RAISE EXCEPTION 'missing entity_b membership'; \
           END IF; \
           IF (SELECT COUNT(*) FROM membership_assignment \
               WHERE document_record_id = '{document_record_id}'::uuid \
                 AND target_project_id = '{project}'::uuid) <> 1 THEN \
             RAISE EXCEPTION 'missing project membership'; \
           END IF; \
         END $tepp$"
    );
    repo.session_mut()
        .execute(&count_proof)
        .expect("one document must persist two entity and one project membership");
}

fn prove_membership_exactly_one_rejections(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    document_record_id: Uuid,
    entity_a: Uuid,
    project: Uuid,
    available: AvailableTime,
    system: SystemTime,
) {
    let dual_target = format!(
        "INSERT INTO membership_assignment (\
            membership_assignment_id, tenant_record_id, document_record_id, \
            text_segment_id, target_entity_id, target_project_id, \
            membership_type_code, membership_weight, valid_from_window, \
            valid_to_window, valid_time_precision_code, system_time, available_time\
         ) VALUES (\
            '{id}'::uuid, '{tenant_record_id}'::uuid, '{document_record_id}'::uuid, \
            NULL, '{entity_a}'::uuid, '{project}'::uuid, \
            'invalid', 1, '[2026-01-01,2026-01-01]'::tstzrange, \
            NULL, 'second', '{system}'::timestamptz, '{available}'::timestamptz\
         )",
        id = Uuid::now_v7(),
        system = system.to_rfc3339(),
        available = available.to_rfc3339(),
    );
    assert!(
        repo.session_mut().execute(&dual_target).is_err(),
        "exactly-one target check must reject dual entity+project keys"
    );

    let text_segment_id = Uuid::now_v7();
    repo.insert_text_segment(&TextSegmentRecord {
        text_segment_id,
        tenant_record_id,
        document_record_id,
        start_byte: 0,
        end_byte: 8,
        system_time: system,
        available_time: available,
    })
    .expect("insert text_segment");
    let dual_unit = format!(
        "INSERT INTO membership_assignment (\
            membership_assignment_id, tenant_record_id, document_record_id, \
            text_segment_id, target_entity_id, target_project_id, \
            membership_type_code, membership_weight, valid_from_window, \
            valid_to_window, valid_time_precision_code, system_time, available_time\
         ) VALUES (\
            '{id}'::uuid, '{tenant_record_id}'::uuid, '{document_record_id}'::uuid, \
            '{text_segment_id}'::uuid, '{entity_a}'::uuid, NULL, \
            'invalid', 1, '[2026-01-01,2026-01-01]'::tstzrange, \
            NULL, 'second', '{system}'::timestamptz, '{available}'::timestamptz\
         )",
        id = Uuid::now_v7(),
        system = system.to_rfc3339(),
        available = available.to_rfc3339(),
    );
    assert!(
        repo.session_mut().execute(&dual_unit).is_err(),
        "exactly-one observed-unit check must reject dual document+segment keys"
    );
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
