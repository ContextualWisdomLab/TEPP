"""Add PR 45 regression tests before applying implementation fixes."""

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    """Replace exactly one source fragment or fail closed."""
    file_path = Path(path)
    text = file_path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement target, found {count}")
    file_path.write_text(text.replace(old, new, 1), encoding="utf-8")


live_repository_test_marker = """    #[test]
    fn insert_and_revise_bind_tenant_session_before_document_sql() {
"""
live_repository_test = """    #[test]
    fn tenant_scoped_insert_binds_session_context() {
        let mut repo = LiveDocumentRepository::new(RecordingSqlSession::new());
        let manifest = ReproducibilityManifestRecord {
            reproducibility_manifest_id: uuid::Uuid::from_u128(11),
            tenant_record_id: uuid::Uuid::from_u128(12),
            knowledge_cutoff: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z")
                .expect("knowledge cutoff"),
            evidence_digest: "ab".repeat(32),
            code_commit_sha: "c".repeat(40),
            dependency_lock_digest: "de".repeat(32),
            system_time: SystemTime::parse_rfc3339("2026-01-01T00:00:00Z")
                .expect("system"),
            available_time: AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z")
                .expect("available"),
        };

        repo.insert_reproducibility_manifest(&manifest)
            .expect("manifest insert");
        let executed = repo.session().executed();
        assert_eq!(
            executed.len(),
            2,
            "tenant-scoped insert must bind the session before the INSERT"
        );
        assert!(executed[0].contains("tepp.current_tenant_record_id"));
        assert!(executed[0].contains(&manifest.tenant_record_id.to_string()));
        assert!(executed[1].contains("INSERT INTO reproducibility_manifest"));
    }

    #[test]
    fn insert_and_revise_bind_tenant_session_before_document_sql() {
"""
replace_once(
    "crates/persistence_postgres/src/live_repository.rs",
    live_repository_test_marker,
    live_repository_test,
)

artifact_old = """        let referenced = insert_source_artifact_sql(&with_ref).expect("ref");
        assert!(referenced.contains("s3://tepp/object"));
"""
artifact_new = """        let referenced = insert_source_artifact_sql(&with_ref).expect("ref");
        assert!(referenced.contains("s3://tepp/object"));
        let referenced_assertion =
            assert_source_artifact_matches_sql(&with_ref).expect("referenced assertion");
        assert!(referenced_assertion.contains("s3://tepp/object"));
"""
replace_once(
    "crates/persistence_postgres/src/artifact_sql.rs",
    artifact_old,
    artifact_new,
)

retention_marker = """    #[test]
    fn hold_scope_mismatch_and_unknown_codes_fail_closed() {
"""
retention_test = """    #[test]
    fn lifecycle_windows_fail_closed() {
        let future = SystemTime::parse_rfc3339("2026-02-01T00:00:00Z").expect("future");
        let past = SystemTime::parse_rfc3339("2025-12-31T00:00:00Z").expect("past");

        let mut active_policy_closed = policy();
        active_policy_closed.system_to = Some(future);
        assert_eq!(
            insert_retention_policy_sql(&active_policy_closed),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut superseded_policy_open = policy();
        superseded_policy_open.policy_status_code = "superseded".into();
        assert_eq!(
            insert_retention_policy_sql(&superseded_policy_open),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut superseded_policy_inverted = policy();
        superseded_policy_inverted.policy_status_code = "superseded".into();
        superseded_policy_inverted.system_to = Some(past);
        assert_eq!(
            insert_retention_policy_sql(&superseded_policy_inverted),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut active_hold_closed = hold();
        active_hold_closed.system_to = Some(future);
        assert_eq!(
            insert_legal_hold_sql(&active_hold_closed),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut released_hold_open = hold();
        released_hold_open.hold_status_code = "released".into();
        assert_eq!(
            insert_legal_hold_sql(&released_hold_open),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );

        let mut released_hold_inverted = hold();
        released_hold_inverted.hold_status_code = "released".into();
        released_hold_inverted.system_to = Some(past);
        assert_eq!(
            insert_legal_hold_sql(&released_hold_inverted),
            Err(PersistenceError::InvalidRetentionLifecycle)
        );
    }

    #[test]
    fn hold_scope_mismatch_and_unknown_codes_fail_closed() {
"""
replace_once(
    "crates/persistence_postgres/src/retention_sql.rs",
    retention_marker,
    retention_test,
)

migration_use_old = "    use super::{MigrationCatalog, validate_migration_catalog};"
migration_use_new = (
    "    use super::{\n"
    "        MigrationCatalog, RETENTION_DOWN, RETENTION_UP, validate_migration_catalog,\n"
    "    };"
)
replace_once(
    "crates/persistence_postgres/src/migration.rs",
    migration_use_old,
    migration_use_new,
)
migration_marker = """    #[test]
    fn helper_predicates_are_exhaustive() {
"""
migration_test = """    #[test]
    fn retention_migration_orders_trigger_and_privilege_dependencies() {
        let release_definition = RETENTION_UP
            .rfind("CREATE OR REPLACE FUNCTION release_legal_hold")
            .expect("release function definition");
        let release_privilege = RETENTION_UP
            .rfind("GRANT EXECUTE ON FUNCTION release_legal_hold")
            .expect("release function privilege");
        assert!(
            release_definition < release_privilege,
            "routine privileges must be applied only after routine creation"
        );

        let trigger_drop = RETENTION_DOWN
            .find("DROP TRIGGER IF EXISTS legal_hold_enforce_release")
            .expect("trigger drop");
        let function_drop = RETENTION_DOWN
            .find("DROP FUNCTION IF EXISTS enforce_legal_hold_release")
            .expect("function drop");
        assert!(
            trigger_drop < function_drop,
            "rollback must remove trigger dependencies before their functions"
        );
    }

    #[test]
    fn helper_predicates_are_exhaustive() {
"""
replace_once(
    "crates/persistence_postgres/src/migration.rs",
    migration_marker,
    migration_test,
)

live_test_path = Path("crates/persistence_postgres/tests/live_postgres.rs")
live_test = live_test_path.read_text(encoding="utf-8")
import_old = "reset_app_runtime_role_sql, set_session_tenant_sql,"
import_new = (
    "reset_app_runtime_role_sql, select_active_analysis_document_sql, "
    "set_session_tenant_sql,"
)
if live_test.count(import_old) != 1:
    raise SystemExit("live_postgres.rs: import replacement target mismatch")
live_test = live_test.replace(import_old, import_new, 1)

start = live_test.index("    let eligibility = format!(", live_test.index("fn prove_retention"))
end_marker = """    repo.submit_active_analysis_document(unheld_document_id)
        .expect("active-analysis select must exclude tombstones");
"""
end = live_test.index(end_marker, start) + len(end_marker)
replacement = """    let active_selection = select_active_analysis_document_sql(unheld_document_id);
    let eligibility = format!(
        "DO $tepp$ BEGIN \\
           IF EXISTS ({active_selection}) THEN \\
             RAISE EXCEPTION 'tombstoned document remained analysis-eligible'; \\
           END IF; \\
           IF EXISTS ( \\
             SELECT 1 FROM evidence_tombstone \\
             WHERE evidence_tombstone_id = '{tombstone}'::uuid \\
               AND reproduction_status_code = 'available' \\
           ) THEN \\
             RAISE EXCEPTION 'deleted raw source claimed available reproduction'; \\
           END IF; \\
         END $tepp$",
        tombstone = tombstone.evidence_tombstone_id,
    );
    repo.session_mut()
        .execute(&eligibility)
        .expect("active-analysis SQL must return zero tombstoned rows");
"""
live_test_path.write_text(live_test[:start] + replacement + live_test[end:], encoding="utf-8")
