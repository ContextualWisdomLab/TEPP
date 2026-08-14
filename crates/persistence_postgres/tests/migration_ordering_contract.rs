//! Execution-order contracts for lifecycle migration routines and rollback.
//!
//! PostgreSQL resolves routines and trigger dependencies statement by statement.
//! These tests prevent privilege statements from preceding routine definitions
//! and prevent rollback from dropping routines while dependent triggers exist.

const RETENTION_UP_SQL: &str =
    include_str!("../../../migrations/0007_retention_deletion_legal_hold.up.sql");
const RETENTION_DOWN_SQL: &str =
    include_str!("../../../migrations/0007_retention_deletion_legal_hold.down.sql");

fn statement_position(sql: &str, needle: &str) -> usize {
    sql.find(needle)
        .unwrap_or_else(|| panic!("missing migration statement: {needle}"))
}

#[test]
fn lifecycle_routine_privileges_follow_routine_definitions() {
    let ordered_pairs = [
        (
            "CREATE OR REPLACE FUNCTION enforce_retention_policy_succession",
            "REVOKE ALL ON FUNCTION enforce_retention_policy_succession",
        ),
        (
            "CREATE OR REPLACE FUNCTION supersede_retention_policy",
            "REVOKE ALL ON FUNCTION supersede_retention_policy",
        ),
        (
            "CREATE OR REPLACE FUNCTION supersede_retention_policy",
            "GRANT EXECUTE ON FUNCTION supersede_retention_policy",
        ),
        (
            "CREATE OR REPLACE FUNCTION reject_held_evidence_deletion",
            "REVOKE ALL ON FUNCTION reject_held_evidence_deletion",
        ),
        (
            "CREATE OR REPLACE FUNCTION reject_tombstoned_evidence_restore",
            "REVOKE ALL ON FUNCTION reject_tombstoned_evidence_restore",
        ),
        (
            "CREATE OR REPLACE FUNCTION guard_legal_hold_insert",
            "REVOKE ALL ON FUNCTION guard_legal_hold_insert",
        ),
        (
            "CREATE OR REPLACE FUNCTION enforce_legal_hold_release",
            "REVOKE ALL ON FUNCTION enforce_legal_hold_release",
        ),
        (
            "CREATE OR REPLACE FUNCTION release_legal_hold",
            "REVOKE ALL ON FUNCTION release_legal_hold",
        ),
        (
            "CREATE OR REPLACE FUNCTION release_legal_hold",
            "GRANT EXECUTE ON FUNCTION release_legal_hold",
        ),
    ];

    for (definition, privilege) in ordered_pairs {
        assert!(
            statement_position(RETENTION_UP_SQL, definition)
                < statement_position(RETENTION_UP_SQL, privilege),
            "routine privileges must follow their definition: {privilege}"
        );
    }
}

#[test]
fn rollback_drops_lifecycle_routines_after_trigger_cleanup() {
    let trigger_cleanup_end = statement_position(RETENTION_DOWN_SQL, "$tepp$;");
    let routine_drops = [
        "DROP FUNCTION IF EXISTS release_legal_hold",
        "DROP FUNCTION IF EXISTS supersede_retention_policy",
        "DROP FUNCTION IF EXISTS enforce_retention_policy_succession",
        "DROP FUNCTION IF EXISTS enforce_legal_hold_release",
        "DROP FUNCTION IF EXISTS reject_tombstoned_evidence_restore",
        "DROP FUNCTION IF EXISTS reject_held_evidence_deletion",
        "DROP FUNCTION IF EXISTS guard_legal_hold_insert",
    ];

    for routine_drop in routine_drops {
        assert!(
            trigger_cleanup_end < statement_position(RETENTION_DOWN_SQL, routine_drop),
            "rollback must remove dependent triggers before dropping routines: {routine_drop}"
        );
    }
}
