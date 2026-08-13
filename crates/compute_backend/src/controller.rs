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
    /// forbidden memory adaptation, mixed-precision finals, or an overflowing
    /// peak prediction.
    pub fn plan(&self, request: &WorkloadRequest) -> Result<MicroBatchPlan, ComputeBackendError> {
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

        if !self.inventory.device_present() {
            return Ok(Self::cpu_plan(
                request.requested_batch(),
                FallbackReason::DeviceUnavailable,
            ));
        }

        let usable = self.inventory.budget().usable_bytes();
        if usable == 0 {
            return Ok(Self::cpu_plan(
                request.requested_batch(),
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
                    None,
                ));
            }
            if batch == 1 {
                return Ok(Self::cpu_plan(
                    request.requested_batch(),
                    FallbackReason::InsufficientVram,
                ));
            }
            batch /= 2;
        }
    }

    /// Treat device OOM as an expected state and fall back after bounded retries.
    ///
    /// The returned CPU plan keeps the original batch so observations are not
    /// dropped. This slice does not claim a live accelerator retry lane.
    ///
    /// # Errors
    ///
    /// Returns [`ComputeBackendError::RetryBudgetExceeded`] when the plan is
    /// already on the CPU reference path.
    pub fn recover_from_oom(
        &self,
        plan: &MicroBatchPlan,
    ) -> Result<MicroBatchPlan, ComputeBackendError> {
        if plan.backend() != ComputeBackendKind::GpuStreamed {
            return Err(ComputeBackendError::RetryBudgetExceeded);
        }
        let mut remaining = self.max_retries;
        let mut batch = plan.batch_size();
        while remaining > 0 {
            remaining -= 1;
            if batch > 1 {
                batch /= 2;
            }
        }
        let _ = batch;
        Ok(Self::cpu_plan(
            plan.batch_size(),
            FallbackReason::OutOfMemoryRetryExhausted,
        ))
    }

    const fn cpu_plan(batch_size: u32, reason: FallbackReason) -> MicroBatchPlan {
        MicroBatchPlan::new(
            ComputeBackendKind::CpuF64Reference,
            batch_size,
            0,
            PrecisionMode::ReferenceF64,
            Some(reason),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::VramController;
    use crate::error::ComputeBackendError;
    use crate::inventory::DeviceInventory;
    use crate::plan::{ComputeBackendKind, FallbackReason};
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
            cpu.recover_from_oom(&planned),
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
    fn oom_recovery_covers_zero_retries_and_unit_batches() {
        let inventory =
            DeviceInventory::gpu(VramProfile::Gib12, VramProfile::Gib12.bytes()).expect("12");
        let zero_retry = VramController::new(inventory, 0).expect("zero retry");
        let planned = zero_retry.plan(&request(4, 8)).expect("gpu");
        assert_eq!(planned.backend(), ComputeBackendKind::GpuStreamed);
        let recovered = zero_retry.recover_from_oom(&planned).expect("fallback");
        assert_eq!(
            recovered.fallback(),
            Some(FallbackReason::OutOfMemoryRetryExhausted)
        );

        let unit_retry = VramController::new(inventory, 3).expect("unit retry");
        let unit_plan = unit_retry.plan(&request(1, 8)).expect("unit gpu");
        assert_eq!(unit_plan.batch_size(), 1);
        let recovered = unit_retry.recover_from_oom(&unit_plan).expect("unit oom");
        assert_eq!(recovered.backend(), ComputeBackendKind::CpuF64Reference);
        assert_eq!(recovered.batch_size(), 1);
    }
}
