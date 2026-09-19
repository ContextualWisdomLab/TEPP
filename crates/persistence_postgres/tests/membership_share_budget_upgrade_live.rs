//! Live PostgreSQL upgrade contracts for pre-existing Membership share state.

#![cfg(feature = "live-sqlx")]

use persistence_postgres::{
    LiveDocumentRepository, LiveSqlxPoolOptions, MigrationCatalog, SqlSession, apply_sql_batch,
    open_live_sqlx_pool, require_live_sqlx_config,
};
use uuid::Uuid;

const LIVE_GATE_ENV: &str = "TEPP_LIVE_POSTGRES";
const MEMBERSHIP_SHARE_BUDGET_UP: &str =
    include_str!("../../../migrations/0010_membership_same_role_share_budget.up.sql");

fn live_postgres_requested() -> bool {
    std::env::var(LIVE_GATE_ENV).is_ok_and(|value| value == "1")
}

#[test]
fn successor_refuses_invalid_legacy_membership_state_and_accepts_pointwise_valid_state() {
    if !live_postgres_requested() {
        return;
    }

    let config = require_live_sqlx_config()
        .expect("DATABASE_URL must be valid when TEPP_LIVE_POSTGRES=1");
    let options = LiveSqlxPoolOptions::new(1, 5_000).expect("pool options");
    let pool = open_live_sqlx_pool(&config, options).expect("open live PostgreSQL pool");
    let mut repo = LiveDocumentRepository::new(pool);
    let full_catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    let predecessor_up = full_catalog
        .up_sql()
        .strip_suffix(MEMBERSHIP_SHARE_BUDGET_UP)
        .expect("0010 must be the terminal forward migration in the embedded catalog");

    reset_to_predecessor(&mut repo, &full_catalog, predecessor_up);
    let (tenant_record_id, entities) = seed_scope(&mut repo, 3);
    let underflow_document = Uuid::now_v7();
    insert_legacy_membership(
        &mut repo,
        tenant_record_id,
        entities[0],
        underflow_document,
        "department",
        "1e-10000",
        "'[2026-01-01,2026-01-01]'::tstzrange",
        "NULL",
    );
    assert!(
        apply_sql_batch(repo.session_mut(), MEMBERSHIP_SHARE_BUDGET_UP).is_err(),
        "0010 must reject a positive NUMERIC legacy share that becomes binary64 zero"
    );
    assert_successor_enforcement_installed(&mut repo);
    assert!(
        repo.session_mut()
            .execute(&membership_insert_sql(
                tenant_record_id,
                entities[1],
                Uuid::now_v7(),
                "department",
                "1e-10000",
                "'[2026-01-02,2026-01-02]'::tstzrange",
                "NULL",
            ))
            .is_err(),
        "failed historical validation must still leave future writes protected"
    );
    repo.session_mut()
        .execute(&format!(
            "DELETE FROM membership_assignment WHERE document_record_id = '{underflow_document}'::uuid"
        ))
        .expect("data owner remediation of the invalid predecessor row");
    apply_sql_batch(repo.session_mut(), MEMBERSHIP_SHARE_BUDGET_UP)
        .expect("retry after explicit remediation must be idempotent and succeed");
    assert_successor_enforcement_installed(&mut repo);

    reset_to_predecessor(&mut repo, &full_catalog, predecessor_up);
    let (tenant_record_id, entities) = seed_scope(&mut repo, 3);
    let ordinary_overrun = Uuid::now_v7();
    for (entity_record_id, weight) in [(entities[0], "0.75"), (entities[1], "0.5")] {
        insert_legacy_membership(
            &mut repo,
            tenant_record_id,
            entity_record_id,
            ordinary_overrun,
            "department",
            weight,
            "'[2026-02-01,2026-02-01]'::tstzrange",
            "'[2026-02-28,2026-02-28]'::tstzrange",
        );
    }
    assert!(
        apply_sql_batch(repo.session_mut(), MEMBERSHIP_SHARE_BUDGET_UP).is_err(),
        "0010 must reject a pre-existing pointwise same-role aggregate above unity"
    );
    assert_successor_enforcement_installed(&mut repo);

    reset_to_predecessor(&mut repo, &full_catalog, predecessor_up);
    let (tenant_record_id, entities) = seed_scope(&mut repo, 3);
    let binary64_overrun = Uuid::now_v7();
    for (index, weight) in [
        "0.9734628667233794",
        "0.0038851022715484258",
        "0.02265203100507213",
    ]
    .into_iter()
    .enumerate()
    {
        insert_legacy_membership(
            &mut repo,
            tenant_record_id,
            entities[index],
            binary64_overrun,
            "department",
            weight,
            "'[2026-03-01,2026-03-01]'::tstzrange",
            "'[2026-03-31,2026-03-31]'::tstzrange",
        );
    }
    assert!(
        apply_sql_batch(repo.session_mut(), MEMBERSHIP_SHARE_BUDGET_UP).is_err(),
        "0010 must reject the #612 represented-binary64 legacy overrun"
    );
    assert_successor_enforcement_installed(&mut repo);

    reset_to_predecessor(&mut repo, &full_catalog, predecessor_up);
    let (tenant_record_id, entities) = seed_scope(&mut repo, 3);
    let duplicate_edge = Uuid::now_v7();
    for weight in ["0.5", "0.5"] {
        insert_legacy_membership(
            &mut repo,
            tenant_record_id,
            entities[0],
            duplicate_edge,
            "department",
            weight,
            "'[2026-03-01,2026-03-01]'::tstzrange",
            "'[2026-03-31,2026-03-31]'::tstzrange",
        );
    }
    assert!(
        apply_sql_batch(repo.session_mut(), MEMBERSHIP_SHARE_BUDGET_UP).is_err(),
        "0010 must reject a pre-existing duplicate temporal edge even when total share is unity"
    );
    assert_successor_enforcement_installed(&mut repo);

    reset_to_predecessor(&mut repo, &full_catalog, predecessor_up);
    let (tenant_record_id, entities) = seed_scope(&mut repo, 3);

    let exact_unity = Uuid::now_v7();
    for (entity_record_id, weight) in [(entities[0], "0.75"), (entities[1], "0.25")] {
        insert_legacy_membership(
            &mut repo,
            tenant_record_id,
            entity_record_id,
            exact_unity,
            "department",
            weight,
            "'[2026-04-01,2026-04-01]'::tstzrange",
            "'[2026-04-30,2026-04-30]'::tstzrange",
        );
    }

    let pointwise_valid = Uuid::now_v7();
    for (entity_record_id, weight, from_window, to_window) in [
        (
            entities[0],
            "0.6",
            "'[2026-05-01,2026-05-01]'::tstzrange",
            "'[2026-05-10,2026-05-10]'::tstzrange",
        ),
        (
            entities[1],
            "0.6",
            "'[2026-05-20,2026-05-20]'::tstzrange",
            "'[2026-05-30,2026-05-30]'::tstzrange",
        ),
        (
            entities[2],
            "0.4",
            "'[2026-05-05,2026-05-05]'::tstzrange",
            "'[2026-05-25,2026-05-25]'::tstzrange",
        ),
    ] {
        insert_legacy_membership(
            &mut repo,
            tenant_record_id,
            entity_record_id,
            pointwise_valid,
            "department",
            weight,
            from_window,
            to_window,
        );
    }

    let leave_reentry = Uuid::now_v7();
    insert_legacy_membership(
        &mut repo,
        tenant_record_id,
        entities[0],
        leave_reentry,
        "department",
        "1",
        "'[2026-06-01,2026-06-01]'::tstzrange",
        "'[2026-06-09,2026-06-10)'::tstzrange",
    );
    insert_legacy_membership(
        &mut repo,
        tenant_record_id,
        entities[0],
        leave_reentry,
        "department",
        "1",
        "'[2026-06-10,2026-06-10]'::tstzrange",
        "'[2026-06-20,2026-06-20]'::tstzrange",
    );

    let role_separated = Uuid::now_v7();
    insert_legacy_membership(
        &mut repo,
        tenant_record_id,
        entities[0],
        role_separated,
        "department",
        "1",
        "'(,2026-07-10]'::tstzrange",
        "'[2026-07-20,)'::tstzrange",
    );
    insert_legacy_membership(
        &mut repo,
        tenant_record_id,
        entities[1],
        role_separated,
        "project",
        "1",
        "'[2026-07-01,2026-07-01]'::tstzrange",
        "'[2026-07-30,2026-07-30]'::tstzrange",
    );

    apply_sql_batch(repo.session_mut(), MEMBERSHIP_SHARE_BUDGET_UP)
        .expect("pointwise-valid predecessor state must upgrade without normalization");
    assert_successor_enforcement_installed(&mut repo);
}

fn reset_to_predecessor(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    full_catalog: &MigrationCatalog,
    predecessor_up: &str,
) {
    let _ = apply_sql_batch(repo.session_mut(), full_catalog.down_sql());
    let _ = repo
        .session_mut()
        .execute("DROP ROLE IF EXISTS tepp_app_runtime");
    apply_sql_batch(repo.session_mut(), predecessor_up)
        .expect("canonical 0001..0009 predecessor must apply");
}

fn seed_scope(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    entity_count: usize,
) -> (Uuid, Vec<Uuid>) {
    let tenant_record_id = Uuid::now_v7();
    repo.session_mut()
        .execute(&format!(
            "INSERT INTO tenant_record (tenant_record_id, tenant_status_code, system_time) \
             VALUES ('{tenant_record_id}'::uuid, 'active', '2026-01-01T00:00:00Z'::timestamptz)"
        ))
        .expect("seed tenant");

    let entities: Vec<_> = (0..entity_count).map(|_| Uuid::now_v7()).collect();
    for entity_record_id in &entities {
        repo.session_mut()
            .execute(&format!(
                "INSERT INTO entity_record (entity_record_id, tenant_record_id, entity_type_code, system_time, available_time) \
                 VALUES ('{entity_record_id}'::uuid, '{tenant_record_id}'::uuid, 'membership_group', \
                         '2026-01-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz)"
            ))
            .expect("seed membership target");
    }
    (tenant_record_id, entities)
}

fn insert_legacy_membership(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
    tenant_record_id: Uuid,
    entity_record_id: Uuid,
    document_record_id: Uuid,
    role: &str,
    weight: &str,
    valid_from_window: &str,
    valid_to_window: &str,
) {
    repo.session_mut()
        .execute(&membership_insert_sql(
            tenant_record_id,
            entity_record_id,
            document_record_id,
            role,
            weight,
            valid_from_window,
            valid_to_window,
        ))
        .expect("predecessor schema must admit the legacy fixture");
}

fn membership_insert_sql(
    tenant_record_id: Uuid,
    entity_record_id: Uuid,
    document_record_id: Uuid,
    role: &str,
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
            '{}'::uuid, '{tenant_record_id}'::uuid, '{document_record_id}'::uuid, NULL, \
            '{entity_record_id}'::uuid, NULL, '{role}', {weight}, \
            {valid_from_window}, {valid_to_window}, 'second', \
            '2026-01-01T00:00:00Z'::timestamptz, '2026-01-01T00:00:00Z'::timestamptz\
         )",
        Uuid::now_v7()
    )
}

fn assert_successor_enforcement_installed(
    repo: &mut LiveDocumentRepository<persistence_postgres::LiveSqlxPool>,
) {
    repo.session_mut()
        .execute(
            "DO $tepp_membership_upgrade$ BEGIN \
             IF to_regclass('membership_share_budget_guard') IS NULL THEN \
               RAISE EXCEPTION 'membership share-budget guard was not installed'; \
             END IF; \
             IF NOT EXISTS ( \
               SELECT 1 FROM pg_trigger \
               WHERE tgname = 'membership_assignment_same_role_share_budget' \
                 AND tgrelid = 'membership_assignment'::regclass \
                 AND NOT tgisinternal \
             ) THEN \
               RAISE EXCEPTION 'membership share-budget trigger was not installed'; \
             END IF; \
             END $tepp_membership_upgrade$",
        )
        .expect("successor admission authority must be installed before historical validation");
}
