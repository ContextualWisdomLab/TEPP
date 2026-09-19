-- Preserve the Membership owner's exact same-role share budget at the PostgreSQL boundary.
--
-- `membership_weight` remains NUMERIC for compatibility with the existing schema, but budget
-- admission canonicalizes every stored input through PostgreSQL double precision and then reasons
-- over the exact represented binary64 value. This matches the Rust Membership owner without
-- replacing scientific identity with decimal SUM or ordinary floating accumulation.
--
-- A narrow guard row serializes writers for one tenant/observed-unit/role lane before the
-- pointwise temporal aggregate and duplicate-edge predicates are evaluated. Admission protection
-- is installed before the historical scan so predecessor writes cannot slip through a validation
-- to trigger cutover gap.

CREATE OR REPLACE FUNCTION membership_binary64_scaled_numerator(weight numeric)
RETURNS numeric
LANGUAGE plpgsql
IMMUTABLE
STRICT
AS $membership_binary64_scaled_numerator$
DECLARE
    payload bytea;
    raw_bits bigint;
    raw_exponent integer;
    fraction bigint;
    significand numeric;
    exponent_shift integer;
    factor numeric := 1;
    factor_base numeric := 2;
    factor_exponent integer;
BEGIN
    IF weight <= 0 OR weight > 1 THEN
        RAISE EXCEPTION 'membership weight is outside (0, 1]'
            USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_weight_unit_interval';
    END IF;

    payload := pg_catalog.float8send(weight::double precision);
    raw_bits :=
          (get_byte(payload, 0)::bigint << 56)
        | (get_byte(payload, 1)::bigint << 48)
        | (get_byte(payload, 2)::bigint << 40)
        | (get_byte(payload, 3)::bigint << 32)
        | (get_byte(payload, 4)::bigint << 24)
        | (get_byte(payload, 5)::bigint << 16)
        | (get_byte(payload, 6)::bigint << 8)
        |  get_byte(payload, 7)::bigint;

    IF raw_bits <= 0 OR raw_bits > 4607182418800017408 THEN
        RAISE EXCEPTION 'membership weight is not representable in the owner binary64 domain'
            USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_weight_unit_interval';
    END IF;

    raw_exponent := ((raw_bits >> 52) & 2047)::integer;
    fraction := raw_bits & 4503599627370495;
    IF raw_exponent = 0 THEN
        RETURN fraction::numeric;
    END IF;

    significand := (4503599627370496 + fraction)::numeric;
    exponent_shift := raw_exponent - 1;
    factor_exponent := exponent_shift;

    WHILE factor_exponent > 0 LOOP
        IF factor_exponent % 2 = 1 THEN
            factor := factor * factor_base;
        END IF;
        factor_exponent := factor_exponent / 2;
        IF factor_exponent > 0 THEN
            factor_base := factor_base * factor_base;
        END IF;
    END LOOP;

    RETURN significand * factor;
END;
$membership_binary64_scaled_numerator$;

CREATE OR REPLACE FUNCTION membership_possible_activity_envelope(
    valid_from_window tstzrange,
    valid_to_window tstzrange
)
RETURNS tstzrange
LANGUAGE sql
IMMUTABLE
AS $membership_possible_activity_envelope$
    SELECT tstzrange(
        lower(valid_from_window),
        CASE WHEN valid_to_window IS NULL THEN NULL ELSE upper(valid_to_window) END,
        (CASE WHEN lower_inc(valid_from_window) THEN '[' ELSE '(' END)
        ||
        (CASE
            WHEN valid_to_window IS NOT NULL AND upper_inc(valid_to_window) THEN ']'
            ELSE ')'
        END)
    )
$membership_possible_activity_envelope$;

CREATE OR REPLACE FUNCTION membership_same_role_max_existing_numerator(
    candidate_tenant_record_id uuid,
    candidate_observed_unit_kind text,
    candidate_observed_unit_id uuid,
    candidate_membership_type_code text,
    candidate_envelope tstzrange,
    excluded_membership_assignment_id uuid
)
RETURNS numeric
LANGUAGE sql
STABLE
AS $membership_same_role_max_existing_numerator$
    WITH lane_memberships AS (
        SELECT
            membership_binary64_scaled_numerator(existing.membership_weight) AS numerator,
            membership_possible_activity_envelope(
                existing.valid_from_window,
                existing.valid_to_window
            ) AS envelope
        FROM membership_assignment AS existing
        WHERE existing.tenant_record_id = candidate_tenant_record_id
          AND existing.membership_type_code = candidate_membership_type_code
          AND existing.membership_assignment_id <> excluded_membership_assignment_id
          AND (
                (candidate_observed_unit_kind = 'document'
                 AND existing.document_record_id = candidate_observed_unit_id
                 AND existing.text_segment_id IS NULL)
             OR (candidate_observed_unit_kind = 'text_segment'
                 AND existing.text_segment_id = candidate_observed_unit_id
                 AND existing.document_record_id IS NULL)
          )
    ),
    relevant AS (
        SELECT numerator, envelope
        FROM lane_memberships
        WHERE envelope && candidate_envelope
    ),
    boundaries AS (
        SELECT lower(candidate_envelope) AS boundary
        WHERE NOT lower_inf(candidate_envelope)
        UNION
        SELECT upper(candidate_envelope) AS boundary
        WHERE NOT upper_inf(candidate_envelope)
        UNION
        SELECT lower(envelope) AS boundary
        FROM relevant
        WHERE NOT lower_inf(envelope)
        UNION
        SELECT upper(envelope) AS boundary
        FROM relevant
        WHERE NOT upper_inf(envelope)
    ),
    point_states AS (
        SELECT COALESCE(SUM(relevant.numerator), 0) AS numerator
        FROM boundaries
        LEFT JOIN relevant ON relevant.envelope @> boundaries.boundary
        WHERE candidate_envelope @> boundaries.boundary
        GROUP BY boundaries.boundary
    ),
    right_open_states AS (
        SELECT COALESCE(SUM(relevant.numerator), 0) AS numerator
        FROM boundaries
        LEFT JOIN relevant
          ON (lower_inf(relevant.envelope) OR lower(relevant.envelope) <= boundaries.boundary)
         AND (upper_inf(relevant.envelope) OR upper(relevant.envelope) > boundaries.boundary)
        WHERE (lower_inf(candidate_envelope) OR lower(candidate_envelope) <= boundaries.boundary)
          AND (upper_inf(candidate_envelope) OR upper(candidate_envelope) > boundaries.boundary)
        GROUP BY boundaries.boundary
    ),
    lower_unbounded_state AS (
        SELECT COALESCE(SUM(relevant.numerator), 0) AS numerator
        FROM relevant
        WHERE lower_inf(candidate_envelope)
          AND lower_inf(relevant.envelope)
    ),
    states AS (
        SELECT numerator FROM point_states
        UNION ALL
        SELECT numerator FROM right_open_states
        UNION ALL
        SELECT numerator FROM lower_unbounded_state
    )
    SELECT COALESCE(MAX(numerator), 0)
    FROM states
$membership_same_role_max_existing_numerator$;

CREATE OR REPLACE FUNCTION membership_duplicate_temporal_edge_exists(
    candidate_tenant_record_id uuid,
    candidate_observed_unit_kind text,
    candidate_observed_unit_id uuid,
    candidate_target_entity_id uuid,
    candidate_target_project_id uuid,
    candidate_membership_type_code text,
    candidate_envelope tstzrange,
    excluded_membership_assignment_id uuid
)
RETURNS boolean
LANGUAGE sql
STABLE
AS $membership_duplicate_temporal_edge_exists$
    SELECT EXISTS (
        SELECT 1
        FROM membership_assignment AS existing
        WHERE existing.tenant_record_id = candidate_tenant_record_id
          AND existing.membership_type_code = candidate_membership_type_code
          AND existing.membership_assignment_id <> excluded_membership_assignment_id
          AND (
                (candidate_observed_unit_kind = 'document'
                 AND existing.document_record_id = candidate_observed_unit_id
                 AND existing.text_segment_id IS NULL)
             OR (candidate_observed_unit_kind = 'text_segment'
                 AND existing.text_segment_id = candidate_observed_unit_id
                 AND existing.document_record_id IS NULL)
          )
          AND (
                (candidate_target_entity_id IS NOT NULL
                 AND candidate_target_project_id IS NULL
                 AND existing.target_entity_id = candidate_target_entity_id
                 AND existing.target_project_id IS NULL)
             OR (candidate_target_project_id IS NOT NULL
                 AND candidate_target_entity_id IS NULL
                 AND existing.target_project_id = candidate_target_project_id
                 AND existing.target_entity_id IS NULL)
          )
          AND membership_possible_activity_envelope(
                existing.valid_from_window,
                existing.valid_to_window
              ) && candidate_envelope
    )
$membership_duplicate_temporal_edge_exists$;

CREATE TABLE IF NOT EXISTS membership_share_budget_guard (
    tenant_record_id uuid NOT NULL REFERENCES tenant_record (tenant_record_id),
    observed_unit_kind text NOT NULL,
    observed_unit_id uuid NOT NULL,
    membership_type_code text NOT NULL,
    system_time timestamptz NOT NULL DEFAULT statement_timestamp(),
    available_time timestamptz NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (tenant_record_id, observed_unit_kind, observed_unit_id, membership_type_code),
    CONSTRAINT membership_share_budget_guard_unit_kind CHECK (
        observed_unit_kind IN ('document', 'text_segment')
    )
);

GRANT SELECT, INSERT ON TABLE membership_share_budget_guard TO tepp_app_runtime;
GRANT UPDATE (system_time) ON TABLE membership_share_budget_guard TO tepp_app_runtime;

ALTER TABLE membership_share_budget_guard ENABLE ROW LEVEL SECURITY;
ALTER TABLE membership_share_budget_guard FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS membership_share_budget_guard_tenant_isolation ON membership_share_budget_guard;
CREATE POLICY membership_share_budget_guard_tenant_isolation ON membership_share_budget_guard
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

CREATE OR REPLACE FUNCTION enforce_membership_same_role_share_budget()
RETURNS trigger
LANGUAGE plpgsql
VOLATILE
AS $enforce_membership_same_role_share_budget$
DECLARE
    observed_kind text;
    observed_id uuid;
    candidate_envelope tstzrange;
    existing_numerator numeric;
    candidate_numerator numeric;
    unity_numerator numeric;
BEGIN
    IF NEW.document_record_id IS NOT NULL THEN
        observed_kind := 'document';
        observed_id := NEW.document_record_id;
    ELSIF NEW.text_segment_id IS NOT NULL THEN
        observed_kind := 'text_segment';
        observed_id := NEW.text_segment_id;
    ELSE
        RAISE EXCEPTION 'membership assignment has no observed unit'
            USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_observed_unit_exactly_one';
    END IF;

    candidate_numerator := membership_binary64_scaled_numerator(NEW.membership_weight);
    unity_numerator := membership_binary64_scaled_numerator(1::numeric);

    INSERT INTO membership_share_budget_guard (
        tenant_record_id,
        observed_unit_kind,
        observed_unit_id,
        membership_type_code,
        system_time,
        available_time
    ) VALUES (
        NEW.tenant_record_id,
        observed_kind,
        observed_id,
        NEW.membership_type_code,
        statement_timestamp(),
        statement_timestamp()
    )
    ON CONFLICT (tenant_record_id, observed_unit_kind, observed_unit_id, membership_type_code)
    DO UPDATE SET system_time = membership_share_budget_guard.system_time;

    candidate_envelope := membership_possible_activity_envelope(
        NEW.valid_from_window,
        NEW.valid_to_window
    );

    IF membership_duplicate_temporal_edge_exists(
        NEW.tenant_record_id,
        observed_kind,
        observed_id,
        NEW.target_entity_id,
        NEW.target_project_id,
        NEW.membership_type_code,
        candidate_envelope,
        NEW.membership_assignment_id
    ) THEN
        RAISE EXCEPTION 'duplicate membership temporal edge overlaps'
            USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_duplicate_temporal_edge';
    END IF;

    existing_numerator := membership_same_role_max_existing_numerator(
        NEW.tenant_record_id,
        observed_kind,
        observed_id,
        NEW.membership_type_code,
        candidate_envelope,
        NEW.membership_assignment_id
    );

    IF existing_numerator + candidate_numerator > unity_numerator THEN
        RAISE EXCEPTION 'overlapping same-role membership shares exceed unity'
            USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_same_role_share_budget';
    END IF;

    RETURN NEW;
END;
$enforce_membership_same_role_share_budget$;

DROP TRIGGER IF EXISTS membership_assignment_same_role_share_budget ON membership_assignment;
CREATE TRIGGER membership_assignment_same_role_share_budget
BEFORE INSERT OR UPDATE OF
    tenant_record_id,
    document_record_id,
    text_segment_id,
    target_entity_id,
    target_project_id,
    membership_type_code,
    membership_weight,
    valid_from_window,
    valid_to_window
ON membership_assignment
FOR EACH ROW
EXECUTE FUNCTION enforce_membership_same_role_share_budget();

-- Historical validation runs only after future writes are protected. This deliberately mirrors the
-- operational shape of `NOT VALID -> VALIDATE`: an invalid predecessor dataset leaves the new
-- admission gate installed, fails the migration, and can be repaired by the data owner before an
-- idempotent retry. No long explicit table lock is held across the scientific scan.
DO $membership_existing_share_budget_validation$
DECLARE
    existing membership_assignment%ROWTYPE;
    observed_kind text;
    observed_id uuid;
    candidate_envelope tstzrange;
    existing_numerator numeric;
    candidate_numerator numeric;
    unity_numerator numeric := membership_binary64_scaled_numerator(1::numeric);
BEGIN
    FOR existing IN SELECT * FROM membership_assignment LOOP
        candidate_numerator := membership_binary64_scaled_numerator(existing.membership_weight);
        candidate_envelope := membership_possible_activity_envelope(
            existing.valid_from_window,
            existing.valid_to_window
        );

        IF existing.document_record_id IS NOT NULL THEN
            observed_kind := 'document';
            observed_id := existing.document_record_id;
        ELSIF existing.text_segment_id IS NOT NULL THEN
            observed_kind := 'text_segment';
            observed_id := existing.text_segment_id;
        ELSE
            RAISE EXCEPTION 'membership assignment has no observed unit'
                USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_observed_unit_exactly_one';
        END IF;

        IF membership_duplicate_temporal_edge_exists(
            existing.tenant_record_id,
            observed_kind,
            observed_id,
            existing.target_entity_id,
            existing.target_project_id,
            existing.membership_type_code,
            candidate_envelope,
            existing.membership_assignment_id
        ) THEN
            RAISE EXCEPTION 'pre-existing duplicate membership temporal edge overlaps'
                USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_duplicate_temporal_edge';
        END IF;

        existing_numerator := membership_same_role_max_existing_numerator(
            existing.tenant_record_id,
            observed_kind,
            observed_id,
            existing.membership_type_code,
            candidate_envelope,
            existing.membership_assignment_id
        );

        IF existing_numerator + candidate_numerator > unity_numerator THEN
            RAISE EXCEPTION 'pre-existing overlapping same-role membership shares exceed unity'
                USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_same_role_share_budget';
        END IF;
    END LOOP;
END;
$membership_existing_share_budget_validation$;
