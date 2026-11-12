//! Planned backend, micro-batch, and fallback reason.

use crate::request::{PrecisionMode, WorkloadRequest};

/// Executable backend selected by the VRAM controller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComputeBackendKind {
    /// CPU `f64` numerical reference and universal fallback.
    CpuF64Reference,
    /// Streamed GPU plan that still finalizes diagnostics on CPU `f64`.
    GpuStreamed,
}

/// Why a plan left the accelerator or reduced a batch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackReason {
    /// Usable VRAM could not hold even a unit micro-batch.
    InsufficientVram,
    /// Bounded OOM retries still could not keep the work on device.
    OutOfMemoryRetryExhausted,
    /// No accelerator was present.
    DeviceUnavailable,
    /// A non-finite guard forced the CPU reference path.
    NonFiniteGuard,
}

/// A planned micro-batch that preserves the full observation set.
///
/// The plan records the workload it was issued for, so OOM recovery can refuse
/// a plan derived from a different workload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MicroBatchPlan {
    backend: ComputeBackendKind,
    batch_size: u32,
    predicted_peak_bytes: u64,
    precision: PrecisionMode,
    oom_retry_count: u32,
    fallback: Option<FallbackReason>,
    workload: WorkloadRequest,
}

impl MicroBatchPlan {
    pub(crate) const fn new(
        backend: ComputeBackendKind,
        batch_size: u32,
        predicted_peak_bytes: u64,
        precision: PrecisionMode,
        oom_retry_count: u32,
        fallback: Option<FallbackReason>,
        workload: WorkloadRequest,
    ) -> Self {
        Self {
            backend,
            batch_size,
            predicted_peak_bytes,
            precision,
            oom_retry_count,
            fallback,
            workload,
        }
    }

    /// Return the selected backend.
    #[must_use]
    pub const fn backend(self) -> ComputeBackendKind {
        self.backend
    }

    /// Return the planned micro-batch size.
    #[must_use]
    pub const fn batch_size(self) -> u32 {
        self.batch_size
    }

    /// Return the predicted peak working-set plus batch charge.
    #[must_use]
    pub const fn predicted_peak_bytes(self) -> u64 {
        self.predicted_peak_bytes
    }

    /// Return the precision used for final diagnostics.
    #[must_use]
    pub const fn precision(self) -> PrecisionMode {
        self.precision
    }

    /// Return how many observed OOMs led to this plan.
    #[must_use]
    pub const fn oom_retry_count(self) -> u32 {
        self.oom_retry_count
    }

    /// Return the fallback reason, if the accelerator was not used.
    #[must_use]
    pub const fn fallback(self) -> Option<FallbackReason> {
        self.fallback
    }

    /// Return whether this plan was issued for a workload equal in every field
    /// to `request`.
    #[must_use]
    pub fn issued_for(&self, request: &WorkloadRequest) -> bool {
        self.workload == *request
    }
}

/// Predict peak bytes for a micro-batch plus fixed working set.
///
/// # Errors
///
/// Returns [`crate::ComputeBackendError::InvalidBudget`] on overflow.
pub const fn predicted_peak_bytes(
    batch_size: u32,
    bytes_per_observation: u64,
    working_set_bytes: u64,
) -> Result<u64, crate::ComputeBackendError> {
    let Some(batch_bytes) = bytes_per_observation.checked_mul(batch_size as u64) else {
        return Err(crate::ComputeBackendError::InvalidBudget);
    };
    match batch_bytes.checked_add(working_set_bytes) {
        Some(peak) => Ok(peak),
        None => Err(crate::ComputeBackendError::InvalidBudget),
    }
}

#[cfg(test)]
mod tests {
    use super::{ComputeBackendKind, FallbackReason, MicroBatchPlan, predicted_peak_bytes};
    use crate::error::ComputeBackendError;
    use crate::request::{
        AdaptationPolicy, CorpusPlacement, CutoffPolicy, ModelComplexity, ObservationRetention,
        PrecisionMode, WorkloadRequest,
    };

    fn workload(batch: u32) -> WorkloadRequest {
        WorkloadRequest::new(
            4,
            2,
            8,
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
        .expect("valid workload")
    }

    #[test]
    fn peak_prediction_and_plan_accessors() {
        assert_eq!(predicted_peak_bytes(2, 8, 16).expect("peak"), 32);
        assert_eq!(
            predicted_peak_bytes(2, u64::MAX, 1),
            Err(ComputeBackendError::InvalidBudget)
        );
        assert_eq!(
            predicted_peak_bytes(1, u64::MAX, 1),
            Err(ComputeBackendError::InvalidBudget)
        );
        let plan = MicroBatchPlan::new(
            ComputeBackendKind::CpuF64Reference,
            3,
            24,
            PrecisionMode::ReferenceF64,
            2,
            Some(FallbackReason::NonFiniteGuard),
            workload(3),
        );
        assert_eq!(plan.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(plan.batch_size(), 3);
        assert_eq!(plan.predicted_peak_bytes(), 24);
        assert_eq!(plan.precision(), PrecisionMode::ReferenceF64);
        assert_eq!(plan.oom_retry_count(), 2);
        assert_eq!(plan.fallback(), Some(FallbackReason::NonFiniteGuard));
        assert!(plan.issued_for(&workload(3)));
        assert!(!plan.issued_for(&workload(4)));
    }
}
