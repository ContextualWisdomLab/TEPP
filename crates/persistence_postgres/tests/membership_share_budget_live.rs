//! Live PostgreSQL contracts for the Membership owner's same-role share budget.

#![cfg(feature = "live-sqlx")]

use persistence_postgres::{
    LiveDocumentRepository, LiveSqlxPoolOptions, MigrationCatalog, SqlSession, apply_sql_batch,
    open_live_sqlx_pool, require_live_sqlx_config,
};
use std::sync::{Arc, Barrier};
use std::thread;
use uuid::Uuid;

const LIVE_GATE_ENV: &str = "TEPP_LIVE_POSTGRES";

fn live_postgres_requested() -> bool {
    std::env::var(LIVE_GATE_ENV).is_ok_and(|value| value == "1")
}

#[test]
fn live_postgres_preserves_exact_same_role_share_budget() {
    if !live_postgres_requested() {
        return;
    }

    let config = require_live_sqlx_config()
        .expect("DATABASE_URL must be valid when TEPP_LIVE_POSTGRES=1");
    let options = LiveSqlxPoolOptions::new(1, 5_000).expect("pool options");
    let pool = open_live_sqlx_pool(&config, options).expect("open live PostgreSQL pool");
    let mut repo = LiveDocumentRepository::new(pool);
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");

    let _ = apply_sql_batch(repo.session_mut(), catalog.down_sql());
    let _ = repo
        .session_mut()
        .execute("DROP ROLE IF EXISTS tepp_app_runtime");
    repo.apply_migrations(&catalog)
        .expect("membership budget migration catalog must apply");

    let tenant_record_id = Uuid::now_v7();
    let entity_a = Uuid::now_v7();
    let entity_b = Uuid::now_v7();
    seed_tenant_and_entities(&mut repo, tenant_record_id, &[entity_a, entity_b]);

    let overrun_document = Uuid::now_v7();
    repo.session_mut()
        .execute(&membership_insert_sql(
            tenant_record_id,
            overrun_document,
            entity_a,
            Uuid::now_v7(),
            "department",
            "0.75",
            "2026-01-01",
            None,
        ))
        .expect("first same-role share");
    assert!(
        repo.session_mut()
            .execute(&membership_insert_sql(
                tenant_record_id,
                overrun_document,
                entity_b,
                Uuid::now_v7(),
                "department",
                "0.5",
                "2026-01-01",
                None,
            ))
            .is_err(),
        "overlapping same-role shares above unity must fail closed"
    );
    assert_membership_count(&mut repo, overrun_document, "department", 1);

    let unity_document = Uuid::now_v7();
    for (entity, weight) in [(entity_a, "0.75"), (entity_b, "0.25")] {
        repo.session_mut()
            .execute(&membership_insert_sql(
                tenant_record_id,
                unity_document,
                entity,
                Uuid::now_v7(),
                "department",
                weight,
                "2026-01-01",
                None,
            ))
            .expect("exact-unity same-role shares remain valid");
    }

    let different_role_document = Uuid::now_v7();
    for (entity, role) in [(entity_a, "department"), (entity_b, "project")] {
        repo.session_mut()
            .execute(&membership_insert_sql(
                tenant_record_id,
                different_role_document,
                entity,
                Uuid::now_v7(),
                role,
                "1",
                "2026-01-01",
                None,
            ))
            .expect("different classification roles have independent budgets");
    }

    let disjoint_document = Uuid::now_v7();
    repo.session_mut()
        .execute(&membership_insert_sql(
            tenant_record_id,
            disjoint_document,
            entity_a,
            Uuid::now_v7(),
            "department",
            "1",
            "2026-01-01",
            Some("2026-01-10"),
        ))
        .expect("first event-time spell");
    repo.session_mut()
        .execute(&membership_insert_sql(
            tenant_record_id,
            disjoint_document,
            entity_b,
            Uuid::now_v7(),
            "department",
            "1",
            "2026-01-11",
            Some("2026-01-20"),
        ))
        .expect("disjoint event-time spell has an independent budget");

    let binary64_boundary_document = Uuid::now_v7();
    for (index, weight) in [
        "0.9734628667233794",
        "0.0038851022715484258",
        "0.02265203100507213",
    ]
    .into_iter()
    .enumerate()
    {
        let result = repo.session_mut().execute(&membership_insert_sql(
            tenant_record_id,
            binary64_boundary_document,
            if index == 1 { entity_b } else { entity_a },
            Uuid::now_v7(),
            "department",
            weight,
            "2026-01-01",
            None,
        ));
        if index < 2 {
            result.expect("prefix of binary64 counterexample remains admissible");
        } else {
            assert!(
                result.is_err(),
                "exact binary64 overrun must not be hidden by decimal NUMERIC summation"
            );
        }
    }
    assert_membership_count(
        &mut repo,
        binary64_boundary_document,
        "department",
        2,
    );

    assert!(
        repo.session_mut()
            .execute(&membership_insert_sql(
                tenant_record_id,
                Uuid::now_v7(),
                entity_a,
                Uuid::now_v7(),
                "department",
                "1e-10000",
                "2026-01-01",
                None,
            ))
            .is_err(),
        "positive NUMERIC values that underflow binary64 must not become zero-share affiliations"
    );
}

#[test]
fn concurrent_same_role_first_writers_cannot_commit_an_overrun() {
    if !live_postgres_requested() {
        return;
    }

    let config = require_live_sqlx_config()
        .expect("DATABASE_URL must be valid when TEPP_LIVE_POSTGRES=1");
    let options = LiveSqlxPoolOptions::new(1, 5_000).expect("pool options");
    let pool = open_live_sqlx_pool(&config, options).expect("open live PostgreSQL pool");
    let mut repo = LiveDocumentRepository::new(pool);
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");

    let _ = apply_sql_batch(repo.session_mut(), catalog.down_sql());
    let _ = repo
        .session_mut()
        .execute("DROP ROLE IF EXISTS tepp_app_runtime");
    repo.apply_migrations(&catalog)
        .expect("membership budget migration catalog must apply");

    let tenant_record_id = Uuid::now_v7();
    let entity_a = Uuid::now_v7();
    let entity_b = Uuid::now_v7();
    let document_record_id = Uuid::now_v7();
    seed_tenant_and_entities(&mut repo, tenant_record_id, &[entity_a, entity_b]);

    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = [entity_a, entity_b]
        .into_iter()
        .map(|entity_record_id| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let config = require_live_sqlx_config().expect("thread live PostgreSQL config");
                let options = LiveSqlxPoolOptions::new(1, 5_000).expect("thread pool options");
                let mut pool = open_live_sqlx_pool(&config, options).expect("thread pool");
                let sql = membership_insert_sql(
                    tenant_record_id,
                    document_record_id,
                    entity_record_id,
                    Uuid::now_v7(),
                    "department",
                    "0.6",
                    "2026-01-01",
                    None,
                );
                barrier.wait();
                pool.execute(&sql).is_ok()
            })
        })
        .collect();
    let committed = handles
        .into_iter()
        .map(|handle| handle.join().expect("writer thread"))
        .filter(|&ok| ok)
        .count();
    assert_eq!(committed, 1, "only one 0.6 first writer may commit");
    assert_membership_count(&mut repo, document_record_id, "department", 1);
}

fn seed_tenant_and_entities(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    entity_record_ids: &[Uuid],
) {
    repo.session_mut()
        .execute(&format!(
            "INSERT INTO tenant_record (tenant_record_id, tenant_status_code, system_time) \
             VALUES ('{tenant_record_id}'::uuid, 'active', '2026-01-01T00:00:00Z'::timestamptz)"
        ))
        .expect("seed tenant");
    for entity_record_id in entity_record_ids {
        repo.session_mut()
            .execute(&format!(
                "INSERT INTO entity_record (entity_record_id, tenant_record_id, entity_type_code, system_time, available_time) \
                 VALUES ('{entity_record_id}'::uuid, '{tenant_record_id}'::uuid, 'membership_group', \
                         '2026-01-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz)"
            ))
            .expect("seed membership target");
    }
}

fn assert_membership_count(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    document_record_id: Uuid,
    role: &str,
    expected: i64,
) {
    repo.session_mut()
        .execute(&format!(
            "DO $tepp_membership_budget$ BEGIN \
             IF (SELECT COUNT(*) FROM membership_assignment \
                 WHERE document_record_id = '{document_record_id}'::uuid \
                   AND membership_type_code = '{role}') <> {expected} THEN \
               RAISE EXCEPTION 'unexpected membership budget row count'; \
             END IF; \
             END $tepp_membership_budget$"
        ))
        .expect("membership row count");
}

fn membership_insert_sql(
    tenant_record_id: Uuid,
    document_record_id: Uuid,
    entity_record_id: Uuid,
    membership_assignment_id: Uuid,
    role: &str,
    weight: &str,
    valid_from: &str,
    valid_to: Option<&str>,
) -> String {
    let to_window = valid_to.map_or_else(
        || "NULL".to_owned(),
        |end| format!("'[{end},{end}]'::tstzrange"),
    );
    format!(
        "INSERT INTO membership_assignment (\
            membership_assignment_id, tenant_record_id, document_record_id, text_segment_id, \
            target_entity_id, target_project_id, membership_type_code, membership_weight, \
            valid_from_window, valid_to_window, valid_time_precision_code, system_time, available_time\
         ) VALUES (\
            '{membership_assignment_id}'::uuid, '{tenant_record_id}'::uuid, \
            '{document_record_id}'::uuid, NULL, '{entity_record_id}'::uuid, NULL, \
            '{role}', {weight}, '[{valid_from},{valid_from}]'::tstzrange, {to_window}, 'second', \
            '2026-01-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz\
         )"
    )
}
