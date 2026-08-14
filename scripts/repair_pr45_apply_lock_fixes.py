"""Repair PR 45 lifecycle advisory locks and symmetric hold/deletion checks."""

from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    """Replace exactly one SQL fragment or fail closed."""
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one replacement target, found {count}")
    return text.replace(old, new, 1)


path = Path("migrations/0007_retention_deletion_legal_hold.up.sql")
sql = path.read_text(encoding="utf-8")

index_marker = """CREATE INDEX evidence_tombstone_deletion_request_id_idx
    ON evidence_tombstone (deletion_request_id);
"""
index_replacement = """CREATE INDEX evidence_tombstone_deletion_request_id_idx
    ON evidence_tombstone (deletion_request_id);

CREATE INDEX deletion_request_completed_scope_lookup
    ON deletion_request (tenant_record_id, system_time, target_document_id)
    WHERE request_status_code = 'completed';
"""
sql = replace_once(sql, index_marker, index_replacement, "completed-deletion index")

delete_lock_old = """    -- Serialize hold/deletion races for the same tenant+document scope.
    PERFORM pg_advisory_xact_lock(
        hashtextextended(NEW.tenant_record_id::text, 0),
        hashtextextended(NEW.target_document_id::text, 0)
    );
"""
delete_lock_new = """    IF NEW.request_status_code = 'completed' THEN
        -- Acquire the tenant lock first so tenant-wide and document holds share
        -- one deadlock-free order. Each call uses PostgreSQL's single-bigint
        -- advisory-lock overload.
        PERFORM pg_advisory_xact_lock(
            hashtextextended(
                'tepp:lifecycle:tenant:' || NEW.tenant_record_id::text,
                0
            )
        );
        PERFORM pg_advisory_xact_lock(
            hashtextextended(
                'tepp:lifecycle:document:'
                    || NEW.tenant_record_id::text
                    || ':'
                    || NEW.target_document_id::text,
                0
            )
        );
    END IF;
"""
sql = replace_once(sql, delete_lock_old, delete_lock_new, "deletion advisory locks")

guard_old = """CREATE OR REPLACE FUNCTION guard_legal_hold_insert()
RETURNS trigger
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
DECLARE
    tenant_context text;
    lock_document uuid;
BEGIN
    tenant_context := nullif(current_setting('tepp.current_tenant_record_id', true), '');
    IF tenant_context IS NULL OR NEW.tenant_record_id::text <> tenant_context THEN
        RAISE EXCEPTION 'tenant session context is required for lifecycle mutation'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF NEW.hold_status_code <> 'active' OR NEW.system_to IS NOT NULL THEN
        RAISE EXCEPTION 'legal hold insert must be an open active hold'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    lock_document := COALESCE(NEW.held_document_id, '00000000-0000-0000-0000-000000000000'::uuid);
    PERFORM pg_advisory_xact_lock(
        hashtextextended(NEW.tenant_record_id::text, 0),
        hashtextextended(lock_document::text, 0)
    );
    RETURN NEW;
END;
$$;
"""
guard_new = """CREATE OR REPLACE FUNCTION guard_legal_hold_insert()
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
    IF NEW.hold_status_code <> 'active' OR NEW.system_to IS NOT NULL THEN
        RAISE EXCEPTION 'legal hold insert must be an open active hold'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF NEW.hold_scope_code NOT IN ('document', 'tenant')
        OR (NEW.hold_scope_code = 'document' AND NEW.held_document_id IS NULL)
        OR (NEW.hold_scope_code = 'tenant' AND NEW.held_document_id IS NOT NULL)
    THEN
        RAISE EXCEPTION 'legal hold scope is inconsistent'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;

    -- Completed deletions and legal holds always acquire the tenant lock first.
    -- Document-scoped operations then acquire the document lock, eliminating
    -- hold/deletion TOCTOU races without relying on a nonexistent two-bigint
    -- advisory-lock overload.
    PERFORM pg_advisory_xact_lock(
        hashtextextended(
            'tepp:lifecycle:tenant:' || NEW.tenant_record_id::text,
            0
        )
    );
    IF NEW.hold_scope_code = 'document' THEN
        PERFORM pg_advisory_xact_lock(
            hashtextextended(
                'tepp:lifecycle:document:'
                    || NEW.tenant_record_id::text
                    || ':'
                    || NEW.held_document_id::text,
                0
            )
        );
    END IF;

    -- The deletion trigger checks newly committed holds. This symmetric check
    -- covers the opposite serialization order: a completed deletion that
    -- committed first cannot coexist with a hold effective at or before it.
    IF EXISTS (
        SELECT 1
        FROM public.deletion_request AS completed_deletion
        WHERE completed_deletion.tenant_record_id = NEW.tenant_record_id
          AND completed_deletion.request_status_code = 'completed'
          AND completed_deletion.system_time >= NEW.system_time
          AND (
              NEW.hold_scope_code = 'tenant'
              OR completed_deletion.target_document_id = NEW.held_document_id
          )
    ) THEN
        RAISE EXCEPTION 'completed deletion blocks legal hold activation'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;

    RETURN NEW;
END;
$$;
"""
sql = replace_once(sql, guard_old, guard_new, "legal-hold insert guard")

release_declare_old = """DECLARE
    tenant_context text;
    current_hold legal_hold%ROWTYPE;
    lock_document uuid;
BEGIN
"""
release_declare_new = """DECLARE
    tenant_context text;
    current_hold legal_hold%ROWTYPE;
BEGIN
"""
sql = replace_once(sql, release_declare_old, release_declare_new, "release declaration")

release_lock_old = """    lock_document := COALESCE(current_hold.held_document_id, '00000000-0000-0000-0000-000000000000'::uuid);
    PERFORM pg_advisory_xact_lock(
        hashtextextended(current_hold.tenant_record_id::text, 0),
        hashtextextended(lock_document::text, 0)
    );
"""
release_lock_new = """    PERFORM pg_advisory_xact_lock(
        hashtextextended(
            'tepp:lifecycle:tenant:' || current_hold.tenant_record_id::text,
            0
        )
    );
    IF current_hold.hold_scope_code = 'document' THEN
        PERFORM pg_advisory_xact_lock(
            hashtextextended(
                'tepp:lifecycle:document:'
                    || current_hold.tenant_record_id::text
                    || ':'
                    || current_hold.held_document_id::text,
                0
            )
        );
    END IF;
"""
sql = replace_once(sql, release_lock_old, release_lock_new, "release advisory locks")

path.write_text(sql, encoding="utf-8")
