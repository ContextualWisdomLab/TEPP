//! Public contracts for immutable RFC 9562 UUIDv7 evidence identifiers.

use evidence_core::{EvidenceError, EvidenceId};
use std::str::FromStr;

#[test]
fn generated_evidence_identifier_uses_uuid_v7() {
    let identifier = EvidenceId::new();

    assert_eq!(identifier.as_uuid().get_version_num(), 7);
}

#[test]
fn rfc_uuid_v7_vector_round_trips_canonically() {
    let expected = "017f22e2-79b0-7cc3-98c4-dc0c0c07398f";

    let identifier = EvidenceId::from_str(expected).expect("RFC UUIDv7 must parse");

    assert_eq!(identifier.to_string(), expected);
    assert_eq!(EvidenceId::from_uuid(identifier.as_uuid()), Ok(identifier));
}

#[test]
fn non_v7_and_malformed_identifiers_fail_closed() {
    let version_four = "550e8400-e29b-41d4-a716-446655440000";

    assert_eq!(
        EvidenceId::from_str(version_four),
        Err(EvidenceError::InvalidEvidenceId)
    );
    assert_eq!(
        EvidenceId::from_str("not-a-uuid"),
        Err(EvidenceError::InvalidEvidenceId)
    );
}
