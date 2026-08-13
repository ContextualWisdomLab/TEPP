//! Append-only migration DDL must block every destructive table operation.

use persistence_postgres::{MigrationCatalog, validate_migration_catalog};

const APPEND_ONLY_TABLES: [&str; 6] = [
    "source_artifact",
    "audit_event",
    "reproducibility_manifest",
    "corpus_split_manifest",
    "model_run",
    "model_artifact",
];

fn normalized(sql: &str) -> String {
    sql.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

#[test]
fn embedded_append_only_ddl_rejects_update_delete_and_truncate() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    validate_migration_catalog(&catalog).expect("embedded migration contract");

    let up_sql = normalized(catalog.up_sql());
    assert!(up_sql.contains("raise exception 'append-only table % rejects %'"));
    assert!(up_sql.contains("errcode = 'integrity_constraint_violation'"));

    for table in APPEND_ONLY_TABLES {
        assert!(
            up_sql.contains(&format!(
                "revoke update, delete on table {table} from tepp_app_runtime"
            )),
            "runtime role must not retain UPDATE/DELETE on {table}"
        );
        assert!(
            up_sql.contains(&format!(
                "revoke truncate on table {table} from tepp_app_runtime"
            )),
            "runtime role must not retain TRUNCATE on {table}"
        );
        assert!(
            up_sql.contains(&format!(
                "create trigger {table}_reject_mutation before update or delete or truncate on {table} for each statement execute function reject_append_only_mutation()"
            )),
            "statement trigger must reject UPDATE, DELETE, and TRUNCATE on {table}"
        );
    }
}

#[test]
fn rollback_removes_rejection_triggers_without_granting_truncate() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    let down_sql = normalized(catalog.down_sql());

    assert!(down_sql.contains("drop function if exists reject_append_only_mutation()"));
    for table in APPEND_ONLY_TABLES {
        assert!(
            down_sql.contains(
                "drop trigger if exists %i on %i', trigger_table || '_reject_mutation', trigger_table"
            ) || down_sql.contains(&format!("drop trigger if exists {table}_reject_mutation on {table}")),
            "rollback must remove the append-only trigger for {table}"
        );
        assert!(
            down_sql.contains(&format!(
                "grant update, delete on table {table} to tepp_app_runtime"
            )),
            "rollback must restore only the privileges granted before migration 0004 on {table}"
        );
        assert!(
            !down_sql.contains(&format!(
                "grant truncate on table {table} to tepp_app_runtime"
            )),
            "rollback must not introduce a new TRUNCATE privilege on {table}"
        );
    }
}
