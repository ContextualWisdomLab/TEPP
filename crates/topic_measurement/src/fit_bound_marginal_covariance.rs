//! Fit-local topic-basis binding for document marginal covariance.
//!
//! [`DocumentMarginalCovariance`] is the numerical selected-inverse result from a
//! joint precision and intentionally retains the provisional topic UUID vector
//! supplied to that precision. Scientific recovery also needs proof that each
//! covariance belongs to the same fitted topic-term basis as the owner-issued
//! location/content state. This module composes those owner outputs without
//! changing covariance arithmetic or promoting provisional UUIDs to semantic or
//! Evidence-authenticated topic identity.

use temporal_core::EventTime;
use uuid::Uuid;

use crate::{
    DocumentMarginalCovariance, FittedTopicBasisIdentity, ReferenceTopicFit,
    TopicMeasurementError,
};

/// One document marginal covariance bound to the exact fitted topic-term basis.
///
/// The wrapped covariance is still expressed in the fit's local ALR order. Its
/// [`FittedTopicBasisIdentity`] is derived from the same [`ReferenceTopicFit`]
/// that produces the joint precision, so callers cannot attach a detached basis
/// identity after covariance calculation. The retained `topic_ids` remain
/// provisional compatibility coordinates and are not source/vocabulary or
/// semantic topic authority.
#[derive(Clone, Debug, PartialEq)]
pub struct FitBoundDocumentMarginalCovariance {
    marginal: DocumentMarginalCovariance,
    topic_basis_identity: FittedTopicBasisIdentity,
}

impl FitBoundDocumentMarginalCovariance {
    /// Derive one document marginal covariance and bind its fitted topic basis.
    ///
    /// The scalar path delegates to [`Self::from_bound_fit_many`] so fit-local
    /// basis binding and selected-inverse arithmetic cannot drift from the batch
    /// path used by repeated scientific coverage.
    ///
    /// # Errors
    ///
    /// Propagates malformed fitted-basis, joint-precision, missing-document, and
    /// non-finite covariance failures from their numerical owners.
    pub fn from_bound_fit(
        fit: &ReferenceTopicFit,
        topic_ids: Vec<Uuid>,
        document_id: Uuid,
    ) -> Result<Self, TopicMeasurementError> {
        Self::from_bound_fit_many(fit, topic_ids, &[document_id])?
            .into_iter()
            .next()
            .ok_or(TopicMeasurementError::InvalidModelInput)
    }

    /// Derive multiple document marginals from one fit-owned joint precision.
    ///
    /// The fitted topic-basis identity and joint precision are each derived once
    /// from the same owner-issued [`ReferenceTopicFit`]. The numerical owner then
    /// factors that precision once for the requested document set. Requested
    /// document order is preserved; empty, duplicate, or absent identities fail
    /// closed in the selected-inverse owner.
    ///
    /// This remains fit-local numerical evidence. It does not authenticate
    /// source/vocabulary provenance, promote provisional topic UUIDs to semantic
    /// identity, or establish empirical interval calibration.
    ///
    /// # Errors
    ///
    /// Propagates fitted-basis, joint-precision, request-identity, factorization,
    /// solve, symmetry, and positive-definiteness failures from their owners.
    pub fn from_bound_fit_many(
        fit: &ReferenceTopicFit,
        topic_ids: Vec<Uuid>,
        document_ids: &[Uuid],
    ) -> Result<Vec<Self>, TopicMeasurementError> {
        let topic_basis_identity = FittedTopicBasisIdentity::from_bound_fit(fit)?;
        let precision = fit.build_joint_coordinate_precision(topic_ids)?;
        precision
            .document_marginal_covariances(document_ids)?
            .into_iter()
            .map(|marginal| {
                Ok(Self {
                    marginal,
                    topic_basis_identity: topic_basis_identity.clone(),
                })
            })
            .collect()
    }

    /// Return the fitted document whose marginal covariance was solved.
    #[must_use]
    pub const fn document_id(&self) -> Uuid {
        self.marginal.document_id()
    }

    /// Return the fitted document's retained event-time coordinate.
    #[must_use]
    pub const fn event_time(&self) -> EventTime {
        self.marginal.event_time()
    }

    /// Return provisional topic UUIDs in ALR numerator/reference order.
    ///
    /// These UUIDs preserve the existing joint-precision compatibility contract;
    /// use [`Self::topic_basis_identity`] for fit-local numerical topic identity.
    #[must_use]
    pub fn topic_ids(&self) -> &[Uuid] {
        self.marginal.topic_ids()
    }

    /// Return the exact fitted topic-term basis bound to this covariance.
    #[must_use]
    pub const fn topic_basis_identity(&self) -> &FittedTopicBasisIdentity {
        &self.topic_basis_identity
    }

    /// Return the `(K-1) × (K-1)` document marginal covariance matrix.
    #[must_use]
    pub fn values(&self) -> &[Vec<f64>] {
        self.marginal.values()
    }
}
