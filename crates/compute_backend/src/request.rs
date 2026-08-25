//! Workload request and precision policy.

use crate::error::ComputeBackendError;

/// Arithmetic mode for transient kernels versus final diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrecisionMode {
    /// CPU `f64` reference precision required for diagnostics.
    ReferenceF64,
    /// Approved mixed precision for transient device computation only.
    TransientMixed,
}

/// Whether a full document-by-topic tensor may reside on device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorpusPlacement {
    /// Stream micro-batches only.
    StreamedMicroBatches,
    /// Pin the full corpus responsibility tensor on the device.
    FullCorpusOnDevice,
}

/// Whether observations may be dropped under memory pressure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationRetention {
    /// Keep every observation.
    KeepAll,
    /// Drop observations so a batch fits.
    DropToFit,
}

/// Whether topic or model complexity may shrink to fit memory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelComplexity {
    /// Keep the requested topic/model complexity.
    KeepSpecified,
    /// Reduce complexity so a batch fits.
    ReduceToFit,
}

/// Whether a knowledge cutoff may move to fit memory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CutoffPolicy {
    /// Keep the requested cutoff.
    KeepCutoff,
    /// Move the cutoff so a batch fits.
    MoveToFit,
}

/// Memory-adaptation policies that must remain explicit at workload creation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdaptationPolicy {
    /// Whether the corpus may be fully resident on the device.
    pub corpus_placement: CorpusPlacement,
    /// Whether observations may be dropped to fit the device budget.
    pub observation_retention: ObservationRetention,
    /// Whether model complexity may be reduced to fit the device budget.
    pub model_complexity: ModelComplexity,
    /// Whether the knowledge cutoff may move to fit the device budget.
    pub cutoff_policy: CutoffPolicy,
}

/// A streamed workload that must never pin a full document-by-topic tensor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkloadRequest {
    document_count: u64,
    topic_count: u64,
    bytes_per_observation: u64,
    working_set_bytes: u64,
    requested_batch: u32,
    corpus_placement: CorpusPlacement,
    observation_retention: ObservationRetention,
    model_complexity: ModelComplexity,
    cutoff_policy: CutoffPolicy,
    final_quantity_precision: PrecisionMode,
}

impl WorkloadRequest {
    /// Construct a fail-closed workload request.
    ///
    /// Streamed document and topic cardinalities are stored independently; the
    /// constructor deliberately does not materialize or size a hypothetical
    /// full-corpus tensor that the controller refuses to allocate.
    ///
    /// # Errors
    ///
    /// Returns [`ComputeBackendError::InvalidBudget`] when counts, batch size,
    /// or per-observation bytes are zero.
    pub const fn new(
        document_count: u64,
        topic_count: u64,
        bytes_per_observation: u64,
        working_set_bytes: u64,
        requested_batch: u32,
        policy: AdaptationPolicy,
        final_quantity_precision: PrecisionMode,
    ) -> Result<Self, ComputeBackendError> {
        if document_count == 0
            || topic_count == 0
            || bytes_per_observation == 0
            || requested_batch == 0
        {
            return Err(ComputeBackendError::InvalidBudget);
        }
        Ok(Self {
            document_count,
            topic_count,
            bytes_per_observation,
            working_set_bytes,
            requested_batch,
            corpus_placement: policy.corpus_placement,
            observation_retention: policy.observation_retention,
            model_complexity: policy.model_complexity,
            cutoff_policy: policy.cutoff_policy,
            final_quantity_precision,
        })
    }

    /// Return the document count.
    #[must_use]
    pub const fn document_count(self) -> u64 {
        self.document_count
    }

    /// Return the topic count.
    #[must_use]
    pub const fn topic_count(self) -> u64 {
        self.topic_count
    }

    /// Return bytes charged per streamed observation.
    #[must_use]
    pub const fn bytes_per_observation(self) -> u64 {
        self.bytes_per_observation
    }

    /// Return the fixed working-set charge.
    #[must_use]
    pub const fn working_set_bytes(self) -> u64 {
        self.working_set_bytes
    }

    /// Return the caller-requested micro-batch.
    #[must_use]
    pub const fn requested_batch(self) -> u32 {
        self.requested_batch
    }

    /// Return the corpus placement policy.
    #[must_use]
    pub const fn corpus_placement(self) -> CorpusPlacement {
        self.corpus_placement
    }

    /// Return the observation-retention policy.
    #[must_use]
    pub const fn observation_retention(self) -> ObservationRetention {
        self.observation_retention
    }

    /// Return the model-complexity policy.
    #[must_use]
    pub const fn model_complexity(self) -> ModelComplexity {
        self.model_complexity
    }

    /// Return the cutoff policy.
    #[must_use]
    pub const fn cutoff_policy(self) -> CutoffPolicy {
        self.cutoff_policy
    }

    /// Return the precision required for final diagnostics.
    #[must_use]
    pub const fn final_quantity_precision(self) -> PrecisionMode {
        self.final_quantity_precision
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AdaptationPolicy, CorpusPlacement, CutoffPolicy, ModelComplexity, ObservationRetention,
        PrecisionMode, WorkloadRequest,
    };
    use crate::error::ComputeBackendError;

    fn request(
        documents: u64,
        topics: u64,
        bytes_per_observation: u64,
        batch: u32,
    ) -> Result<WorkloadRequest, ComputeBackendError> {
        WorkloadRequest::new(
            documents,
            topics,
            bytes_per_observation,
            0,
            batch,
            AdaptationPolicy {
                corpus_placement: CorpusPlacement::StreamedMicroBatches,
                observation_retention: ObservationRetention::KeepAll,
                model_complexity: ModelComplexity::KeepSpecified,
                cutoff_policy: CutoffPolicy::KeepCutoff,
            },
            PrecisionMode::ReferenceF64,
        )
    }

    #[test]
    fn request_rejects_zero_counts() {
        assert_eq!(request(0, 1, 8, 1), Err(ComputeBackendError::InvalidBudget));
        assert_eq!(request(1, 0, 8, 1), Err(ComputeBackendError::InvalidBudget));
        assert_eq!(request(1, 1, 0, 1), Err(ComputeBackendError::InvalidBudget));
        assert_eq!(request(1, 1, 8, 0), Err(ComputeBackendError::InvalidBudget));
    }

    #[test]
    fn streamed_dimensions_are_not_multiplied_into_a_full_tensor() {
        let request = request(u64::MAX, u64::MAX, 8, 1).expect("streamed cardinality");
        assert_eq!(request.document_count(), u64::MAX);
        assert_eq!(request.topic_count(), u64::MAX);
    }

    #[test]
    fn request_accessors_preserve_policy_enums() {
        let request = WorkloadRequest::new(
            2,
            3,
            8,
            16,
            4,
            AdaptationPolicy {
                corpus_placement: CorpusPlacement::StreamedMicroBatches,
                observation_retention: ObservationRetention::KeepAll,
                model_complexity: ModelComplexity::KeepSpecified,
                cutoff_policy: CutoffPolicy::KeepCutoff,
            },
            PrecisionMode::TransientMixed,
        )
        .expect("valid");
        assert_eq!(request.document_count(), 2);
        assert_eq!(request.topic_count(), 3);
        assert_eq!(request.bytes_per_observation(), 8);
        assert_eq!(request.working_set_bytes(), 16);
        assert_eq!(request.requested_batch(), 4);
        assert_eq!(
            request.corpus_placement(),
            CorpusPlacement::StreamedMicroBatches
        );
        assert_eq!(
            request.observation_retention(),
            ObservationRetention::KeepAll
        );
        assert_eq!(request.model_complexity(), ModelComplexity::KeepSpecified);
        assert_eq!(request.cutoff_policy(), CutoffPolicy::KeepCutoff);
        assert_eq!(
            request.final_quantity_precision(),
            PrecisionMode::TransientMixed
        );
    }
}
