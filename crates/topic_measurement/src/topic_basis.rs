//! Deterministic identities for one fitted numerical topic basis.
//!
//! These identities bind local topic indexes to the exact fitted topic-term
//! probability rows produced by the numerical owner. They are deliberately
//! narrower than source/vocabulary provenance: a basis identity does not prove
//! which Evidence-owned snapshot or lexical vocabulary supplied the columns.
//! Release consumers must bind that owner provenance separately before treating
//! these coordinates as durable external authority.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::{ReferenceTopicModel, TopicMeasurementError};

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
    /// Derive a deterministic identity from a converged model's exact topic rows.
    ///
    /// Topic probability values are hashed as their exact IEEE-754 binary64 bit
    /// patterns after finite-positive validation. No topic is renamed or
    /// reordered. Duplicate topic rows fail closed because a content-derived
    /// coordinate could not distinguish their local labels.
    ///
    /// # Errors
    ///
    /// Returns [`TopicMeasurementError::InvalidModelInput`] when dimensions are
    /// invalid, a probability is non-finite/non-positive, or two fitted topic
    /// rows have the same content identity.
    pub fn from_model(
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
            let sha256 = lowercase_hex(hasher.finalize().as_slice());
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

        Ok(Self {
            vocabulary_size,
            sha256: lowercase_hex(basis_hasher.finalize().as_slice()),
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
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
