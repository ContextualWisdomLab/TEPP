//! Live PostgreSQL contracts for Membership duplicate temporal-edge refusal.

#![cfg(feature = "live-sqlx")]

use persistence_postgres::{
    LiveDocumentRepository, LiveSqlxPoolOptions, MigrationCatalog, SqlSession, apply_sql_batch,
    open_live_sqlx_pool, require_live_sqlx_config,
};
use uuid::Uuid;

const LIVE_GATE_ENV: &str = "TEPP_LIVE_POSTGRES";

fn live_postgres_requested() -> bool {
    std::env::var(LIVE_GATE_ENV).is_ok_and(|value| value == "1")
}

#[test]
fn same_target_same_role_must_be_temporally_disjoint() {
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
        .expect("0001..0010 migration catalog must apply");

    let tenant_record_id = Uuid::now_v7();
    let entity_a = Uuid::now_v7();
    let entity_b = Uuid::now_v7();
    seed_scope(&mut repo, tenant_record_id, &[entity_a, entity_b]);

    let overlapping_duplicate = Uuid::now_v7();
    insert_membership(
        &mut repo,
        tenant_record_id,
        overlapping_duplicate,
        entity_a,
        "department",
        "0.5",
        "'[2026-01-01,2026-01-01]'::tstzrange",
        "'[2026-01-31,2026-01-31]'::tstzrange",
    )
    .expect("first temporal edge");
    assert!(
        insert_membership(
            &mut repo,
            tenant_record_id,
            overlapping_duplicate,
            entity_a,
            "department",
            "0.5",
            "'[2026-01-10,2026-01-10]'::tstzrange",
            "'[2026-01-20,2026-01-20]'::tstzrange",
        )
        .is_err(),
        "same member, target, and role must reject a possible temporal duplicate even at exact-unity total share"
    );
    assert_membership_count(&mut repo, overlapping_duplicate, 1);

    let leave_reentry = Uuid::now_v7();
    insert_membership(
        &mut repo,
        tenant_record_id,
        leave_reentry,
        entity_a,
        "department",
        "1",
        "'[2026-02-01,2026-02-01]'::tstzrange",
        "'[2026-02-10,2026-02-10)'::tstzrange",
    )
    .expect("first spell");
    insert_membership(
        &mut repo,
        tenant_record_id,
        leave_reentry,
        entity_a,
        "department",
        "1",
        "'[2026-02-10,2026-02-10]'::tstzrange",
        "'[2026-02-20,2026-02-20]'::tstzrange",
    )
    .expect("strictly disjoint leave/re-entry spell remains valid");
    assert_membership_count(&mut repo, leave_reentry, 2);

    let different_targets = Uuid::now_v7();
    for entity_record_id in [entity_a, entity_b] {
        insert_membership(
            &mut repo,
            tenant_record_id,
            different_targets,
            entity_record_id,
            "department",
            "0.5",
            "'(,2026-03-05]'::tstzrange",
            "'[2026-03-10,)'::tstzrange",
        )
        .expect("different targets are multiple membership, not a duplicate edge");
    }
    assert_membership_count(&mut repo, different_targets, 2);

    let different_roles = Uuid::now_v7();
    for role in ["department", "project"] {
        insert_membership(
            &mut repo,
            tenant_record_id,
            different_roles,
            entity_a,
            role,
            "1",
            "'[2026-04-01,2026-04-01]'::tstzrange",
            "'[2026-04-30,2026-04-30]'::tstzrange",
        )
        .expect("the same target under a different classification role is a distinct edge");
    }
    assert_membership_count(&mut repo, different_roles, 2);
}

fn seed_scope(
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

fn insert_membership(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    document_record_id: Uuid,
    entity_record_id: Uuid,
    role: &str,
    weight: &str,
    valid_from_window: &str,
    valid_to_window: &str,
) -> Result<(), persistence_postgres::PersistenceError> {
    repo.session_mut().execute(&format!(
        "INSERT INTO membership_assignment (\
            membership_assignment_id, tenant_record_id, document_record_id, text_segment_id, \
            target_entity_id, target_project_id, membership_type_code, membership_weight, \
            valid_from_window, valid_to_window, valid_time_precision_code, system_time, available_time\
         ) VALUES (\
            '{}'::uuid, '{tenant_record_id}'::uuid, '{document_record_id}'::uuid, NULL, \
            '{entity_record_id}'::uuid, NULL, '{role}', {weight}, \
            {valid_from_window}, {valid_to_window}, 'second', \
            '2026-01-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz\
         )",
        Uuid::now_v7()
    ))
}

fn assert_membership_count(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    document_record_id: Uuid,
    expected: i64,
) {
    repo.session_mut()
        .execute(&format!(
            "DO $tepp_membership_duplicate$ BEGIN \
             IF (SELECT COUNT(*) FROM membership_assignment \
                 WHERE document_record_id = '{document_record_id}'::uuid) <> {expected} THEN \
               RAISE EXCEPTION 'unexpected membership row count'; \
             END IF; \
             END $tepp_membership_duplicate$"
        ))
        .expect("membership row count");
}
