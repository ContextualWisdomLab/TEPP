-- Align persisted membership shares with the Membership owner domain `(0, 1]`.
-- `0006` already rejects non-positive shares; this successor adds the missing
-- upper bound without rewriting released or reserved migration history.
--
-- Add the constraint NOT VALID first so the ACCESS EXCLUSIVE DDL lock is held
-- only for catalog installation rather than a full table scan. PostgreSQL still
-- enforces a NOT VALID CHECK on new writes. The subsequent VALIDATE scan uses a
-- weaker lock while proving all pre-existing rows satisfy the owner contract.

ALTER TABLE membership_assignment
    ADD CONSTRAINT membership_assignment_weight_unit_interval
    CHECK (membership_weight > 0 AND membership_weight <= 1) NOT VALID;

ALTER TABLE membership_assignment
    VALIDATE CONSTRAINT membership_assignment_weight_unit_interval;
