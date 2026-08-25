//! Checkpoint artifacts versus the CPU `f64` estimator.

use crate::CheckpointAuthorityError;

/// Closed vocabulary of scientific-authority roles for a run artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactRole {
    /// The production CPU `f64` reference estimator.
    CpuF64Estimator,
    /// A serialized model checkpoint produced by a run.
    ModelCheckpoint,
}

impl ArtifactRole {
    /// Return the stable wire role name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::CpuF64Estimator => "cpu_f64_estimator",
            Self::ModelCheckpoint => "model_checkpoint",
        }
    }

    /// Parse a stable wire role name.
    ///
    /// # Errors
    ///
    /// Returns [`CheckpointAuthorityError::InvalidAuthorityPayload`] for
    /// unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, CheckpointAuthorityError> {
        match name {
            "cpu_f64_estimator" => Ok(Self::CpuF64Estimator),
            "model_checkpoint" => Ok(Self::ModelCheckpoint),
            _ => Err(CheckpointAuthorityError::InvalidAuthorityPayload),
        }
    }
}

/// Identity, digest, and run provenance required to accept a checkpoint
/// as an artifact (never as the estimator).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckpointOffer<'a> {
    /// Opaque artifact identity assigned by the owning boundary.
    pub artifact_identity: &'a str,
    /// Canonical lowercase hex `SHA-256` of the checkpoint bytes.
    pub content_digest: &'a str,
    /// Model-run identity that produced the checkpoint.
    pub model_run_identity: &'a str,
}

/// Refuse to treat a checkpoint as the CPU `f64` estimator.
///
/// # Errors
///
/// Returns [`CheckpointAuthorityError::CheckpointIsNotEstimator`] when
/// `role` is [`ArtifactRole::ModelCheckpoint`].
pub fn refuse_checkpoint_as_estimator(role: ArtifactRole) -> Result<(), CheckpointAuthorityError> {
    match role {
        ArtifactRole::ModelCheckpoint => Err(CheckpointAuthorityError::CheckpointIsNotEstimator),
        ArtifactRole::CpuF64Estimator => Ok(()),
    }
}

/// Accept a checkpoint only as a validated run artifact.
///
/// Identity, model-run provenance, and a canonical digest are required.
/// Success does not grant estimator authority.
///
/// # Errors
///
/// Returns a missing-field or digest error when the offer is untrusted.
pub fn accept_checkpoint_artifact(
    offer: &CheckpointOffer<'_>,
) -> Result<(), CheckpointAuthorityError> {
    if offer.artifact_identity.is_empty() {
        return Err(CheckpointAuthorityError::MissingIdentity);
    }
    if offer.model_run_identity.is_empty() {
        return Err(CheckpointAuthorityError::MissingProvenance);
    }
    validate_sha256_hex(offer.content_digest)
}

/// Fraction of recovered artifact roles that match known truth.
///
/// # Errors
///
/// Returns [`CheckpointAuthorityError::InvalidAuthorityPayload`] when either
/// slice is empty or the lengths differ.
pub fn authority_recovery_rate(
    truth: &[ArtifactRole],
    decided: &[ArtifactRole],
) -> Result<f64, CheckpointAuthorityError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(CheckpointAuthorityError::InvalidAuthorityPayload);
    }
    let mut matches = 0_u32;
    for (truth_role, decided_role) in truth.iter().zip(decided) {
        if truth_role == decided_role {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

fn validate_sha256_hex(digest: &str) -> Result<(), CheckpointAuthorityError> {
    if digest.is_empty() {
        return Err(CheckpointAuthorityError::MissingDigest);
    }
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(CheckpointAuthorityError::InvalidDigest);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactRole, CheckpointOffer, accept_checkpoint_artifact, authority_recovery_rate,
        refuse_checkpoint_as_estimator,
    };
    use crate::CheckpointAuthorityError;

    #[test]
    fn local_branches_cover_roles_payloads_and_wire_names() {
        assert_eq!(
            refuse_checkpoint_as_estimator(ArtifactRole::ModelCheckpoint),
            Err(CheckpointAuthorityError::CheckpointIsNotEstimator)
        );
        refuse_checkpoint_as_estimator(ArtifactRole::CpuF64Estimator).expect("estimator");
        for role in [ArtifactRole::CpuF64Estimator, ArtifactRole::ModelCheckpoint] {
            assert_eq!(
                ArtifactRole::from_wire_name(role.wire_name()).expect("round-trip"),
                role
            );
        }
        assert_eq!(
            ArtifactRole::from_wire_name("posterior_summary"),
            Err(CheckpointAuthorityError::InvalidAuthorityPayload)
        );
        let offer = CheckpointOffer {
            artifact_identity: "artifact-01",
            content_digest: &"cd".repeat(32),
            model_run_identity: "run-01",
        };
        accept_checkpoint_artifact(&offer).expect("artifact");
        assert_eq!(
            accept_checkpoint_artifact(&CheckpointOffer {
                artifact_identity: "",
                ..offer
            }),
            Err(CheckpointAuthorityError::MissingIdentity)
        );
        assert_eq!(
            accept_checkpoint_artifact(&CheckpointOffer {
                model_run_identity: "",
                ..offer
            }),
            Err(CheckpointAuthorityError::MissingProvenance)
        );
        assert_eq!(
            accept_checkpoint_artifact(&CheckpointOffer {
                content_digest: "",
                ..offer
            }),
            Err(CheckpointAuthorityError::MissingDigest)
        );
        assert_eq!(
            accept_checkpoint_artifact(&CheckpointOffer {
                content_digest: "ab",
                ..offer
            }),
            Err(CheckpointAuthorityError::InvalidDigest)
        );
        assert_eq!(
            accept_checkpoint_artifact(&CheckpointOffer {
                content_digest: &"gh".repeat(32),
                ..offer
            }),
            Err(CheckpointAuthorityError::InvalidDigest)
        );
        let matched = authority_recovery_rate(
            &[ArtifactRole::ModelCheckpoint],
            &[ArtifactRole::ModelCheckpoint],
        )
        .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            authority_recovery_rate(&[], &[]),
            Err(CheckpointAuthorityError::InvalidAuthorityPayload)
        );
        assert_eq!(
            authority_recovery_rate(&[ArtifactRole::ModelCheckpoint], &[]),
            Err(CheckpointAuthorityError::InvalidAuthorityPayload)
        );
    }
}
