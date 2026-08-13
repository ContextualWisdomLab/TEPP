//! Classify concurrent-write SQLSTATE codes without a live server.

use crate::PersistenceError;

/// `PostgreSQL` `unique_violation` SQLSTATE.
pub const UNIQUE_VIOLATION_SQLSTATE: &str = "23505";

/// `PostgreSQL` `serialization_failure` SQLSTATE.
pub const SERIALIZATION_FAILURE_SQLSTATE: &str = "40001";

/// `PostgreSQL` `deadlock_detected` SQLSTATE.
pub const DEADLOCK_DETECTED_SQLSTATE: &str = "40P01";

/// `PostgreSQL` `exclusion_violation` SQLSTATE.
pub const EXCLUSION_VIOLATION_SQLSTATE: &str = "23P01";

/// Map a `PostgreSQL` SQLSTATE from a racing write onto a domain error.
///
/// Unique identity collisions stay [`PersistenceError::DuplicateDocumentRecord`].
/// Serialization, deadlock, and exclusion failures become
/// [`PersistenceError::ConcurrentWriteConflict`]. Other codes stay unmapped so
/// the transport can fail closed as a generic execution error.
#[must_use]
pub fn classify_write_conflict(sqlstate: &str) -> Option<PersistenceError> {
    match sqlstate {
        UNIQUE_VIOLATION_SQLSTATE => Some(PersistenceError::DuplicateDocumentRecord),
        SERIALIZATION_FAILURE_SQLSTATE
        | DEADLOCK_DETECTED_SQLSTATE
        | EXCLUSION_VIOLATION_SQLSTATE => Some(PersistenceError::ConcurrentWriteConflict),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEADLOCK_DETECTED_SQLSTATE, EXCLUSION_VIOLATION_SQLSTATE, SERIALIZATION_FAILURE_SQLSTATE,
        UNIQUE_VIOLATION_SQLSTATE, classify_write_conflict,
    };
    use crate::PersistenceError;

    #[test]
    fn known_sqlstates_map_and_unknown_codes_stay_unmapped() {
        assert_eq!(
            classify_write_conflict(UNIQUE_VIOLATION_SQLSTATE),
            Some(PersistenceError::DuplicateDocumentRecord)
        );
        assert_eq!(
            classify_write_conflict(SERIALIZATION_FAILURE_SQLSTATE),
            Some(PersistenceError::ConcurrentWriteConflict)
        );
        assert_eq!(
            classify_write_conflict(DEADLOCK_DETECTED_SQLSTATE),
            Some(PersistenceError::ConcurrentWriteConflict)
        );
        assert_eq!(
            classify_write_conflict(EXCLUSION_VIOLATION_SQLSTATE),
            Some(PersistenceError::ConcurrentWriteConflict)
        );
        assert_eq!(classify_write_conflict("00000"), None);
        assert_eq!(classify_write_conflict(""), None);
        assert_eq!(classify_write_conflict("P0001"), None);
    }
}
