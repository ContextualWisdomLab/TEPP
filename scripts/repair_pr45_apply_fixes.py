"""Apply PR 45 tenant, migration-order, and lifecycle fixes."""

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

old_deletion_table_tail = """    legal_hold_id uuid REFERENCES legal_hold (legal_hold_id),
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL
);

CREATE TABLE evidence_tombstone (
"""
new_deletion_table_tail = """    legal_hold_id uuid REFERENCES legal_hold (legal_hold_id),
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT deletion_request_kind_known CHECK (
        deletion_kind_code IN (
            'logical_revocation',
            'cache_export_removal',
            'identity_tombstone'
        )
    ),
    CONSTRAINT deletion_request_status_known CHECK (
        request_status_code IN (
            'requested',
            'completed',
            'blocked_by_hold',
            'reproduction_limited'
        )
    )
);

CREATE TABLE evidence_tombstone (
"""
up = replace_once(
    up,
    old_deletion_table_tail,
    new_deletion_table_tail,
    "0007 deletion request allowlists",
)

old_tombstone_table_tail = """    reproduction_status_code text NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL
);

CREATE UNIQUE INDEX retention_policy_active_purpose_unique
"""
new_tombstone_table_tail = """    reproduction_status_code text NOT NULL,
    system_time timestamptz NOT NULL,
    available_time timestamptz NOT NULL,
    CONSTRAINT evidence_tombstone_kind_known CHECK (
        deletion_kind_code IN (
            'logical_revocation',
            'cache_export_removal',
            'identity_tombstone'
        )
    ),
    CONSTRAINT evidence_tombstone_reproduction_status_known CHECK (
        reproduction_status_code IN ('unavailable', 'limited', 'unaffected')
    )
);

CREATE UNIQUE INDEX retention_policy_active_purpose_unique
"""
up = replace_once(
    up,
    old_tombstone_table_tail,
    new_tombstone_table_tail,
    "0007 evidence tombstone allowlists",
)

document_restore_trigger = """CREATE TRIGGER document_record_reject_tombstone_restore
    BEFORE INSERT OR UPDATE ON document_record
    FOR EACH ROW
    EXECUTE FUNCTION reject_tombstoned_evidence_restore();

"""
tombstone_guard = """CREATE OR REPLACE FUNCTION guard_evidence_tombstone_insert()
RETURNS trigger
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
DECLARE
    tenant_context text;
BEGIN
    tenant_context := nullif(current_setting('tepp.current_tenant_record_id', true), '');
    IF tenant_context IS NULL OR NEW.tenant_record_id::text <> tenant_context THEN
        RAISE EXCEPTION 'tenant session context is required for lifecycle mutation'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF NOT EXISTS (
        SELECT 1
        FROM public.deletion_request
        WHERE deletion_request_id = NEW.deletion_request_id
          AND tenant_record_id = NEW.tenant_record_id
          AND target_document_id = NEW.tombstoned_document_id
          AND target_data_class_code = NEW.target_data_class_code
          AND deletion_kind_code = NEW.deletion_kind_code
          AND request_status_code = 'completed'
    ) THEN
        RAISE EXCEPTION 'evidence tombstone must match one completed deletion request'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER evidence_tombstone_guard_insert
    BEFORE INSERT ON evidence_tombstone
    FOR EACH ROW
    EXECUTE FUNCTION guard_evidence_tombstone_insert();

"""
up = replace_once(
    up,
    document_restore_trigger,
    document_restore_trigger + tombstone_guard,
    "0007 tombstone request guard",
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
final_routine_privileges = routine_privileges + (
    "REVOKE ALL ON FUNCTION guard_evidence_tombstone_insert() FROM PUBLIC;\n"
)
up = up.rstrip() + "\n\n" + final_routine_privileges
up_path.write_text(up, encoding="utf-8")

down_path = Path("migrations/0007_retention_deletion_legal_hold.down.sql")
down = down_path.read_text(encoding="utf-8")
old_tombstone_trigger_cleanup = """    IF to_regclass('public.evidence_tombstone') IS NOT NULL THEN
        DROP TRIGGER IF EXISTS evidence_tombstone_reject_mutation ON public.evidence_tombstone;
    END IF;
"""
new_tombstone_trigger_cleanup = """    IF to_regclass('public.evidence_tombstone') IS NOT NULL THEN
        DROP TRIGGER IF EXISTS evidence_tombstone_guard_insert ON public.evidence_tombstone;
        DROP TRIGGER IF EXISTS evidence_tombstone_reject_mutation ON public.evidence_tombstone;
    END IF;
"""
down = replace_once(
    down,
    old_tombstone_trigger_cleanup,
    new_tombstone_trigger_cleanup,
    "0007 tombstone guard rollback",
)
existing_function_drops = """DROP FUNCTION IF EXISTS release_legal_hold(uuid, timestamptz);
DROP FUNCTION IF EXISTS supersede_retention_policy(uuid, uuid, integer, text, timestamptz, timestamptz);
DROP FUNCTION IF EXISTS enforce_retention_policy_succession();
DROP FUNCTION IF EXISTS enforce_legal_hold_release();
DROP FUNCTION IF EXISTS reject_tombstoned_evidence_restore();
DROP FUNCTION IF EXISTS reject_held_evidence_deletion();
DROP FUNCTION IF EXISTS guard_legal_hold_insert();
"""
down = replace_once(
    down,
    existing_function_drops,
    existing_function_drops
    + "DROP FUNCTION IF EXISTS guard_evidence_tombstone_insert();\n",
    "0007 tombstone guard function rollback",
)
down_path.write_text(down, encoding="utf-8")
