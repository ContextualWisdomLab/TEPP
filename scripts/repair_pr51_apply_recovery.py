"""Apply PR 51 OOM recovery, numerical reference, and documentation repairs."""

from pathlib import Path


def ensure_after(path: str, marker: str, insertion: str) -> None:
    """Insert text after one marker unless already present."""
    file_path = Path(path)
    text = file_path.read_text(encoding="utf-8")
    if insertion in text:
        return
    count = text.count(marker)
    if count != 1:
        raise SystemExit(f"{path}: expected one insertion marker, found {count}")
    file_path.write_text(text.replace(marker, marker + insertion, 1), encoding="utf-8")


CONTROLLER = r'''//! VRAM controller: reserve, predict, autotune, retry, and fall back.

use crate::error::ComputeBackendError;
use crate::inventory::{DeviceInventory, SafetyReserve, VramBudget};
use crate::plan::{ComputeBackendKind, FallbackReason, MicroBatchPlan, predicted_peak_bytes};
use crate::request::{
    CorpusPlacement, CutoffPolicy, ModelComplexity, ObservationRetention, PrecisionMode,
    WorkloadRequest,
};

/// Plans streamed work under a VRAM budget without changing the estimand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VramController {
    inventory: DeviceInventory,
    max_retries: u32,
}

impl VramController {
    /// Construct a controller with a bounded OOM retry budget.
    ///
    /// # Errors
    ///
    /// This constructor is currently infallible for valid inventories. It
    /// returns [`Result`] so callers can share the crate error type.
    pub const fn new(
        inventory: DeviceInventory,
        max_retries: u32,
    ) -> Result<Self, ComputeBackendError> {
        Ok(Self {
            inventory,
            max_retries,
        })
    }

    /// Return the reserved safety headroom.
    #[must_use]
    pub const fn safety_reserve(self) -> SafetyReserve {
        self.inventory.safety_reserve()
    }

    /// Return the usable VRAM budget.
    #[must_use]
    pub const fn budget(self) -> VramBudget {
        self.inventory.budget()
    }

    /// Return the bounded OOM retry budget.
    #[must_use]
    pub const fn max_retries(self) -> u32 {
        self.max_retries
    }

    /// Plan a micro-batch or CPU fallback without dropping observations.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed [`ComputeBackendError`] when the caller requests a
    /// forbidden memory adaptation, mixed-precision finals, or an overflowing
    /// peak prediction.
    pub fn plan(&self, request: &WorkloadRequest) -> Result<MicroBatchPlan, ComputeBackendError> {
        Self::validate_request(request)?;

        if !self.inventory.device_present() {
            return Ok(Self::cpu_plan(
                request.requested_batch(),
                0,
                FallbackReason::DeviceUnavailable,
            ));
        }

        let usable = self.inventory.budget().usable_bytes();
        if usable == 0 {
            return Ok(Self::cpu_plan(
                request.requested_batch(),
                0,
                FallbackReason::InsufficientVram,
            ));
        }

        let mut batch = request.requested_batch();
        loop {
            let peak = predicted_peak_bytes(
                batch,
                request.bytes_per_observation(),
                request.working_set_bytes(),
            )?;
            if peak <= usable {
                return Ok(MicroBatchPlan::new(
                    ComputeBackendKind::GpuStreamed,
                    batch,
                    peak,
                    PrecisionMode::ReferenceF64,
                    0,
                    None,
                ));
            }
            if batch == 1 {
                return Ok(Self::cpu_plan(
                    request.requested_batch(),
                    0,
                    FallbackReason::InsufficientVram,
                ));
            }
            batch /= 2;
        }
    }

    /// Return the next executable plan after one observed device OOM.
    ///
    /// Each accepted retry halves the current micro-batch and recomputes its
    /// peak estimate from the original workload. Once the configured retry
    /// budget is exhausted, or a unit batch fails, the plan switches to the CPU
    /// `f64` reference without dropping any observation.
    ///
    /// # Errors
    ///
    /// Returns [`ComputeBackendError::RetryBudgetExceeded`] when the supplied
    /// plan is already on the CPU path, and validation/overflow errors for an
    /// invalid workload or retry counter.
    pub fn recover_from_oom(
        &self,
        request: &WorkloadRequest,
        plan: &MicroBatchPlan,
    ) -> Result<MicroBatchPlan, ComputeBackendError> {
        Self::validate_request(request)?;
        if plan.backend() != ComputeBackendKind::GpuStreamed {
            return Err(ComputeBackendError::RetryBudgetExceeded);
        }
        let next_retry = plan
            .oom_retry_count()
            .checked_add(1)
            .ok_or(ComputeBackendError::InvalidBudget)?;
        if next_retry <= self.max_retries && plan.batch_size() > 1 {
            let batch = plan.batch_size() / 2;
            let peak = predicted_peak_bytes(
                batch,
                request.bytes_per_observation(),
                request.working_set_bytes(),
            )?;
            return Ok(MicroBatchPlan::new(
                ComputeBackendKind::GpuStreamed,
                batch,
                peak,
                PrecisionMode::ReferenceF64,
                next_retry,
                None,
            ));
        }
        Ok(Self::cpu_plan(
            request.requested_batch(),
            next_retry,
            FallbackReason::OutOfMemoryRetryExhausted,
        ))
    }

    fn validate_request(request: &WorkloadRequest) -> Result<(), ComputeBackendError> {
        if request.corpus_placement() == CorpusPlacement::FullCorpusOnDevice {
            return Err(ComputeBackendError::FullCorpusTensorRefused);
        }
        if request.observation_retention() == ObservationRetention::DropToFit {
            return Err(ComputeBackendError::ObservationDropForbidden);
        }
        if request.model_complexity() == ModelComplexity::ReduceToFit {
            return Err(ComputeBackendError::ComplexityReductionForbidden);
        }
        if request.cutoff_policy() == CutoffPolicy::MoveToFit {
            return Err(ComputeBackendError::CutoffMutationForbidden);
        }
        if request.final_quantity_precision() != PrecisionMode::ReferenceF64 {
            return Err(ComputeBackendError::UnsupportedPrecision);
        }
        Ok(())
    }

    const fn cpu_plan(
        batch_size: u32,
        oom_retry_count: u32,
        reason: FallbackReason,
    ) -> MicroBatchPlan {
        MicroBatchPlan::new(
            ComputeBackendKind::CpuF64Reference,
            batch_size,
            0,
            PrecisionMode::ReferenceF64,
            oom_retry_count,
            Some(reason),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::VramController;
    use crate::error::ComputeBackendError;
    use crate::inventory::DeviceInventory;
    use crate::plan::{ComputeBackendKind, FallbackReason, MicroBatchPlan};
    use crate::profile::VramProfile;
    use crate::request::{
        CorpusPlacement, CutoffPolicy, ModelComplexity, ObservationRetention, PrecisionMode,
        WorkloadRequest,
    };

    fn request(batch: u32, bytes_per_observation: u64) -> WorkloadRequest {
        WorkloadRequest::new(
            4,
            2,
            bytes_per_observation,
            8,
            batch,
            CorpusPlacement::StreamedMicroBatches,
            ObservationRetention::KeepAll,
            ModelComplexity::KeepSpecified,
            CutoffPolicy::KeepCutoff,
            PrecisionMode::ReferenceF64,
        )
        .expect("valid")
    }

    #[test]
    fn cpu_only_and_unusable_vram_fall_back() {
        let cpu = VramController::new(DeviceInventory::cpu_only(VramProfile::Gib4), 1)
            .expect("cpu controller");
        assert_eq!(cpu.max_retries(), 1);
        assert_eq!(
            cpu.safety_reserve().bytes(),
            VramProfile::Gib4.safety_bytes()
        );
        assert_eq!(cpu.budget().usable_bytes(), 0);
        let planned = cpu.plan(&request(4, 8)).expect("cpu plan");
        assert_eq!(planned.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(planned.fallback(), Some(FallbackReason::DeviceUnavailable));
        assert_eq!(
            cpu.recover_from_oom(&request(4, 8), &planned),
            Err(ComputeBackendError::RetryBudgetExceeded)
        );

        let tight = DeviceInventory::gpu(VramProfile::Gib4, VramProfile::Gib4.safety_bytes())
            .expect("tight");
        let controller = VramController::new(tight, 0).expect("tight controller");
        let planned = controller.plan(&request(2, 8)).expect("unusable");
        assert_eq!(planned.fallback(), Some(FallbackReason::InsufficientVram));
    }

    #[test]
    fn unit_batch_that_still_exceeds_usable_vram_falls_back() {
        let available = VramProfile::Gib4.safety_bytes() + 16;
        let inventory = DeviceInventory::gpu(VramProfile::Gib4, available).expect("small usable");
        let controller = VramController::new(inventory, 1).expect("controller");
        let planned = controller.plan(&request(8, 64)).expect("fallback");
        assert_eq!(planned.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(planned.fallback(), Some(FallbackReason::InsufficientVram));
        assert_eq!(planned.batch_size(), 8);
        assert_eq!(planned.precision(), PrecisionMode::ReferenceF64);
        assert_eq!(planned.predicted_peak_bytes(), 0);
        assert_eq!(planned.oom_retry_count(), 0);
    }

    #[test]
    fn overflowing_peak_fails_closed() {
        let inventory =
            DeviceInventory::gpu(VramProfile::Gib24, VramProfile::Gib24.bytes()).expect("24");
        let controller = VramController::new(inventory, 1).expect("controller");
        let huge = WorkloadRequest::new(
            1,
            1,
            u64::MAX,
            u64::MAX,
            2,
            CorpusPlacement::StreamedMicroBatches,
            ObservationRetention::KeepAll,
            ModelComplexity::KeepSpecified,
            CutoffPolicy::KeepCutoff,
            PrecisionMode::ReferenceF64,
        )
        .expect("request");
        assert_eq!(
            controller.plan(&huge),
            Err(ComputeBackendError::InvalidBudget)
        );
    }

    #[test]
    fn oom_recovery_emits_retries_then_falls_back() {
        let inventory =
            DeviceInventory::gpu(VramProfile::Gib12, VramProfile::Gib12.bytes()).expect("12");
        let controller = VramController::new(inventory, 1).expect("controller");
        let workload = request(4, 8);
        let initial = controller.plan(&workload).expect("gpu");
        let retry = controller
            .recover_from_oom(&workload, &initial)
            .expect("retry");
        assert_eq!(retry.backend(), ComputeBackendKind::GpuStreamed);
        assert_eq!(retry.batch_size(), 2);
        assert_eq!(retry.oom_retry_count(), 1);
        let fallback = controller
            .recover_from_oom(&workload, &retry)
            .expect("fallback");
        assert_eq!(fallback.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(fallback.batch_size(), 4);
        assert_eq!(fallback.oom_retry_count(), 2);

        let zero_retry = VramController::new(inventory, 0).expect("zero retry");
        let immediate = zero_retry
            .recover_from_oom(&workload, &initial)
            .expect("immediate fallback");
        assert_eq!(immediate.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(immediate.oom_retry_count(), 1);

        let unit_workload = request(1, 8);
        let unit_plan = controller.plan(&unit_workload).expect("unit gpu");
        let unit_fallback = controller
            .recover_from_oom(&unit_workload, &unit_plan)
            .expect("unit fallback");
        assert_eq!(unit_fallback.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(unit_fallback.batch_size(), 1);
    }

    #[test]
    fn overflowing_retry_counter_fails_closed() {
        let inventory =
            DeviceInventory::gpu(VramProfile::Gib12, VramProfile::Gib12.bytes()).expect("12");
        let controller = VramController::new(inventory, u32::MAX).expect("controller");
        let workload = request(4, 8);
        let invalid = MicroBatchPlan::new(
            ComputeBackendKind::GpuStreamed,
            4,
            40,
            PrecisionMode::ReferenceF64,
            u32::MAX,
            None,
        );
        assert_eq!(
            controller.recover_from_oom(&workload, &invalid),
            Err(ComputeBackendError::InvalidBudget)
        );
    }
}
'''

PLAN = r'''//! Planned backend, micro-batch, and fallback reason.

use crate::request::PrecisionMode;

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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MicroBatchPlan {
    backend: ComputeBackendKind,
    batch_size: u32,
    predicted_peak_bytes: u64,
    precision: PrecisionMode,
    oom_retry_count: u32,
    fallback: Option<FallbackReason>,
}

impl MicroBatchPlan {
    pub(crate) const fn new(
        backend: ComputeBackendKind,
        batch_size: u32,
        predicted_peak_bytes: u64,
        precision: PrecisionMode,
        oom_retry_count: u32,
        fallback: Option<FallbackReason>,
    ) -> Self {
        Self {
            backend,
            batch_size,
            predicted_peak_bytes,
            precision,
            oom_retry_count,
            fallback,
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
    use crate::request::PrecisionMode;

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
        );
        assert_eq!(plan.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(plan.batch_size(), 3);
        assert_eq!(plan.predicted_peak_bytes(), 24);
        assert_eq!(plan.precision(), PrecisionMode::ReferenceF64);
        assert_eq!(plan.oom_retry_count(), 2);
        assert_eq!(plan.fallback(), Some(FallbackReason::NonFiniteGuard));
    }
}
'''

REQUEST = r'''//! Workload request and precision policy.

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
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
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
            corpus_placement,
            observation_retention,
            model_complexity,
            cutoff_policy,
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
        CorpusPlacement, CutoffPolicy, ModelComplexity, ObservationRetention, PrecisionMode,
        WorkloadRequest,
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
            CorpusPlacement::StreamedMicroBatches,
            ObservationRetention::KeepAll,
            ModelComplexity::KeepSpecified,
            CutoffPolicy::KeepCutoff,
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
            CorpusPlacement::StreamedMicroBatches,
            ObservationRetention::KeepAll,
            ModelComplexity::KeepSpecified,
            CutoffPolicy::KeepCutoff,
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
'''

REFERENCE = r'''//! CPU `f64` streamed reference arithmetic.

use crate::error::ComputeBackendError;

/// Stream a compensated weighted sum on the CPU `f64` reference path.
///
/// Neumaier-style compensation preserves low-order terms in cancellation-heavy
/// inputs while keeping deterministic input order. This sequential function is
/// the numerical reference for later fixed-pool CPU and GPU implementations.
///
/// # Errors
///
/// Returns [`ComputeBackendError::InvalidBudget`] when the slices are empty or
/// unequal, and [`ComputeBackendError::NonFiniteOutput`] when any term or
/// accumulator is non-finite.
pub fn streamed_weighted_sum(weights: &[f64], values: &[f64]) -> Result<f64, ComputeBackendError> {
    if weights.is_empty() || weights.len() != values.len() {
        return Err(ComputeBackendError::InvalidBudget);
    }
    let mut total = 0.0_f64;
    let mut compensation = 0.0_f64;
    for (weight, value) in weights.iter().zip(values) {
        let term = require_finite(*weight)? * require_finite(*value)?;
        let term = require_finite(term)?;
        let next = require_finite(total + term)?;
        let correction = if total.abs() >= term.abs() {
            (total - next) + term
        } else {
            (term - next) + total
        };
        compensation = require_finite(compensation + correction)?;
        total = next;
    }
    require_finite(total + compensation)
}

/// Reject a non-finite diagnostic quantity.
///
/// # Errors
///
/// Returns [`ComputeBackendError::NonFiniteOutput`] when `value` is NaN or
/// infinite.
pub fn require_finite(value: f64) -> Result<f64, ComputeBackendError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ComputeBackendError::NonFiniteOutput)
    }
}

/// Compare a candidate quantity against the CPU `f64` reference.
///
/// # Errors
///
/// Returns [`ComputeBackendError::NonFiniteOutput`] when either value or the
/// tolerance is non-finite, [`ComputeBackendError::InvalidTolerance`] for a
/// negative tolerance, and [`ComputeBackendError::ParityFailure`] when the
/// absolute gap exceeds the non-negative tolerance.
pub fn require_cpu_gpu_parity(
    cpu_reference: f64,
    candidate: f64,
    tolerance: f64,
) -> Result<(), ComputeBackendError> {
    let left = require_finite(cpu_reference)?;
    let right = require_finite(candidate)?;
    let bound = require_finite(tolerance)?;
    if bound < 0.0 {
        return Err(ComputeBackendError::InvalidTolerance);
    }
    if (left - right).abs() <= bound {
        Ok(())
    } else {
        Err(ComputeBackendError::ParityFailure)
    }
}

#[cfg(test)]
mod tests {
    use super::{require_cpu_gpu_parity, require_finite, streamed_weighted_sum};
    use crate::error::ComputeBackendError;

    #[test]
    fn compensated_reference_recovers_low_order_cancellation_term() {
        let result = streamed_weighted_sum(&[1.0, 1.0, 1.0], &[1e16, 1.0, -1e16])
            .expect("compensated sum");
        assert!((result - 1.0).abs() < 1e-15);
        let reverse = streamed_weighted_sum(&[1.0, 1.0, 1.0], &[-1e16, 1.0, 1e16])
            .expect("reverse compensation branch");
        assert!((reverse - 1.0).abs() < 1e-15);
    }

    #[test]
    fn reference_path_rejects_invalid_and_non_finite_input() {
        assert_eq!(
            streamed_weighted_sum(&[], &[1.0]),
            Err(ComputeBackendError::InvalidBudget)
        );
        assert_eq!(
            streamed_weighted_sum(&[1.0], &[1.0, 2.0]),
            Err(ComputeBackendError::InvalidBudget)
        );
        assert_eq!(
            streamed_weighted_sum(&[f64::NAN], &[1.0]),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            streamed_weighted_sum(&[1.0], &[f64::INFINITY]),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            streamed_weighted_sum(&[1e308], &[1e308]),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            require_finite(f64::NEG_INFINITY),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        let finite = require_finite(1.5).expect("finite");
        assert!((finite - 1.5).abs() < 1e-15);
        require_cpu_gpu_parity(1.0, 1.0, 0.0).expect("exact parity");
        assert_eq!(
            require_cpu_gpu_parity(1.0, 2.0, 0.1),
            Err(ComputeBackendError::ParityFailure)
        );
        assert_eq!(
            require_cpu_gpu_parity(1.0, 1.0, -0.1),
            Err(ComputeBackendError::InvalidTolerance)
        );
        assert_eq!(
            require_cpu_gpu_parity(f64::NAN, 1.0, 0.1),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            require_cpu_gpu_parity(1.0, f64::NAN, 0.1),
            Err(ComputeBackendError::NonFiniteOutput)
        );
        assert_eq!(
            require_cpu_gpu_parity(1.0, 1.0, f64::NAN),
            Err(ComputeBackendError::NonFiniteOutput)
        );
    }
}
'''

ERROR = r'''//! Fail-closed VRAM and compute-backend errors.

use std::fmt;

/// A fail-closed compute-backend error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ComputeBackendError {
    /// Device allocation failed. This is an expected operating state.
    OutOfMemory,
    /// The accelerator disappeared after planning.
    DeviceLoss,
    /// A reference or diagnostic quantity was non-finite.
    NonFiniteOutput,
    /// CPU `f64` and candidate outputs diverged beyond tolerance.
    ParityFailure,
    /// A parity tolerance was negative.
    InvalidTolerance,
    /// Mixed precision was requested for a final diagnostic quantity.
    UnsupportedPrecision,
    /// A claimed accelerator could not be initialized.
    BackendInitFailure,
    /// A full document-by-topic tensor was requested on device memory.
    FullCorpusTensorRefused,
    /// Observations would be dropped to fit memory.
    ObservationDropForbidden,
    /// Topic or model complexity would be reduced to fit memory.
    ComplexityReductionForbidden,
    /// A knowledge cutoff would change to fit memory.
    CutoffMutationForbidden,
    /// A budget, inventory, or workload field was empty or overflowed.
    InvalidBudget,
    /// Telemetry attempted to carry raw source text.
    SourceTextInTelemetry,
    /// Further OOM retries were requested after the bounded budget.
    RetryBudgetExceeded,
}

impl ComputeBackendError {
    /// Return whether the error is a tested operating state rather than a bug.
    #[must_use]
    pub const fn is_expected_operating_state(self) -> bool {
        matches!(self, Self::OutOfMemory)
    }
}

impl fmt::Display for ComputeBackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::OutOfMemory => "device out of memory",
            Self::DeviceLoss => "compute device lost",
            Self::NonFiniteOutput => "non-finite compute output",
            Self::ParityFailure => "cpu gpu parity failure",
            Self::InvalidTolerance => "invalid parity tolerance",
            Self::UnsupportedPrecision => "mixed precision cannot finalize diagnostics",
            Self::BackendInitFailure => "compute backend initialization failed",
            Self::FullCorpusTensorRefused => "full-corpus device tensor is refused",
            Self::ObservationDropForbidden => "observations cannot be dropped to fit memory",
            Self::ComplexityReductionForbidden => {
                "model complexity cannot be reduced to fit memory"
            }
            Self::CutoffMutationForbidden => "knowledge cutoff cannot change to fit memory",
            Self::InvalidBudget => "invalid compute budget",
            Self::SourceTextInTelemetry => "telemetry cannot carry source text",
            Self::RetryBudgetExceeded => "oom retry budget exceeded",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ComputeBackendError {}

/// Return the typed out-of-memory operating state.
#[must_use]
pub const fn report_out_of_memory() -> ComputeBackendError {
    ComputeBackendError::OutOfMemory
}

/// Return the typed device-loss failure.
#[must_use]
pub const fn report_device_loss() -> ComputeBackendError {
    ComputeBackendError::DeviceLoss
}

/// Return the typed backend-initialization failure.
#[must_use]
pub const fn refuse_uninitialized_backend() -> ComputeBackendError {
    ComputeBackendError::BackendInitFailure
}

#[cfg(test)]
mod tests {
    use super::{
        ComputeBackendError, refuse_uninitialized_backend, report_device_loss, report_out_of_memory,
    };

    #[test]
    fn messages_and_operating_states_are_stable() {
        for (error, message, expected) in [
            (ComputeBackendError::OutOfMemory, "device out of memory", true),
            (ComputeBackendError::DeviceLoss, "compute device lost", false),
            (
                ComputeBackendError::NonFiniteOutput,
                "non-finite compute output",
                false,
            ),
            (
                ComputeBackendError::ParityFailure,
                "cpu gpu parity failure",
                false,
            ),
            (
                ComputeBackendError::InvalidTolerance,
                "invalid parity tolerance",
                false,
            ),
            (
                ComputeBackendError::UnsupportedPrecision,
                "mixed precision cannot finalize diagnostics",
                false,
            ),
            (
                ComputeBackendError::BackendInitFailure,
                "compute backend initialization failed",
                false,
            ),
            (
                ComputeBackendError::FullCorpusTensorRefused,
                "full-corpus device tensor is refused",
                false,
            ),
            (
                ComputeBackendError::ObservationDropForbidden,
                "observations cannot be dropped to fit memory",
                false,
            ),
            (
                ComputeBackendError::ComplexityReductionForbidden,
                "model complexity cannot be reduced to fit memory",
                false,
            ),
            (
                ComputeBackendError::CutoffMutationForbidden,
                "knowledge cutoff cannot change to fit memory",
                false,
            ),
            (
                ComputeBackendError::InvalidBudget,
                "invalid compute budget",
                false,
            ),
            (
                ComputeBackendError::SourceTextInTelemetry,
                "telemetry cannot carry source text",
                false,
            ),
            (
                ComputeBackendError::RetryBudgetExceeded,
                "oom retry budget exceeded",
                false,
            ),
        ] {
            assert_eq!(error.to_string(), message);
            assert_eq!(error.is_expected_operating_state(), expected);
        }
        assert_eq!(report_out_of_memory(), ComputeBackendError::OutOfMemory);
        assert_eq!(report_device_loss(), ComputeBackendError::DeviceLoss);
        assert_eq!(
            refuse_uninitialized_backend(),
            ComputeBackendError::BackendInitFailure
        );
    }
}
'''

for path, content in (
    ("crates/compute_backend/src/controller.rs", CONTROLLER),
    ("crates/compute_backend/src/plan.rs", PLAN),
    ("crates/compute_backend/src/request.rs", REQUEST),
    ("crates/compute_backend/src/reference.rs", REFERENCE),
    ("crates/compute_backend/src/error.rs", ERROR),
):
    Path(path).write_text(content, encoding="utf-8")

cargo_path = Path("Cargo.toml")
cargo = cargo_path.read_text(encoding="utf-8")
for section_marker in (
    '    "crates/tepp_api",\n]',
):
    while cargo.count(section_marker) > 0:
        cargo = cargo.replace(
            section_marker,
            '    "crates/tepp_api",\n    "crates/compute_backend",\n]',
            1,
        )
        if cargo.count('    "crates/compute_backend",') >= 2:
            break
if cargo.count('    "crates/compute_backend",') != 2:
    raise SystemExit("Cargo.toml compute_backend membership mismatch")
cargo_path.write_text(cargo, encoding="utf-8")

ensure_after(
    "scripts/check_workspace_contract.py",
    '    "tepp_api",\n',
    '    "compute_backend",\n',
)

quality_path = Path("tests/quality/test_check_docstrings.py")
quality = quality_path.read_text(encoding="utf-8")
if "from scripts import check_workspace_contract as contract" not in quality:
    quality = quality.replace(
        "from scripts import check_docstrings as docstrings\n",
        "from scripts import check_docstrings as docstrings\nfrom scripts import check_workspace_contract as contract\n",
        1,
    )
quality = quality.replace(
    "self.assertEqual(len(crate_roots), 10)",
    "self.assertEqual(len(crate_roots), len(contract.EXPECTED_CRATES))",
)
quality_path.write_text(quality, encoding="utf-8")

ensure_after(
    "ARCHITECTURE.md",
    "| `tepp_api` | versioned DTO, schema, and export contracts |\n",
    "| `compute_backend` | VRAM-budgeted streamed planning, executable OOM retry plans, and a compensated CPU `f64` reference |\n",
)
ensure_after(
    "DOCUMENTATION.md",
    "| Actions fleet research doctoring | [`docs/research/actions-workflow-fleet.md`](docs/research/actions-workflow-fleet.md) |\n",
    "| VRAM budget / GPU fallback doctoring | [`docs/research/vram-budget-types.md`](docs/research/vram-budget-types.md) |\n",
)

RESEARCH = r'''# VRAM budget types, executable OOM retries, and CPU `f64` reference

## Scope

This slice delivers the first executable ADR 0006 contract in `compute_backend`:

1. classify devices into the accepted 4/6/8/12/24-GiB profiles;
2. reserve one eighth of profile capacity as unused safety memory;
3. predict peak bytes as `batch × bytes_per_observation + working_set`;
4. autotune the micro-batch by successive halving until the predicted peak fits usable VRAM;
5. after each observed OOM, emit a smaller executable GPU plan with an incremented retry count, then fall back to the CPU `f64` reference after the bounded retry budget or a failed unit batch;
6. refuse full-corpus document-by-topic device tensors and refuse dropping observations, shrinking topic/model complexity, or moving a knowledge cutoff to fit memory;
7. keep mixed precision out of final diagnostic quantities and reject negative parity tolerances;
8. keep raw source text out of allocation telemetry;
9. use compensated deterministic summation for the sequential CPU `f64` numerical reference.

Live CUDA/WGPU kernels, deterministic fixed-pool CPU multithreading, mixed-precision device lanes, and hardware CPU/GPU parity remain accepted-target. This slice does not claim an accelerator or a multithreaded production estimator.

## Authoritative sources

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://standards.ieee.org/ieee/754/6210/

Micikevicius, P., Narang, S., Alben, J., Diamos, G., Elsen, E., Garcia, D., Ginsburg, B., Houston, M., Kuchaiev, O., Venkatesh, G., & Wu, H. (2018). Mixed precision training. In *International Conference on Learning Representations*. https://openreview.net/forum?id=r1gs9JgRZ

NVIDIA Corporation. (2024). *CUDA C++ programming guide*. https://docs.nvidia.com/cuda/cuda-c-programming-guide/

Ogita, T., Rump, S. M., & Oishi, S. (2005). Accurate sum and dot product. *SIAM Journal on Scientific Computing, 26*(6), 1955–1988. https://doi.org/10.1137/030601818

Rhu, M., Gimelshein, N., Clemons, J., Zulfiqar, A., & Keckler, S. W. (2016). vDNN: Virtualized deep neural networks for scalable, memory-efficient neural network design. In *2016 49th Annual IEEE/ACM International Symposium on Microarchitecture (MICRO)* (pp. 1–13). IEEE. https://doi.org/10.1109/MICRO.2016.7783721

## Formula notes

- **Profile capacity** is \(p \times 2^{30}\) bytes for \(p \in \{4,6,8,12,24\}\).
- **Safety reserve** is \(p \times 2^{30} / 8\). Usable VRAM is \(\max(0, a - s)\) for available bytes \(a\) and reserve \(s\).
- **Peak** is \(b \cdot c + w\) for batch \(b\), per-observation charge \(c\), and working set \(w\). Overflow fails closed.
- **OOM retry** is stateful: retry count \(r\) increments after each observed OOM, batch is halved when \(r\leq r_{max}\), and the peak is recomputed from the original workload. No loop is counted as a retry unless an executable plan is returned to the caller.
- **CPU `f64` reference** uses deterministic compensated summation in IEEE 754 binary64 so cancellation-heavy low-order terms are not needlessly discarded (IEEE, 2019; Ogita et al., 2005).
- Streamed document/topic cardinalities are not multiplied into a hypothetical full-corpus allocation; the forbidden full-corpus policy is rejected by the controller.
- Mixed precision may be recorded as a transient mode only; final diagnostics remain binary64 (Micikevicius et al., 2018).

## Verification

- cancellation-heavy CPU `f64` weighted sums recover the low-order term and known totals with computed RMSE;
- 24-GiB profiles admit a larger autotuned micro-batch than 4-GiB profiles for the same workload;
- each accepted OOM retry returns a smaller GPU plan and an exact retry count before CPU fallback;
- streamed extreme cardinalities remain valid because no full tensor is sized;
- negative parity tolerances, full-corpus placement, observation drop, complexity reduction, cutoff mutation, mixed-final precision, and source-text telemetry fail closed.
'''
Path("docs/research/vram-budget-types.md").write_text(RESEARCH, encoding="utf-8")

changelog_path = Path("CHANGELOG.md")
changelog = changelog_path.read_text(encoding="utf-8")
bullet = "- `compute_backend` ADR 0006 first slice: VRAM profiles and reserve-aware micro-batching, executable successive OOM retry plans, CPU fallback, compensated `f64` reference arithmetic, non-negative parity tolerance, and fail-closed estimand-preserving memory policies.\n"
if bullet not in changelog:
    marker = "### Added\n\n"
    if changelog.count(marker) != 1:
        raise SystemExit("CHANGELOG Added marker mismatch")
    changelog = changelog.replace(marker, marker + bullet, 1)
changelog_path.write_text(changelog, encoding="utf-8")
