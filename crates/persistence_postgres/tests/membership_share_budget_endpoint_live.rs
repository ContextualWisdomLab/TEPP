//! Live PostgreSQL contracts for membership uncertainty-window endpoint semantics.

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
fn possible_overlap_respects_tstzrange_endpoint_inclusivity() {
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
    seed_tenant_and_entities(&mut repo, tenant_record_id, &[entity_a, entity_b]);

    let excluded_existing_end = Uuid::now_v7();
    insert_membership(
        &mut repo,
        tenant_record_id,
        excluded_existing_end,
        entity_a,
        "1",
        "'[2026-04-01,2026-04-01]'::tstzrange",
        "'[2026-04-10,2026-04-11)'::tstzrange",
    )
    .expect("first full membership");
    insert_membership(
        &mut repo,
        tenant_record_id,
        excluded_existing_end,
        entity_b,
        "1",
        "'[2026-04-11,2026-04-11]'::tstzrange",
        "'[2026-04-20,2026-04-20]'::tstzrange",
    )
    .expect("excluded existing upper endpoint makes the meeting spells disjoint");
    assert_membership_count(&mut repo, excluded_existing_end, 2);

    let included_meeting = Uuid::now_v7();
    insert_membership(
        &mut repo,
        tenant_record_id,
        included_meeting,
        entity_a,
        "1",
        "'[2026-05-01,2026-05-01]'::tstzrange",
        "'[2026-05-10,2026-05-11]'::tstzrange",
    )
    .expect("first included-end membership");
    assert!(
        insert_membership(
            &mut repo,
            tenant_record_id,
            included_meeting,
            entity_b,
            "1",
            "'[2026-05-11,2026-05-11]'::tstzrange",
            "'[2026-05-20,2026-05-20]'::tstzrange",
        )
        .is_err(),
        "included end and included start at the same instant can overlap and must share one budget"
    );
    assert_membership_count(&mut repo, included_meeting, 1);

    let excluded_candidate_start = Uuid::now_v7();
    insert_membership(
        &mut repo,
        tenant_record_id,
        excluded_candidate_start,
        entity_a,
        "1",
        "'[2026-06-01,2026-06-01]'::tstzrange",
        "'[2026-06-11,2026-06-11]'::tstzrange",
    )
    .expect("first exact-end membership");
    insert_membership(
        &mut repo,
        tenant_record_id,
        excluded_candidate_start,
        entity_b,
        "1",
        "'(2026-06-11,2026-06-12]'::tstzrange",
        "'[2026-06-20,2026-06-20]'::tstzrange",
    )
    .expect("excluded candidate lower endpoint makes the meeting spells disjoint");
    assert_membership_count(&mut repo, excluded_candidate_start, 2);
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

fn insert_membership(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    document_record_id: Uuid,
    entity_record_id: Uuid,
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
            '{entity_record_id}'::uuid, NULL, 'department', {weight}, \
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
            "DO $tepp_membership_endpoint$ BEGIN \
             IF (SELECT COUNT(*) FROM membership_assignment \
                 WHERE document_record_id = '{document_record_id}'::uuid \
                   AND membership_type_code = 'department') <> {expected} THEN \
               RAISE EXCEPTION 'unexpected membership row count'; \
             END IF; \
             END $tepp_membership_endpoint$"
        ))
        .expect("membership row count");
}
