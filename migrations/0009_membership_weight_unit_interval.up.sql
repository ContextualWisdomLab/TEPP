-- Align persisted membership shares with the Membership owner domain `(0, 1]`.
-- `0006` already rejects non-positive shares; this successor adds the missing
-- upper bound without rewriting released or reserved migration history.

ALTER TABLE membership_assignment
    ADD CONSTRAINT membership_assignment_weight_unit_interval
    CHECK (membership_weight > 0 AND membership_weight <= 1);
