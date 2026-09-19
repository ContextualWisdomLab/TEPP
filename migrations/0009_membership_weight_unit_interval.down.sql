-- Restore the predecessor persistence contract from `0006`: positive shares
-- remain enforced there, while the successor upper bound is removed.

ALTER TABLE membership_assignment
    DROP CONSTRAINT membership_assignment_weight_unit_interval;
