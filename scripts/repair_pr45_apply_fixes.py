"""Apply PR 45 tenant, migration, advisory-lock, and lifecycle fixes."""

from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    """Replace exactly one fragment or fail closed."""
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


live_path = Path("crates/persistence_postgres/src/live_repository.rs")
live = live_path.read_text(encoding="utf-8")
old_helper = """    /// Bind the session tenant GUC required by FORCE RLS and the 0007
    /// tombstone-restore trigger before mutating `document_record`.
    fn bind_document_tenant(&mut self, tenant_record_id: Uuid) -> Result<(), PersistenceError> {
"""
new_helper = """    /// Bind the session tenant GUC before tenant-scoped persistence operations.
    fn bind_session_tenant(&mut self, tenant_record_id: Uuid) -> Result<(), PersistenceError> {
"""
live = replace_once(live, old_helper, new_helper, "live_repository tenant helper")
live = live.replace("self.bind_document_tenant(", "self.bind_session_tenant(")

tenant_methods = [
    "append_audit",
    "insert_reproducibility_manifest",
    "insert_corpus_split_manifest",
    "insert_model_run",
    "insert_model_artifact",
    "insert_membership_assignment",
    "insert_event_relation",
    "insert_event_mention",
    "insert_event_instance",
    "insert_source_artifact",
    "insert_retention_policy",
    "insert_legal_hold",
    "insert_deletion_request",
    "insert_completed_deletion_request",
    "insert_evidence_tombstone",
]
for method in tenant_methods:
    start = live.find(f"    pub fn {method}(")
    if start < 0:
        raise SystemExit(f"live_repository.rs: method not found: {method}")
    brace = live.find(" {\n", start)
    if brace < 0:
        raise SystemExit(f"live_repository.rs: method brace not found: {method}")
    insert_at = brace + len(" {\n")
    expected = "        let sql ="
    if not live.startswith(expected, insert_at):
        raise SystemExit(
            f"live_repository.rs: unexpected first statement for {method}: "
            f"{live[insert_at:insert_at + 80]!r}"
        )
    signature = live[start:brace]
    parameter = "event" if "event: &AuditEvent" in signature else "record"
    live = (
        live[:insert_at]
        + f"        self.bind_session_tenant({parameter}.tenant_record_id)?;\n"
        + live[insert_at:]
    )
live_path.write_text(live, encoding="utf-8")

up_path = Path("migrations/0007_retention_deletion_legal_hold.up.sql")
up = up_path.read_text(encoding="utf-8")
up = replace_once(
    up,
    """    PERFORM pg_advisory_xact_lock(
        hashtextextended(NEW.tenant_record_id::text, 0),
        hashtextextended(NEW.target_document_id::text, 0)
    );
""",
    """    PERFORM pg_advisory_xact_lock(
        hashtextextended(
            NEW.tenant_record_id::text || ':' || NEW.target_document_id::text,
            0
        )
    );
""",
    "0007 deletion advisory lock",
)
up = replace_once(
    up,
    """    PERFORM pg_advisory_xact_lock(
        hashtextextended(NEW.tenant_record_id::text, 0),
        hashtextextended(lock_document::text, 0)
    );
""",
    """    PERFORM pg_advisory_xact_lock(
        hashtextextended(
            NEW.tenant_record_id::text || ':' || lock_document::text,
            0
        )
    );
""",
    "0007 hold-insert advisory lock",
)
up = replace_once(
    up,
    """    PERFORM pg_advisory_xact_lock(
        hashtextextended(current_hold.tenant_record_id::text, 0),
        hashtextextended(lock_document::text, 0)
    );
""",
    """    PERFORM pg_advisory_xact_lock(
        hashtextextended(
            current_hold.tenant_record_id::text || ':' || lock_document::text,
            0
        )
    );
""",
    "0007 hold-release advisory lock",
)
routine_privileges = """REVOKE ALL ON FUNCTION enforce_retention_policy_succession() FROM PUBLIC;
REVOKE ALL ON FUNCTION supersede_retention_policy(uuid, uuid, integer, text, timestamptz, timestamptz) FROM PUBLIC;
REVOKE ALL ON FUNCTION reject_held_evidence_deletion() FROM PUBLIC;
REVOKE ALL ON FUNCTION reject_tombstoned_evidence_restore() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION supersede_retention_policy(uuid, uuid, integer, text, timestamptz, timestamptz) TO tepp_app_runtime;
REVOKE ALL ON FUNCTION release_legal_hold(uuid, timestamptz) FROM PUBLIC;
REVOKE ALL ON FUNCTION enforce_legal_hold_release() FROM PUBLIC;
REVOKE ALL ON FUNCTION guard_legal_hold_insert() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION release_legal_hold(uuid, timestamptz) TO tepp_app_runtime;
"""
up = replace_once(up, routine_privileges + "\n", "", "0007 routine privileges")
up = up.rstrip() + "\n\n" + routine_privileges
up_path.write_text(up, encoding="utf-8")

down_path = Path("migrations/0007_retention_deletion_legal_hold.down.sql")
down = down_path.read_text(encoding="utf-8")
function_drops = """DROP FUNCTION IF EXISTS release_legal_hold(uuid, timestamptz);
DROP FUNCTION IF EXISTS supersede_retention_policy(uuid, uuid, integer, text, timestamptz, timestamptz);
DROP FUNCTION IF EXISTS enforce_retention_policy_succession();
DROP FUNCTION IF EXISTS enforce_legal_hold_release();
DROP FUNCTION IF EXISTS reject_tombstoned_evidence_restore();
DROP FUNCTION IF EXISTS reject_held_evidence_deletion();
DROP FUNCTION IF EXISTS guard_legal_hold_insert();
"""
down = replace_once(down, function_drops + "\n", "", "0007 function drops")
down = down.rstrip() + "\n\n" + function_drops
down_path.write_text(down, encoding="utf-8")
