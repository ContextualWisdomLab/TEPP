//! Live PostgreSQL regression for unbounded membership validity uncertainty.

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
fn unbounded_membership_windows_cannot_bypass_same_role_share_budget() {
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

    let upper_unbounded_document = Uuid::now_v7();
    repo.session_mut()
        .execute(&membership_insert_sql(
            tenant_record_id,
            upper_unbounded_document,
            entity_a,
            Uuid::now_v7(),
            "0.75",
            "'[2026-01-01,2026-01-01]'::tstzrange",
            "'[2026-01-10,)'::tstzrange",
        ))
        .expect("upper-unbounded end uncertainty is schema-admissible");
    assert!(
        repo.session_mut()
            .execute(&membership_insert_sql(
                tenant_record_id,
                upper_unbounded_document,
                entity_b,
                Uuid::now_v7(),
                "0.5",
                "'[2026-02-01,2026-02-01]'::tstzrange",
                "'[2026-02-02,2026-02-02]'::tstzrange",
            ))
            .is_err(),
        "an upper-unbounded end window can overlap every later spell and must remain in the budget"
    );
    assert_membership_count(&mut repo, upper_unbounded_document, 1);

    let lower_unbounded_document = Uuid::now_v7();
    repo.session_mut()
        .execute(&membership_insert_sql(
            tenant_record_id,
            lower_unbounded_document,
            entity_a,
            Uuid::now_v7(),
            "0.75",
            "'[2026-02-01,2026-02-01]'::tstzrange",
            "'[2026-02-10,2026-02-10]'::tstzrange",
        ))
        .expect("finite baseline membership");
    assert!(
        repo.session_mut()
            .execute(&membership_insert_sql(
                tenant_record_id,
                lower_unbounded_document,
                entity_b,
                Uuid::now_v7(),
                "0.5",
                "'(,2026-02-05]'::tstzrange",
                "'[2026-02-20,2026-02-20]'::tstzrange",
            ))
            .is_err(),
        "a lower-unbounded start window must not turn the overlap predicate into SQL UNKNOWN"
    );
    assert_membership_count(&mut repo, lower_unbounded_document, 1);

    let finite_disjoint_document = Uuid::now_v7();
    for (entity_record_id, from, to) in [
        (entity_a, "2026-03-01", "2026-03-10"),
        (entity_b, "2026-03-11", "2026-03-20"),
    ] {
        repo.session_mut()
            .execute(&membership_insert_sql(
                tenant_record_id,
                finite_disjoint_document,
                entity_record_id,
                Uuid::now_v7(),
                "1",
                &format!("'[{from},{from}]'::tstzrange"),
                &format!("'[{to},{to}]'::tstzrange"),
            ))
            .expect("finite disjoint spells keep independent budgets");
    }
    assert_membership_count(&mut repo, finite_disjoint_document, 2);
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
    expected: i64,
) {
    repo.session_mut()
        .execute(&format!(
            "DO $tepp_membership_unbounded$ BEGIN \
             IF (SELECT COUNT(*) FROM membership_assignment \
                 WHERE document_record_id = '{document_record_id}'::uuid \
                   AND membership_type_code = 'department') <> {expected} THEN \
               RAISE EXCEPTION 'unexpected membership row count'; \
             END IF; \
             END $tepp_membership_unbounded$"
        ))
        .expect("membership row count");
}

fn membership_insert_sql(
    tenant_record_id: Uuid,
    document_record_id: Uuid,
    entity_record_id: Uuid,
    membership_assignment_id: Uuid,
    weight: &str,
    valid_from_window: &str,
    valid_to_window: &str,
) -> String {
    format!(
        "INSERT INTO membership_assignment (\
            membership_assignment_id, tenant_record_id, document_record_id, text_segment_id, \
            target_entity_id, target_project_id, membership_type_code, membership_weight, \
            valid_from_window, valid_to_window, valid_time_precision_code, system_time, available_time\
         ) VALUES (\
            '{membership_assignment_id}'::uuid, '{tenant_record_id}'::uuid, \
            '{document_record_id}'::uuid, NULL, '{entity_record_id}'::uuid, NULL, \
            'department', {weight}, {valid_from_window}, {valid_to_window}, 'second', \
            '2026-01-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz\
         )"
    )
}
