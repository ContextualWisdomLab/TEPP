//! Static cutover contracts for the Membership share-budget migration.

const MEMBERSHIP_SHARE_BUDGET_UP: &str =
    include_str!("../../../migrations/0010_membership_same_role_share_budget.up.sql");

#[test]
fn retry_replaces_the_trigger_without_a_write_admission_gap() {
    assert!(
        MEMBERSHIP_SHARE_BUDGET_UP.contains(
            "CREATE OR REPLACE TRIGGER membership_assignment_same_role_share_budget"
        ),
        "retry must atomically replace the active admission trigger"
    );
    assert!(
        !MEMBERSHIP_SHARE_BUDGET_UP.contains(
            "DROP TRIGGER IF EXISTS membership_assignment_same_role_share_budget ON membership_assignment;\nCREATE TRIGGER"
        ),
        "retry must not commit a drop/create trigger gap before historical validation"
    );
}
