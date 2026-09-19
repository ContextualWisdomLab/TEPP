DROP TRIGGER IF EXISTS membership_assignment_same_role_share_budget ON membership_assignment;
DROP FUNCTION IF EXISTS enforce_membership_same_role_share_budget();
DROP FUNCTION IF EXISTS membership_binary64_scaled_numerator(numeric);
DROP TABLE IF EXISTS membership_share_budget_guard;
