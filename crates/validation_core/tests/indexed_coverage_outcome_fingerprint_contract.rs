use validation_core::{
    CoverageCalibrationReplicationOutcome, canonical_indexed_coverage_outcomes_sha256,
};

fn outcomes() -> Vec<CoverageCalibrationReplicationOutcome> {
    vec![
        CoverageCalibrationReplicationOutcome::successful(0, vec![0.95, 0.91]),
        CoverageCalibrationReplicationOutcome::numerical_failure(1),
        CoverageCalibrationReplicationOutcome::successful(2, vec![0.97]),
    ]
}

#[test]
fn indexed_coverage_fingerprint_is_canonical_and_bit_exact() {
    let ordered = outcomes();
    let mut reversed = ordered.clone();
    reversed.reverse();

    let ordered_digest = canonical_indexed_coverage_outcomes_sha256(3, &ordered)
        .expect("canonical indexed outcome digest");
    let reversed_digest = canonical_indexed_coverage_outcomes_sha256(3, &reversed)
        .expect("completion order must not affect digest");

    assert_eq!(ordered_digest, reversed_digest);
    assert_eq!(ordered_digest.len(), 64);
    assert!(
        ordered_digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );

    let different_failure_identity = vec![
        CoverageCalibrationReplicationOutcome::numerical_failure(0),
        CoverageCalibrationReplicationOutcome::successful(1, vec![0.95, 0.91]),
        CoverageCalibrationReplicationOutcome::successful(2, vec![0.97]),
    ];
    assert_ne!(
        ordered_digest,
        canonical_indexed_coverage_outcomes_sha256(3, &different_failure_identity)
            .expect("different failure identity digest")
    );

    let positive_zero = vec![
        CoverageCalibrationReplicationOutcome::successful(0, vec![0.0]),
        CoverageCalibrationReplicationOutcome::numerical_failure(1),
        CoverageCalibrationReplicationOutcome::successful(2, vec![0.97]),
    ];
    let negative_zero = vec![
        CoverageCalibrationReplicationOutcome::successful(0, vec![-0.0]),
        CoverageCalibrationReplicationOutcome::numerical_failure(1),
        CoverageCalibrationReplicationOutcome::successful(2, vec![0.97]),
    ];
    assert_ne!(
        canonical_indexed_coverage_outcomes_sha256(3, &positive_zero)
            .expect("positive-zero digest"),
        canonical_indexed_coverage_outcomes_sha256(3, &negative_zero)
            .expect("negative-zero digest"),
        "fingerprint must bind exact binary64 coverage bits"
    );
}

#[test]
fn indexed_coverage_fingerprint_rejects_non_exact_identity_sets() {
    let duplicate = vec![
        CoverageCalibrationReplicationOutcome::successful(0, vec![0.95]),
        CoverageCalibrationReplicationOutcome::numerical_failure(0),
    ];
    assert!(canonical_indexed_coverage_outcomes_sha256(2, &duplicate).is_err());
}
