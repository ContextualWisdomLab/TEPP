DROP TRIGGER IF EXISTS membership_assignment_same_role_share_budget ON membership_assignment;
DROP FUNCTION IF EXISTS enforce_membership_same_role_share_budget();
DROP TABLE IF EXISTS membership_share_budget_guard;
DROP FUNCTION IF EXISTS membership_duplicate_temporal_edge_exists(uuid, text, uuid, uuid, uuid, text, tstzrange, uuid);
DROP FUNCTION IF EXISTS membership_same_role_max_existing_numerator(uuid, text, uuid, text, tstzrange, uuid);
DROP FUNCTION IF EXISTS membership_possible_activity_envelope(tstzrange, tstzrange);
DROP FUNCTION IF EXISTS membership_binary64_scaled_numerator(numeric);
