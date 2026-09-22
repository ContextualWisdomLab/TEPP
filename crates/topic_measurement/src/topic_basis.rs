//! Deterministic identities for one fitted numerical topic basis.
//!
//! These identities bind local topic indexes to the exact fitted topic-term
//! probability rows produced by the numerical owner. Public construction accepts
//! only an owner-issued [`ReferenceTopicFit`], so callers cannot promote a
//! detached or hand-mutated model plus a caller-authored vocabulary width as
//! fit-owned coordinate authority. The identities remain deliberately narrower
//! than source/vocabulary provenance: a basis identity does not prove which
//! Evidence-owned snapshot or lexical vocabulary supplied the columns.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::{
    ReferenceTopicFit, error::TopicMeasurementError, reference::ReferenceTopicModel,
};

/// Version of the fit-local topic-basis identity contract.
pub const FITTED_TOPIC_BASIS_IDENTITY_VERSION: &str = "tepp.fitted_topic_basis_identity.v1";

const TOPIC_COORDINATE_DOMAIN: &[u8] = b"tepp:fitted_topic_coordinate:v1\0";
const TOPIC_BASIS_DOMAIN: &[u8] = b"tepp:fitted_topic_basis:v1\0";

/// Content-bound identity for one topic row in a fitted local topic order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FittedTopicCoordinateIdentity {
    topic_index: usize,
    sha256: String,
}

impl FittedTopicCoordinateIdentity {
    /// Return the topic's local index in the fitted basis.
    #[must_use]
    pub const fn topic_index(&self) -> usize {
        self.topic_index
    }

    /// Return the lowercase SHA-256 identity of the exact fitted topic row.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Ordered identity of the exact fitted topic-term basis.
///
/// Per-topic identities are content-bound and therefore follow the numerical
/// topic row if rows are permuted. The enclosing basis digest is intentionally
/// order-sensitive because ALR numerator/reference coordinates and lineage
/// indexes are defined in that fitted local order.
///
/// This value is fit-local numerical evidence only. It does not authenticate a
/// source snapshot or vocabulary and must not be promoted as semantic topic
/// identity without the Evidence-owned provenance boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FittedTopicBasisIdentity {
    vocabulary_size: usize,
    sha256: String,
    topics: Vec<FittedTopicCoordinateIdentity>,
}

impl FittedTopicBasisIdentity {
    /// Derive an identity from one owner-issued converged reference fit.
    ///
    /// The vocabulary width comes from the admitted input retained inside the
    /// same nominal fit aggregate as the numerical model. Callers therefore do
    /// not supply either side of the model/vocabulary pairing independently.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] when the retained
    /// fitted basis is malformed, non-finite/non-positive, or ambiguous.
    pub fn from_bound_fit(fit: &ReferenceTopicFit) -> Result<Self, TopicMeasurementError> {
        Self::from_model(fit.model(), fit.input().vocabulary_size())
    }

    /// Validate and derive the exact topic-row identity inside the owner crate.
    ///
    /// This detached pair form is intentionally crate-private. It exists for
    /// owner-internal validation and hostile-state tests, not as a consumer
    /// authority constructor.
    pub(crate) fn from_model(
        model: &ReferenceTopicModel,
        vocabulary_size: usize,
    ) -> Result<Self, TopicMeasurementError> {
        let topic_count = model.topic_term_probabilities.len();
        if topic_count < 2 || vocabulary_size < 2 {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        let topic_count_u64 =
            u64::try_from(topic_count).map_err(|_| TopicMeasurementError::InvalidModelInput)?;
        let vocabulary_size_u64 =
            u64::try_from(vocabulary_size).map_err(|_| TopicMeasurementError::InvalidModelInput)?;

        let mut seen = BTreeSet::new();
        let mut topics = Vec::with_capacity(topic_count);
        for (topic_index, row) in model.topic_term_probabilities.iter().enumerate() {
            if row.len() != vocabulary_size
                || row
                    .iter()
                    .any(|probability| !probability.is_finite() || *probability <= 0.0)
            {
                return Err(TopicMeasurementError::InvalidModelInput);
            }

            let mut hasher = Sha256::new();
            hasher.update(TOPIC_COORDINATE_DOMAIN);
            hasher.update(FITTED_TOPIC_BASIS_IDENTITY_VERSION.as_bytes());
            hasher.update(topic_count_u64.to_be_bytes());
            hasher.update(vocabulary_size_u64.to_be_bytes());
            for probability in row {
                hasher.update(probability.to_bits().to_be_bytes());
            }
            let digest = hasher.finalize();
            let sha256 = lowercase_hex(&digest);
            if !seen.insert(sha256.clone()) {
                return Err(TopicMeasurementError::InvalidModelInput);
            }
            topics.push(FittedTopicCoordinateIdentity {
                topic_index,
                sha256,
            });
        }

        let mut basis_hasher = Sha256::new();
        basis_hasher.update(TOPIC_BASIS_DOMAIN);
        basis_hasher.update(FITTED_TOPIC_BASIS_IDENTITY_VERSION.as_bytes());
        basis_hasher.update(topic_count_u64.to_be_bytes());
        basis_hasher.update(vocabulary_size_u64.to_be_bytes());
        for topic in &topics {
            let topic_index = u64::try_from(topic.topic_index)
                .map_err(|_| TopicMeasurementError::InvalidModelInput)?;
            basis_hasher.update(topic_index.to_be_bytes());
            basis_hasher.update(topic.sha256.as_bytes());
        }
        let basis_digest = basis_hasher.finalize();

        Ok(Self {
            vocabulary_size,
            sha256: lowercase_hex(&basis_digest),
            topics,
        })
    }

    /// Return this identity contract's exact version.
    #[must_use]
    pub const fn version(&self) -> &'static str {
        FITTED_TOPIC_BASIS_IDENTITY_VERSION
    }

    /// Return the fitted vocabulary-coordinate width used by every topic row.
    #[must_use]
    pub const fn vocabulary_size(&self) -> usize {
        self.vocabulary_size
    }

    /// Return the order-sensitive lowercase SHA-256 identity of the fitted basis.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// Return per-topic content identities in fitted local topic order.
    #[must_use]
    pub fn topics(&self) -> &[FittedTopicCoordinateIdentity] {
        &self.topics
    }
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let byte = *byte;
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::FittedTopicBasisIdentity;
    use crate::{ReferenceTopicModel, TopicMeasurementError};

    fn model() -> ReferenceTopicModel {
        ReferenceTopicModel {
            seed: 7,
            iterations: 4,
            objective: -1.0,
            topic_term_probabilities: vec![vec![0.7, 0.2, 0.1], vec![0.1, 0.3, 0.6]],
            document_topic_proportions: Vec::new(),
            document_coordinate_variances: Vec::new(),
            prevalence_coefficients: Vec::new(),
            prevalence_features: Vec::new(),
            sequence_edges: Vec::new(),
            connected_post_count: 0,
            lineage_count: 0,
        }
    }

    #[test]
    fn detached_owner_validation_preserves_content_identity_under_permutation() {
        let model = model();
        let identity = FittedTopicBasisIdentity::from_model(&model, 3).expect("basis identity");
        let repeated = FittedTopicBasisIdentity::from_model(&model, 3).expect("repeated identity");
        assert_eq!(identity, repeated);

        let mut permuted = model;
        permuted.topic_term_probabilities.swap(0, 1);
        let permuted_identity =
            FittedTopicBasisIdentity::from_model(&permuted, 3).expect("permuted identity");

        assert_eq!(identity.topics()[0].sha256(), permuted_identity.topics()[1].sha256());
        assert_eq!(identity.topics()[1].sha256(), permuted_identity.topics()[0].sha256());
        assert_ne!(identity.sha256(), permuted_identity.sha256());
    }

    #[test]
    fn detached_owner_validation_refuses_malformed_or_ambiguous_rows() {
        let model = model();
        assert_eq!(
            FittedTopicBasisIdentity::from_model(&model, 2),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut malformed_width = model.clone();
        malformed_width.topic_term_probabilities[0].pop();
        assert_eq!(
            FittedTopicBasisIdentity::from_model(&malformed_width, 3),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut non_finite = model.clone();
        non_finite.topic_term_probabilities[0][0] = f64::NAN;
        assert_eq!(
            FittedTopicBasisIdentity::from_model(&non_finite, 3),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut non_positive = model.clone();
        non_positive.topic_term_probabilities[0][0] = 0.0;
        assert_eq!(
            FittedTopicBasisIdentity::from_model(&non_positive, 3),
            Err(TopicMeasurementError::InvalidModelInput)
        );

        let mut duplicate = model;
        duplicate.topic_term_probabilities[1] = duplicate.topic_term_probabilities[0].clone();
        assert_eq!(
            FittedTopicBasisIdentity::from_model(&duplicate, 3),
            Err(TopicMeasurementError::InvalidModelInput)
        );
    }
}
