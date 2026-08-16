-- TEPP retention, deletion, and legal hold (ADR 0009 / ADR 0013).
-- Policy-driven lifecycle is append-only except for one controlled system-time
-- succession transition. Completed deletion cannot override an active legal
-- hold, and a tombstone blocks restore of the same document identity.
-- Tombstones store action digests, never raw source text.

CREATE TABLE retention_policy (
    retention_policy_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    data_class_code text NOT NULL,
    processing_purpose_code text NOT NULL,
    retention_period_days integer NOT NULL,
    policy_status_code text NOT NULL,
    authority_citation text NOT NULL,
    system_time timestamptz NOT NULL,
    system_to timestamptz,
    available_time timestamptz NOT NULL,
    supersedes_retention_policy_id uuid REFERENCES retention_policy (retention_policy_id),
    CONSTRAINT retention_policy_period_positive CHECK (retention_period_days > 0),
    CONSTRAINT retention_policy_status_known CHECK (
        policy_status_code IN ('active', 'superseded')
    ),
    CONSTRAINT retention_policy_system_order CHECK (
        system_to IS NULL OR system_time <= system_to
    ),
    CONSTRAINT retention_policy_status_window_consistent CHECK (
        (policy_status_code = 'active' AND system_to IS NULL)
        OR (policy_status_code = 'superseded' AND system_to IS NOT NULL)
    )
);

CREATE TABLE legal_hold (
    legal_hold_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    hold_scope_code text NOT NULL,
    held_document_id uuid,
    hold_authority_code text NOT NULL,
    hold_status_code text NOT NULL,
    authority_citation text NOT NULL,
    system_time timestamptz NOT NULL,
    system_to timestamptz,
    available_time timestamptz NOT NULL,
    CONSTRAINT legal_hold_status_known CHECK (
        hold_status_code IN ('active', 'released')
    ),
    CONSTRAINT legal_hold_system_order CHECK (
        system_to IS NULL OR system_time <= system_to
    ),
    CONSTRAINT legal_hold_status_window_consistent CHECK (
        (hold_status_code = 'active' AND system_to IS NULL)
        OR (hold_status_code = 'released' AND system_to IS NOT NULL)
    ),
    CONSTRAINT legal_hold_document_scope_consistent CHECK (
        (hold_scope_code = 'document' AND held_document_id IS NOT NULL)
        OR (hold_scope_code = 'tenant' AND held_document_id IS NULL)
    )
);

CREATE TABLE deletion_request (
    deletion_request_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    retention_policy_id uuid NOT NULL REFERENCES retention_policy (retention_policy_id),
    target_document_id uuid NOT NULL,
    target_data_class_code text NOT NULL,
    processing_purpose_code text NOT NULL,
    deletion_kind_code text NOT NULL,
    request_status_code text NOT NULL,
    legal_hold_id uuid REFERENCES legal_hold (legal_hold_id),
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
    evidence_tombstone_id uuid PRIMARY KEY,
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    tombstoned_document_id uuid NOT NULL,
    deletion_request_id uuid NOT NULL REFERENCES deletion_request (deletion_request_id),
    evidence_digest text NOT NULL,
    target_data_class_code text NOT NULL,
    deletion_kind_code text NOT NULL,
    reproduction_status_code text NOT NULL,
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
    ON retention_policy (tenant_record_id, data_class_code, processing_purpose_code)
    WHERE policy_status_code = 'active' AND system_to IS NULL;

CREATE UNIQUE INDEX retention_policy_successor_unique
    ON retention_policy (supersedes_retention_policy_id)
    WHERE supersedes_retention_policy_id IS NOT NULL;

CREATE UNIQUE INDEX legal_hold_active_document_unique
    ON legal_hold (tenant_record_id, held_document_id)
    WHERE hold_status_code = 'active'
      AND system_to IS NULL
      AND hold_scope_code = 'document'
      AND held_document_id IS NOT NULL;

CREATE UNIQUE INDEX legal_hold_active_tenant_unique
    ON legal_hold (tenant_record_id)
    WHERE hold_status_code = 'active'
      AND system_to IS NULL
      AND hold_scope_code = 'tenant';

CREATE INDEX evidence_tombstone_document_lookup
    ON evidence_tombstone (tenant_record_id, tombstoned_document_id);

CREATE INDEX legal_hold_active_scope_lookup
    ON legal_hold (tenant_record_id, hold_scope_code, held_document_id)
    WHERE hold_status_code = 'active' AND system_to IS NULL;

CREATE INDEX deletion_request_retention_policy_id_idx
    ON deletion_request (retention_policy_id);

CREATE INDEX deletion_request_legal_hold_id_idx
    ON deletion_request (legal_hold_id)
    WHERE legal_hold_id IS NOT NULL;

CREATE INDEX evidence_tombstone_deletion_request_id_idx
    ON evidence_tombstone (deletion_request_id);

CREATE INDEX deletion_request_completed_scope_lookup
    ON deletion_request (tenant_record_id, system_time, target_document_id)
    WHERE request_status_code = 'completed';

CREATE OR REPLACE FUNCTION enforce_retention_policy_succession()
RETURNS trigger
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'retention policy history is append-only'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;

    IF OLD.policy_status_code <> 'active'
        OR OLD.system_to IS NOT NULL
        OR NEW.policy_status_code <> 'superseded'
        OR NEW.system_to IS NULL
        OR NEW.system_to < OLD.system_time
        OR NEW.retention_policy_id IS DISTINCT FROM OLD.retention_policy_id
        OR NEW.tenant_record_id IS DISTINCT FROM OLD.tenant_record_id
        OR NEW.data_class_code IS DISTINCT FROM OLD.data_class_code
        OR NEW.processing_purpose_code IS DISTINCT FROM OLD.processing_purpose_code
        OR NEW.retention_period_days IS DISTINCT FROM OLD.retention_period_days
        OR NEW.authority_citation IS DISTINCT FROM OLD.authority_citation
        OR NEW.system_time IS DISTINCT FROM OLD.system_time
        OR NEW.available_time IS DISTINCT FROM OLD.available_time
        OR NEW.supersedes_retention_policy_id IS DISTINCT FROM OLD.supersedes_retention_policy_id
    THEN
        RAISE EXCEPTION 'retention policy mutation must be a controlled succession'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER retention_policy_enforce_succession
    BEFORE UPDATE OR DELETE ON retention_policy
    FOR EACH ROW
    EXECUTE FUNCTION enforce_retention_policy_succession();

CREATE TRIGGER retention_policy_reject_truncate
    BEFORE TRUNCATE ON retention_policy
    FOR EACH STATEMENT
    EXECUTE FUNCTION reject_append_only_mutation();

CREATE OR REPLACE FUNCTION supersede_retention_policy(
    current_retention_policy_id uuid,
    replacement_retention_policy_id uuid,
    replacement_retention_period_days integer,
    replacement_authority_citation text,
    replacement_system_time timestamptz,
    replacement_available_time timestamptz
)
RETURNS uuid
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
DECLARE
    tenant_context text;
    current_policy retention_policy%ROWTYPE;
BEGIN
    tenant_context := nullif(current_setting('tepp.current_tenant_record_id', true), '');
    IF tenant_context IS NULL THEN
        RAISE EXCEPTION 'tenant session context is required for retention policy succession'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF current_retention_policy_id = replacement_retention_policy_id
        OR replacement_retention_period_days <= 0
        OR replacement_authority_citation IS NULL
        OR btrim(replacement_authority_citation) = ''
    THEN
        RAISE EXCEPTION 'invalid retention policy successor'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;

    SELECT *
    INTO current_policy
    FROM public.retention_policy
    WHERE retention_policy_id = current_retention_policy_id
    FOR UPDATE;

    IF NOT FOUND
        OR current_policy.tenant_record_id::text <> tenant_context
        OR current_policy.policy_status_code <> 'active'
        OR current_policy.system_to IS NOT NULL
        OR replacement_system_time < current_policy.system_time
        OR replacement_available_time < current_policy.available_time
    THEN
        RAISE EXCEPTION 'retention policy successor is not authorized for the active version'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;

    UPDATE public.retention_policy
    SET policy_status_code = 'superseded',
        system_to = replacement_system_time
    WHERE retention_policy_id = current_retention_policy_id;

    INSERT INTO public.retention_policy (
        retention_policy_id,
        tenant_record_id,
        data_class_code,
        processing_purpose_code,
        retention_period_days,
        policy_status_code,
        authority_citation,
        system_time,
        system_to,
        available_time,
        supersedes_retention_policy_id
    ) VALUES (
        replacement_retention_policy_id,
        current_policy.tenant_record_id,
        current_policy.data_class_code,
        current_policy.processing_purpose_code,
        replacement_retention_period_days,
        'active',
        replacement_authority_citation,
        replacement_system_time,
        NULL,
        replacement_available_time,
        current_retention_policy_id
    );

    RETURN replacement_retention_policy_id;
END;
$$;

CREATE OR REPLACE FUNCTION reject_held_evidence_deletion()
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
    IF NEW.request_status_code = 'completed' THEN
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
    IF NEW.request_status_code = 'completed' AND EXISTS (
        SELECT 1
        FROM public.legal_hold
        WHERE tenant_record_id = NEW.tenant_record_id
          AND hold_status_code = 'active'
          AND system_to IS NULL
          AND (
              hold_scope_code = 'tenant'
              OR (
                  hold_scope_code = 'document'
                  AND held_document_id = NEW.target_document_id
              )
          )
    ) THEN
        RAISE EXCEPTION 'legal hold blocks deletion'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER deletion_request_reject_held_deletion
    BEFORE INSERT ON deletion_request
    FOR EACH ROW
    EXECUTE FUNCTION reject_held_evidence_deletion();

CREATE OR REPLACE FUNCTION guard_deletion_request_policy()
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
        FROM public.retention_policy
        WHERE retention_policy_id = NEW.retention_policy_id
          AND tenant_record_id = NEW.tenant_record_id
          AND data_class_code = NEW.target_data_class_code
          AND processing_purpose_code = NEW.processing_purpose_code
    ) THEN
        RAISE EXCEPTION 'deletion request must match cited retention policy'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER deletion_request_guard_policy
    BEFORE INSERT ON deletion_request
    FOR EACH ROW
    EXECUTE FUNCTION guard_deletion_request_policy();

CREATE OR REPLACE FUNCTION reject_tombstoned_evidence_restore()
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
    IF EXISTS (
        SELECT 1
        FROM public.evidence_tombstone
        WHERE tenant_record_id = NEW.tenant_record_id
          AND tombstoned_document_id = NEW.document_record_id
    ) THEN
        RAISE EXCEPTION 'tombstoned evidence cannot be restored'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER document_record_reject_tombstone_restore
    BEFORE INSERT OR UPDATE ON document_record
    FOR EACH ROW
    EXECUTE FUNCTION reject_tombstoned_evidence_restore();

CREATE OR REPLACE FUNCTION guard_evidence_tombstone_insert()
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

GRANT SELECT, INSERT ON TABLE retention_policy TO tepp_app_runtime;
GRANT SELECT, INSERT ON TABLE legal_hold TO tepp_app_runtime;
GRANT SELECT, INSERT ON TABLE deletion_request TO tepp_app_runtime;
GRANT SELECT, INSERT ON TABLE evidence_tombstone TO tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE retention_policy FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE retention_policy FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE legal_hold FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE legal_hold FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE deletion_request FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE deletion_request FROM tepp_app_runtime;
REVOKE UPDATE, DELETE ON TABLE evidence_tombstone FROM tepp_app_runtime;
REVOKE TRUNCATE ON TABLE evidence_tombstone FROM tepp_app_runtime;

CREATE OR REPLACE FUNCTION guard_legal_hold_insert()
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

CREATE OR REPLACE FUNCTION enforce_legal_hold_release()
RETURNS trigger
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'legal hold history is append-only'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF OLD.hold_status_code <> 'active'
        OR OLD.system_to IS NOT NULL
        OR NEW.hold_status_code <> 'released'
        OR NEW.system_to IS NULL
        OR NEW.system_to < OLD.system_time
        OR NEW.legal_hold_id IS DISTINCT FROM OLD.legal_hold_id
        OR NEW.tenant_record_id IS DISTINCT FROM OLD.tenant_record_id
        OR NEW.hold_scope_code IS DISTINCT FROM OLD.hold_scope_code
        OR NEW.held_document_id IS DISTINCT FROM OLD.held_document_id
        OR NEW.hold_authority_code IS DISTINCT FROM OLD.hold_authority_code
        OR NEW.authority_citation IS DISTINCT FROM OLD.authority_citation
        OR NEW.system_time IS DISTINCT FROM OLD.system_time
        OR NEW.available_time IS DISTINCT FROM OLD.available_time
    THEN
        RAISE EXCEPTION 'legal hold mutation must be a controlled release'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION release_legal_hold(
    target_legal_hold_id uuid,
    release_system_time timestamptz
)
RETURNS uuid
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
DECLARE
    tenant_context text;
    current_hold legal_hold%ROWTYPE;
BEGIN
    tenant_context := nullif(current_setting('tepp.current_tenant_record_id', true), '');
    IF tenant_context IS NULL THEN
        RAISE EXCEPTION 'tenant session context is required for legal hold release'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;

    SELECT *
    INTO current_hold
    FROM public.legal_hold
    WHERE legal_hold_id = target_legal_hold_id
    FOR UPDATE;

    IF NOT FOUND
        OR current_hold.tenant_record_id::text <> tenant_context
        OR current_hold.hold_status_code <> 'active'
        OR current_hold.system_to IS NOT NULL
        OR release_system_time < current_hold.system_time
    THEN
        RAISE EXCEPTION 'legal hold release is not authorized for the active version'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;

    PERFORM pg_advisory_xact_lock(
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

    UPDATE public.legal_hold
    SET hold_status_code = 'released',
        system_to = release_system_time
    WHERE legal_hold_id = target_legal_hold_id;

    RETURN target_legal_hold_id;
END;
$$;

CREATE TRIGGER legal_hold_guard_insert
    BEFORE INSERT ON legal_hold
    FOR EACH ROW
    EXECUTE FUNCTION guard_legal_hold_insert();
CREATE TRIGGER legal_hold_enforce_release
    BEFORE UPDATE OR DELETE ON legal_hold
    FOR EACH ROW
    EXECUTE FUNCTION enforce_legal_hold_release();
CREATE TRIGGER legal_hold_reject_mutation
    BEFORE TRUNCATE ON legal_hold
    FOR EACH STATEMENT
    EXECUTE FUNCTION reject_append_only_mutation();
CREATE TRIGGER deletion_request_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON deletion_request
    FOR EACH STATEMENT
    EXECUTE FUNCTION reject_append_only_mutation();
CREATE TRIGGER evidence_tombstone_reject_mutation
    BEFORE UPDATE OR DELETE OR TRUNCATE ON evidence_tombstone
    FOR EACH STATEMENT
    EXECUTE FUNCTION reject_append_only_mutation();

ALTER TABLE retention_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE retention_policy FORCE ROW LEVEL SECURITY;
CREATE POLICY retention_policy_tenant_isolation ON retention_policy
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE legal_hold ENABLE ROW LEVEL SECURITY;
ALTER TABLE legal_hold FORCE ROW LEVEL SECURITY;
CREATE POLICY legal_hold_tenant_isolation ON legal_hold
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE deletion_request ENABLE ROW LEVEL SECURITY;
ALTER TABLE deletion_request FORCE ROW LEVEL SECURITY;
CREATE POLICY deletion_request_tenant_isolation ON deletion_request
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

ALTER TABLE evidence_tombstone ENABLE ROW LEVEL SECURITY;
ALTER TABLE evidence_tombstone FORCE ROW LEVEL SECURITY;
CREATE POLICY evidence_tombstone_tenant_isolation ON evidence_tombstone
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

REVOKE ALL ON FUNCTION enforce_retention_policy_succession() FROM PUBLIC;
REVOKE ALL ON FUNCTION supersede_retention_policy(uuid, uuid, integer, text, timestamptz, timestamptz) FROM PUBLIC;
REVOKE ALL ON FUNCTION reject_held_evidence_deletion() FROM PUBLIC;
REVOKE ALL ON FUNCTION reject_tombstoned_evidence_restore() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION supersede_retention_policy(uuid, uuid, integer, text, timestamptz, timestamptz) TO tepp_app_runtime;
REVOKE ALL ON FUNCTION release_legal_hold(uuid, timestamptz) FROM PUBLIC;
REVOKE ALL ON FUNCTION enforce_legal_hold_release() FROM PUBLIC;
REVOKE ALL ON FUNCTION guard_legal_hold_insert() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION release_legal_hold(uuid, timestamptz) TO tepp_app_runtime;
REVOKE ALL ON FUNCTION guard_evidence_tombstone_insert() FROM PUBLIC;
REVOKE ALL ON FUNCTION guard_deletion_request_policy() FROM PUBLIC;
