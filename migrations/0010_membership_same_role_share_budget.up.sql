-- Preserve the Membership owner's exact same-role share budget at the PostgreSQL boundary.
--
-- `membership_weight` remains NUMERIC for compatibility with the existing schema, but budget
-- admission canonicalizes every stored input through PostgreSQL double precision and then reasons
-- over the exact represented binary64 value. This matches the Rust Membership owner without
-- replacing scientific identity with decimal SUM or ordinary floating accumulation.
--
-- A narrow guard row serializes writers for one tenant/observed-unit/role lane before the
-- overlapping-row aggregate is evaluated. This closes the initially-empty-row race that a plain
-- SELECT/trigger check cannot prevent.

CREATE TABLE membership_share_budget_guard (
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
CREATE POLICY membership_share_budget_guard_tenant_isolation ON membership_share_budget_guard
    FOR ALL
    USING (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    )
    WITH CHECK (
        tenant_record_id::text = nullif(current_setting('tepp.current_tenant_record_id', true), '')
    );

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

    -- float8send exposes the canonical IEEE-754 binary64 payload selected by PostgreSQL's
    -- numeric->double precision conversion. The owner contract is defined on that represented
    -- value, not on the source decimal spelling.
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

    -- Positive finite binary64 values <= 1.0 have monotonically ordered positive bit patterns.
    -- Zero covers decimal values that underflow the owner's smallest positive binary64 share.
    IF raw_bits <= 0 OR raw_bits > 4607182418800017408 THEN
        RAISE EXCEPTION 'membership weight is not representable in the owner binary64 domain'
            USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_weight_unit_interval';
    END IF;

    raw_exponent := ((raw_bits >> 52) & 2047)::integer;
    fraction := raw_bits & 4503599627370495;
    IF raw_exponent = 0 THEN
        -- Subnormal: exact value = fraction * 2^-1074, so the scaled numerator is `fraction`.
        RETURN fraction::numeric;
    END IF;

    significand := (4503599627370496 + fraction)::numeric;
    exponent_shift := raw_exponent - 1;
    factor_exponent := exponent_shift;

    -- Compute 2^exponent_shift with exact NUMERIC integer multiplication and logarithmic work.
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

CREATE OR REPLACE FUNCTION enforce_membership_same_role_share_budget()
RETURNS trigger
LANGUAGE plpgsql
VOLATILE
AS $enforce_membership_same_role_share_budget$
DECLARE
    observed_kind text;
    observed_id uuid;
    candidate_start timestamptz;
    candidate_end timestamptz;
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

    -- The guard row is durable coordination state. ON CONFLICT DO UPDATE obtains a row lock that
    -- survives until transaction end; concurrent first writers therefore cannot both observe an
    -- empty membership lane. At stronger MVCC isolation a stale conflicting row update fails via
    -- PostgreSQL serialization semantics instead of admitting an unverified aggregate.
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

    -- `0006` intentionally permits non-empty uncertainty windows with unbounded sides. PostgreSQL
    -- returns NULL from lower()/upper() for those sides; letting NULL reach the overlap predicate
    -- would turn a real possible overlap into SQL UNKNOWN and omit the row from the budget. Map only
    -- those schema-admitted unbounded envelope sides to temporal infinities for conservative
    -- admission. Empty ranges are already rejected by `0006`.
    candidate_start := COALESCE(lower(NEW.valid_from_window), '-infinity'::timestamptz);
    candidate_end := CASE
        WHEN NEW.valid_to_window IS NULL THEN 'infinity'::timestamptz
        ELSE COALESCE(upper(NEW.valid_to_window), 'infinity'::timestamptz)
    END;

    SELECT COALESCE(SUM(membership_binary64_scaled_numerator(existing.membership_weight)), 0)
      INTO existing_numerator
      FROM membership_assignment AS existing
     WHERE existing.tenant_record_id = NEW.tenant_record_id
       AND existing.membership_type_code = NEW.membership_type_code
       AND existing.membership_assignment_id <> NEW.membership_assignment_id
       AND (
            (observed_kind = 'document'
             AND existing.document_record_id = observed_id
             AND existing.text_segment_id IS NULL)
         OR (observed_kind = 'text_segment'
             AND existing.text_segment_id = observed_id
             AND existing.document_record_id IS NULL)
       )
       -- Membership validity boundaries are uncertainty windows. Budget admission is deliberately
       -- conservative: if two assignments can overlap, their known shares must fit the same budget.
       AND COALESCE(lower(existing.valid_from_window), '-infinity'::timestamptz) <= candidate_end
       AND candidate_start <= CASE
            WHEN existing.valid_to_window IS NULL THEN 'infinity'::timestamptz
            ELSE COALESCE(upper(existing.valid_to_window), 'infinity'::timestamptz)
       END;

    IF existing_numerator + candidate_numerator > unity_numerator THEN
        RAISE EXCEPTION 'overlapping same-role membership shares exceed unity'
            USING ERRCODE = '23514', CONSTRAINT = 'membership_assignment_same_role_share_budget';
    END IF;

    RETURN NEW;
END;
$enforce_membership_same_role_share_budget$;

CREATE TRIGGER membership_assignment_same_role_share_budget
BEFORE INSERT OR UPDATE OF
    tenant_record_id,
    document_record_id,
    text_segment_id,
    membership_type_code,
    membership_weight,
    valid_from_window,
    valid_to_window
ON membership_assignment
FOR EACH ROW
EXECUTE FUNCTION enforce_membership_same_role_share_budget();
