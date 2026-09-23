//! Fit-local topic-basis binding for one document marginal covariance.
//!
//! [`DocumentMarginalCovariance`] is the numerical selected-inverse result from a
//! joint precision and intentionally retains the provisional topic UUID vector
//! supplied to that precision. Scientific recovery also needs proof that the
//! covariance belongs to the same fitted topic-term basis as the owner-issued
//! location/content state. This module composes those two owner outputs without
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
    /// The numerical covariance delegates unchanged to the #704 full-joint
    /// selected-inverse path. The basis identity is derived independently from
    /// the same owner-issued fit; callers supply only provisional topic UUIDs and
    /// the requested document identity.
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
        let topic_basis_identity = FittedTopicBasisIdentity::from_bound_fit(fit)?;
        let precision = fit.build_joint_coordinate_precision(topic_ids)?;
        let marginal = precision.document_marginal_covariance(document_id)?;
        if marginal.topic_ids().len() != topic_basis_identity.topics().len() {
            return Err(TopicMeasurementError::InvalidModelInput);
        }
        Ok(Self {
            marginal,
            topic_basis_identity,
        })
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
