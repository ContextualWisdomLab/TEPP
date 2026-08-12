//! Stable content-redacting error-message contracts.

use temporal_core::TemporalError;

#[test]
fn every_temporal_error_has_a_stable_content_redacting_message() {
    let cases = [
        (
            TemporalError::InvalidTimestamp,
            "invalid temporal timestamp",
        ),
        (
            TemporalError::InvalidTemporalPrecision,
            "invalid temporal precision",
        ),
        (
            TemporalError::InvalidIntervalOrder,
            "invalid temporal interval order",
        ),
        (TemporalError::EmptyInterval, "temporal interval is empty"),
        (
            TemporalError::InvalidIntervalCertainty,
            "invalid temporal interval certainty",
        ),
        (
            TemporalError::RelationRequiresProperBoundedInterval,
            "temporal relation requires proper bounded intervals",
        ),
        (
            TemporalError::InvalidWirePayload,
            "invalid temporal wire payload",
        ),
        (
            TemporalError::UnsupportedWireVersion,
            "unsupported temporal wire version",
        ),
        (
            TemporalError::ClockTypeMismatch,
            "temporal clock type mismatch",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
