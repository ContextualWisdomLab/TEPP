//! Public contracts for immutable RFC 9562 `UUIDv7` evidence identifiers.

use evidence_core::{EvidenceError, EvidenceId};
use std::error::Error;
use std::str::FromStr;

#[test]
fn generated_and_default_identifiers_use_uuid_v7() {
    let generated = EvidenceId::new();
    let defaulted = EvidenceId::default();

    assert_eq!(generated.as_uuid().get_version_num(), 7);
    assert_eq!(defaulted.as_uuid().get_version_num(), 7);
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

#[test]
fn validation_error_is_stable_and_has_no_hidden_source() {
    let error = EvidenceError::InvalidEvidenceId;
    let standard_error: &dyn Error = &error;

    assert_eq!(error.to_string(), "invalid evidence identifier");
    assert!(standard_error.source().is_none());
}
