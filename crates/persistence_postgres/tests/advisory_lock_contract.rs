//! PostgreSQL legal-hold/deletion advisory-lock signature contracts.

const RETENTION_UP: &str =
    include_str!("../../../migrations/0007_retention_deletion_legal_hold.up.sql");

#[test]
fn lifecycle_locks_use_supported_single_bigint_signatures() {
    assert_eq!(
        RETENTION_UP.matches("pg_advisory_xact_lock(").count(),
        3,
        "insert, release, and deletion paths must use the same lock family"
    );
    assert_eq!(
        RETENTION_UP.matches("|| ':' ||").count(),
        3,
        "tenant and document identities must be hashed into one bigint key"
    );
    assert!(
        !RETENTION_UP.contains("hashtextextended(NEW.tenant_record_id::text, 0),"),
        "PostgreSQL has no pg_advisory_xact_lock(bigint, bigint) overload"
    );
    assert!(
        !RETENTION_UP.contains("hashtextextended(current_hold.tenant_record_id::text, 0),"),
        "release must use the same supported one-bigint advisory lock"
    );
}
