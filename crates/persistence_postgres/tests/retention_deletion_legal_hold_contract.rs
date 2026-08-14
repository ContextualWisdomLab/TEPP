//! Retention, deletion, and legal-hold SQL must refuse ungoverned restore.

use persistence_postgres::{MigrationCatalog, validate_migration_catalog};

fn normalized(sql: &str) -> String {
    sql.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

#[test]
fn embedded_catalog_declares_retention_deletion_and_legal_hold() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    validate_migration_catalog(&catalog).expect("embedded migration contract");

    let up_sql = normalized(catalog.up_sql());
    assert!(
        up_sql.contains("create table retention_policy"),
        "policy-driven retention must be a first-class table"
    );
    assert!(
        up_sql.contains("create table legal_hold"),
        "legal hold must be recorded with authority, not inferred"
    );
    assert!(
        up_sql.contains("create table deletion_request"),
        "deletion must be an auditable request, not a silent DELETE"
    );
    assert!(
        up_sql.contains("create table evidence_tombstone"),
        "source deletion must leave a tombstone without raw PII"
    );
    assert!(
        up_sql.contains("create or replace function reject_held_evidence_deletion"),
        "an active legal hold must block completed deletion"
    );
    assert!(
        up_sql.contains("create or replace function reject_tombstoned_evidence_restore"),
        "tombstoned document identities must not be restored"
    );
    assert!(
        up_sql.contains("constraint retention_policy_period_positive"),
        "retention periods must be strictly positive"
    );
    assert!(
        up_sql.contains("constraint legal_hold_document_scope_consistent"),
        "document-scoped holds must name the held document"
    );
}

#[test]
fn retention_policy_succession_is_atomic_and_single_active() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    let up_sql = normalized(catalog.up_sql());

    assert!(
        up_sql.contains("system_to timestamptz"),
        "policy history must close the prior system-time version"
    );
    assert!(
        up_sql.contains("supersedes_retention_policy_id uuid"),
        "replacement policies must identify their predecessor"
    );
    assert!(
        up_sql.contains("create or replace function supersede_retention_policy"),
        "policy succession must be one atomic database operation"
    );
    assert!(
        up_sql.contains("where policy_status_code = 'active' and system_to is null"),
        "only the open active policy may occupy a tenant/class/purpose key"
    );
    assert!(
        up_sql.contains("create trigger retention_policy_enforce_succession"),
        "ordinary policy mutation must remain fail closed"
    );
    assert!(
        up_sql.contains("create or replace function release_legal_hold"),
        "legal holds must have a controlled release path"
    );
    assert!(
        up_sql.contains("create index evidence_tombstone_document_lookup"),
        "tombstone restore checks need an indexed lookup"
    );
    assert!(
        up_sql.contains("pg_advisory_xact_lock"),
        "hold and deletion races must be serialized"
    );
}

#[test]
fn lifecycle_trigger_functions_are_tenant_bound_security_definers() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    let up_sql = normalized(catalog.up_sql());

    for function_name in [
        "reject_held_evidence_deletion",
        "reject_tombstoned_evidence_restore",
        "supersede_retention_policy",
        "release_legal_hold",
        "guard_legal_hold_insert",
    ] {
        let start = up_sql
            .find(&format!("create or replace function {function_name}"))
            .expect("lifecycle function must exist");
        let suffix = &up_sql[start..];
        let end = suffix.find("$$;").expect("function body must close") + 3;
        let function_sql = &suffix[..end];
        assert!(
            function_sql.contains("security definer"),
            "{function_name} must not lose protected rows behind FORCE RLS"
        );
        assert!(
            function_sql.contains("set search_path = public, pg_temp"),
            "{function_name} must pin a trusted search path"
        );
        assert!(
            function_sql.contains("tepp.current_tenant_record_id"),
            "{function_name} must require tenant session context"
        );
    }
}

#[test]
fn rollback_drops_retention_lifecycle_tables_and_functions() {
    let catalog = MigrationCatalog::from_embedded().expect("embedded migration catalog");
    let down_sql = normalized(catalog.down_sql());
    assert!(
        down_sql.contains("drop trigger if exists document_record_reject_tombstone_restore"),
        "0007 rollback must detach the document restore tripwire"
    );
    assert!(
        down_sql.contains("drop function if exists supersede_retention_policy"),
        "0007 rollback must remove the succession API"
    );
    assert!(
        down_sql.contains("drop function if exists enforce_retention_policy_succession"),
        "0007 rollback must remove the succession mutation guard"
    );
    assert!(
        down_sql.contains("drop function if exists release_legal_hold"),
        "0007 rollback must remove the legal-hold release API"
    );
    assert!(
        down_sql.contains("to_regclass('public.legal_hold')"),
        "trigger drops must guard missing tables during partial rollback"
    );
    for table in [
        "evidence_tombstone",
        "deletion_request",
        "legal_hold",
        "retention_policy",
    ] {
        assert!(
            down_sql.contains(&format!("'{table}'")),
            "0007 rollback must drop {table}"
        );
    }
}
