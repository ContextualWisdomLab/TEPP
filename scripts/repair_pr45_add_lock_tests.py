"""Add regression coverage for PR 45 lifecycle lock semantics."""

from pathlib import Path


path = Path("crates/persistence_postgres/src/migration.rs")
text = path.read_text(encoding="utf-8")
marker = """    #[test]
    fn helper_predicates_are_exhaustive() {
"""
regression = """    #[test]
    fn retention_lifecycle_uses_valid_single_key_locks_and_symmetric_guard() {
        assert!(
            !RETENTION_UP.contains(
                "pg_advisory_xact_lock(\\n        hashtextextended(NEW.tenant_record_id::text, 0),"
            ),
            "PostgreSQL does not provide a two-bigint advisory-lock overload"
        );
        assert!(
            RETENTION_UP.matches("'tepp:lifecycle:tenant:'").count() >= 3,
            "deletion, hold insertion, and hold release must share a tenant lock"
        );
        assert!(
            RETENTION_UP
                .matches("'tepp:lifecycle:document:'")
                .count()
                >= 3,
            "document lifecycle paths must share the document lock"
        );
        assert!(RETENTION_UP.contains("FROM public.deletion_request AS completed_deletion"));
        assert!(RETENTION_UP.contains("completed_deletion.request_status_code = 'completed'"));
        assert!(RETENTION_UP.contains("completed_deletion.system_time >= NEW.system_time"));
        assert!(RETENTION_UP.contains("deletion_request_completed_scope_lookup"));
    }

    #[test]
    fn helper_predicates_are_exhaustive() {
"""
if text.count(marker) != 1:
    raise SystemExit(
        "migration.rs: expected one helper_predicates_are_exhaustive marker, "
        f"found {text.count(marker)}"
    )
path.write_text(text.replace(marker, regression, 1), encoding="utf-8")
