//! VRAM controller: reserve, predict, autotune, retry, and fall back.

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
    /// forbidden memory adaptation or mixed-precision finals. An overflowing
    /// peak prediction is treated as unable to fit and falls back to the CPU
    /// reference plan.
    pub fn plan(&self, request: &WorkloadRequest) -> Result<MicroBatchPlan, ComputeBackendError> {
        Self::validate_request(request)?;

        if !self.inventory.device_present() {
            return Ok(Self::cpu_plan(
                request.requested_batch(),
                0,
                FallbackReason::DeviceUnavailable,
            ));
        }

        if self.inventory.budget().usable_bytes() == 0 {
            return Ok(Self::cpu_plan(
                request.requested_batch(),
                0,
                FallbackReason::InsufficientVram,
            ));
        }

        Ok(self.gpu_plan_or_cpu(
            request,
            request.requested_batch(),
            0,
            FallbackReason::InsufficientVram,
        ))
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
            return Ok(self.gpu_plan_or_cpu(
                request,
                plan.batch_size() / 2,
                next_retry,
                FallbackReason::OutOfMemoryRetryExhausted,
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

    fn gpu_plan_or_cpu(
        &self,
        request: &WorkloadRequest,
        mut batch: u32,
        oom_retry_count: u32,
        fallback_reason: FallbackReason,
    ) -> MicroBatchPlan {
        loop {
            if let Some(plan) = self.gpu_plan_if_fits(request, batch, oom_retry_count) {
                return plan;
            }
            if batch == 1 {
                return Self::cpu_plan(request.requested_batch(), oom_retry_count, fallback_reason);
            }
            batch /= 2;
        }
    }

    fn gpu_plan_if_fits(
        &self,
        request: &WorkloadRequest,
        batch: u32,
        oom_retry_count: u32,
    ) -> Option<MicroBatchPlan> {
        let peak = predicted_peak_bytes(
            batch,
            request.bytes_per_observation(),
            request.working_set_bytes(),
        )
        .ok()?;
        if peak > self.inventory.budget().usable_bytes() {
            return None;
        }
        Some(MicroBatchPlan::new(
            ComputeBackendKind::GpuStreamed,
            batch,
            peak,
            PrecisionMode::ReferenceF64,
            oom_retry_count,
            None,
        ))
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
        AdaptationPolicy, CorpusPlacement, CutoffPolicy, ModelComplexity, ObservationRetention,
        PrecisionMode, WorkloadRequest,
    };

    fn request(batch: u32, bytes_per_observation: u64) -> WorkloadRequest {
        WorkloadRequest::new(
            4,
            2,
            bytes_per_observation,
            8,
            batch,
            AdaptationPolicy {
                corpus_placement: CorpusPlacement::StreamedMicroBatches,
                observation_retention: ObservationRetention::KeepAll,
                model_complexity: ModelComplexity::KeepSpecified,
                cutoff_policy: CutoffPolicy::KeepCutoff,
            },
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
    fn overflowing_peak_falls_back_to_cpu() {
        let inventory =
            DeviceInventory::gpu(VramProfile::Gib24, VramProfile::Gib24.bytes()).expect("24");
        let controller = VramController::new(inventory, 1).expect("controller");
        let huge = WorkloadRequest::new(
            1,
            1,
            u64::MAX,
            u64::MAX,
            2,
            AdaptationPolicy {
                corpus_placement: CorpusPlacement::StreamedMicroBatches,
                observation_retention: ObservationRetention::KeepAll,
                model_complexity: ModelComplexity::KeepSpecified,
                cutoff_policy: CutoffPolicy::KeepCutoff,
            },
            PrecisionMode::ReferenceF64,
        )
        .expect("request");
        let plan = controller.plan(&huge).expect("overflow falls back");
        assert_eq!(plan.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(plan.fallback(), Some(FallbackReason::InsufficientVram));
    }

    #[test]
    fn overflowing_oom_retry_peak_falls_back_to_cpu() {
        let inventory =
            DeviceInventory::gpu(VramProfile::Gib24, VramProfile::Gib24.bytes()).expect("24");
        let controller = VramController::new(inventory, 1).expect("controller");
        let huge = WorkloadRequest::new(
            1,
            1,
            u64::MAX,
            u64::MAX,
            2,
            AdaptationPolicy {
                corpus_placement: CorpusPlacement::StreamedMicroBatches,
                observation_retention: ObservationRetention::KeepAll,
                model_complexity: ModelComplexity::KeepSpecified,
                cutoff_policy: CutoffPolicy::KeepCutoff,
            },
            PrecisionMode::ReferenceF64,
        )
        .expect("request");
        let initial = MicroBatchPlan::new(
            ComputeBackendKind::GpuStreamed,
            2,
            0,
            PrecisionMode::ReferenceF64,
            0,
            None,
        );
        let plan = controller
            .recover_from_oom(&huge, &initial)
            .expect("overflow falls back");
        assert_eq!(plan.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(
            plan.fallback(),
            Some(FallbackReason::OutOfMemoryRetryExhausted)
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
        // CPU `f64` fallback restores the requested batch; it is not a GPU retry plan.
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
