//! Live PostgreSQL contract for target-only duplicate-edge updates.

#![cfg(feature = "live-sqlx")]

use persistence_postgres::{
    LiveDocumentRepository, LiveSqlxPoolOptions, MigrationCatalog, SqlSession, apply_sql_batch,
    open_live_sqlx_pool, require_live_sqlx_config,
};
use uuid::Uuid;

const LIVE_GATE_ENV: &str = "TEPP_LIVE_POSTGRES";

#[test]
fn target_only_update_cannot_create_a_duplicate_membership_edge() {
    if !std::env::var(LIVE_GATE_ENV).is_ok_and(|value| value == "1") {
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
    let document_record_id = Uuid::now_v7();
    let entity_a = Uuid::now_v7();
    let entity_b = Uuid::now_v7();
    repo.session_mut()
        .execute(&format!(
            "INSERT INTO tenant_record (tenant_record_id, tenant_status_code, system_time) \
             VALUES ('{tenant_record_id}'::uuid, 'active', '2026-01-01T00:00:00Z'::timestamptz)"
        ))
        .expect("seed tenant");
    for entity_record_id in [entity_a, entity_b] {
        repo.session_mut()
            .execute(&format!(
                "INSERT INTO entity_record (entity_record_id, tenant_record_id, entity_type_code, system_time, available_time) \
                 VALUES ('{entity_record_id}'::uuid, '{tenant_record_id}'::uuid, 'membership_group', \
                         '2026-01-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz)"
            ))
            .expect("seed target");
    }

    let assignment_a = Uuid::now_v7();
    let assignment_b = Uuid::now_v7();
    for (assignment_id, entity_record_id) in [(assignment_a, entity_a), (assignment_b, entity_b)] {
        repo.session_mut()
            .execute(&format!(
                "INSERT INTO membership_assignment (\
                    membership_assignment_id, tenant_record_id, document_record_id, text_segment_id, \
                    target_entity_id, target_project_id, membership_type_code, membership_weight, \
                    valid_from_window, valid_to_window, valid_time_precision_code, system_time, available_time\
                 ) VALUES (\
                    '{assignment_id}'::uuid, '{tenant_record_id}'::uuid, '{document_record_id}'::uuid, NULL, \
                    '{entity_record_id}'::uuid, NULL, 'department', 0.5, \
                    '[2026-08-01,2026-08-01]'::tstzrange, '[2026-08-31,2026-08-31]'::tstzrange, \
                    'second', '2026-01-01T00:00:00Z'::timestamptz, \
                    '2026-01-01T00:00:00Z'::timestamptz\
                 )"
            ))
            .expect("distinct targets remain valid multiple membership");
    }

    assert!(
        repo.session_mut()
            .execute(&format!(
                "UPDATE membership_assignment \
                 SET target_entity_id = '{entity_a}'::uuid \
                 WHERE membership_assignment_id = '{assignment_b}'::uuid"
            ))
            .is_err(),
        "changing only the target must still run duplicate-edge admission"
    );

    repo.session_mut()
        .execute(&format!(
            "DO $tepp_target_update$ BEGIN \
             IF (SELECT target_entity_id FROM membership_assignment \
                 WHERE membership_assignment_id = '{assignment_b}'::uuid) <> '{entity_b}'::uuid THEN \
               RAISE EXCEPTION 'rejected target-only update changed persisted identity'; \
             END IF; \
             END $tepp_target_update$"
        ))
        .expect("rejected update remains atomic");
}
