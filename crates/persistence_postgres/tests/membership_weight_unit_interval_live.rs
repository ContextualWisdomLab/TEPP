//! Live PostgreSQL proof for the Membership `(0, 1]` persistence boundary.

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
fn live_postgres_rejects_membership_weight_above_unity() {
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
        .expect("0001..0009 migrations must apply");

    let tenant_record_id = Uuid::now_v7();
    let entity_record_id = Uuid::now_v7();
    repo.session_mut()
        .execute(&format!(
            "INSERT INTO tenant_record (tenant_record_id, tenant_status_code, system_time) \
             VALUES ('{tenant_record_id}'::uuid, 'active', '2026-01-01T00:00:00Z'::timestamptz)"
        ))
        .expect("seed tenant");
    repo.session_mut()
        .execute(&format!(
            "INSERT INTO entity_record (entity_record_id, tenant_record_id, entity_type_code, system_time, available_time) \
             VALUES ('{entity_record_id}'::uuid, '{tenant_record_id}'::uuid, 'department', \
                     '2026-01-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz)"
        ))
        .expect("seed entity");

    for weight in ["0.5", "1"] {
        repo.session_mut()
            .execute(&membership_insert_sql(
                tenant_record_id,
                entity_record_id,
                Uuid::now_v7(),
                weight,
            ))
            .expect("positive unit-interval membership weight must persist");
    }

    let over_unity = membership_insert_sql(
        tenant_record_id,
        entity_record_id,
        Uuid::now_v7(),
        "1.25",
    );
    assert!(
        repo.session_mut().execute(&over_unity).is_err(),
        "direct SQL above one must fail membership_assignment_weight_unit_interval"
    );

    repo.session_mut()
        .execute(
            "DO $tepp_membership_weight$ BEGIN \
             IF NOT EXISTS ( \
               SELECT 1 FROM pg_constraint \
               WHERE conname = 'membership_assignment_weight_unit_interval' \
                 AND conrelid = 'membership_assignment'::regclass \
             ) THEN \
               RAISE EXCEPTION 'missing membership weight unit-interval constraint'; \
             END IF; \
             IF (SELECT COUNT(*) FROM membership_assignment) <> 2 THEN \
               RAISE EXCEPTION 'invalid membership write changed persisted row count'; \
             END IF; \
             END $tepp_membership_weight$",
        )
        .expect("constraint identity and mutation atomicity");
}

fn membership_insert_sql(
    tenant_record_id: Uuid,
    entity_record_id: Uuid,
    membership_assignment_id: Uuid,
    weight: &str,
) -> String {
    let document_record_id = Uuid::now_v7();
    format!(
        "INSERT INTO membership_assignment (\
            membership_assignment_id, tenant_record_id, document_record_id, text_segment_id, \
            target_entity_id, target_project_id, membership_type_code, membership_weight, \
            valid_from_window, valid_to_window, valid_time_precision_code, system_time, available_time\
         ) VALUES (\
            '{membership_assignment_id}'::uuid, '{tenant_record_id}'::uuid, \
            '{document_record_id}'::uuid, NULL, '{entity_record_id}'::uuid, NULL, \
            'department', {weight}, '[2026-01-01,2026-01-01]'::tstzrange, NULL, 'second', \
            '2026-01-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz\
         )"
    )
}
