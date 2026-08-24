//! A model checkpoint is not the CPU `f64` estimator.

use checkpoint_authority::{
    ArtifactRole, CheckpointAuthorityError, CheckpointOffer, accept_checkpoint_artifact,
    authority_recovery_rate, refuse_checkpoint_as_estimator,
};

#[test]
fn a_checkpoint_cannot_become_the_cpu_f64_estimator() {
    assert_eq!(
        refuse_checkpoint_as_estimator(ArtifactRole::ModelCheckpoint),
        Err(CheckpointAuthorityError::CheckpointIsNotEstimator)
    );
    refuse_checkpoint_as_estimator(ArtifactRole::CpuF64Estimator).expect("estimator");
}

#[test]
fn an_unvalidated_checkpoint_fails_closed() {
    let valid = CheckpointOffer {
        artifact_identity: "artifact-01",
        content_digest: &"ab".repeat(32),
        model_run_identity: "run-01",
    };
    accept_checkpoint_artifact(&valid).expect("validated artifact");

    let missing_identity = CheckpointOffer {
        artifact_identity: "",
        ..valid
    };
    assert_eq!(
        accept_checkpoint_artifact(&missing_identity),
        Err(CheckpointAuthorityError::MissingIdentity)
    );

    let missing_provenance = CheckpointOffer {
        model_run_identity: "",
        ..valid
    };
    assert_eq!(
        accept_checkpoint_artifact(&missing_provenance),
        Err(CheckpointAuthorityError::MissingProvenance)
    );

    let missing_digest = CheckpointOffer {
        content_digest: "",
        ..valid
    };
    assert_eq!(
        accept_checkpoint_artifact(&missing_digest),
        Err(CheckpointAuthorityError::MissingDigest)
    );

    let short_digest = CheckpointOffer {
        content_digest: "abcd",
        ..valid
    };
    assert_eq!(
        accept_checkpoint_artifact(&short_digest),
        Err(CheckpointAuthorityError::InvalidDigest)
    );

    let uppercase_digest = CheckpointOffer {
        content_digest: &"AB".repeat(32),
        ..valid
    };
    assert_eq!(
        accept_checkpoint_artifact(&uppercase_digest),
        Err(CheckpointAuthorityError::InvalidDigest)
    );

    let non_hex_digest = CheckpointOffer {
        content_digest: &"zz".repeat(32),
        ..valid
    };
    assert_eq!(
        accept_checkpoint_artifact(&non_hex_digest),
        Err(CheckpointAuthorityError::InvalidDigest)
    );
}

#[test]
fn recovered_roles_match_known_truth_better_than_an_estimator_collapse() {
    let truth = [
        ArtifactRole::ModelCheckpoint,
        ArtifactRole::CpuF64Estimator,
        ArtifactRole::ModelCheckpoint,
    ];
    let recovered = truth;
    let collapsed = [
        ArtifactRole::CpuF64Estimator,
        ArtifactRole::CpuF64Estimator,
        ArtifactRole::CpuF64Estimator,
    ];
    let recovered_rate = authority_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = authority_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_role, decided_role) in truth.iter().zip(recovered.iter()) {
            if truth_role == decided_role {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_role_payloads_fail_closed() {
    assert_eq!(
        authority_recovery_rate(&[], &[]),
        Err(CheckpointAuthorityError::InvalidAuthorityPayload)
    );
    assert_eq!(
        authority_recovery_rate(&[ArtifactRole::ModelCheckpoint], &[]),
        Err(CheckpointAuthorityError::InvalidAuthorityPayload)
    );
    assert_eq!(
        authority_recovery_rate(
            &[ArtifactRole::ModelCheckpoint, ArtifactRole::CpuF64Estimator],
            &[ArtifactRole::ModelCheckpoint]
        ),
        Err(CheckpointAuthorityError::InvalidAuthorityPayload)
    );
}
