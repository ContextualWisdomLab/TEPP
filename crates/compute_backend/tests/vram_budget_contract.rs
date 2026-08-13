//! VRAM budget, OOM fallback, and CPU `f64` reference contracts.
#![allow(clippy::cast_precision_loss)]

use compute_backend::{
    AllocationTelemetry, ComputeBackendError, ComputeBackendKind, CorpusPlacement, CutoffPolicy,
    DeviceInventory, FallbackReason, ModelComplexity, ObservationRetention, PrecisionMode,
    VramController, VramProfile, WorkloadRequest, streamed_weighted_sum,
};

fn rmse(truth: &[f64], recovered: &[f64]) -> f64 {
    let n = truth.len() as f64;
    let sum_sq: f64 = truth
        .iter()
        .zip(recovered)
        .map(|(left, right)| {
            let residual = left - right;
            residual * residual
        })
        .sum();
    (sum_sq / n).sqrt()
}

fn base_request(batch: u32, bytes_per_observation: u64) -> WorkloadRequest {
    WorkloadRequest::new(
        1_024,
        64,
        bytes_per_observation,
        1_048_576,
        batch,
        CorpusPlacement::StreamedMicroBatches,
        ObservationRetention::KeepAll,
        ModelComplexity::KeepSpecified,
        CutoffPolicy::KeepCutoff,
        PrecisionMode::ReferenceF64,
    )
    .expect("valid workload")
}

#[test]
fn profiles_cover_the_adr_device_classes() {
    let profiles = VramProfile::all();
    assert_eq!(profiles.map(VramProfile::gibibytes), [4, 6, 8, 12, 24]);
    assert_eq!(VramProfile::Gib4.bytes(), 4 * (1 << 30));
    assert_eq!(VramProfile::Gib24.bytes(), 24 * (1 << 30));
}

#[test]
fn streamed_weighted_sum_recovers_known_total_with_computed_rmse() {
    let weights = [0.25_f64, 0.25, 0.25, 0.25];
    let values = [4.0_f64, 8.0, 12.0, 16.0];
    let truth = 10.0_f64;
    let recovered = streamed_weighted_sum(&weights, &values).expect("finite reference");
    let error = rmse(&[truth], &[recovered]);
    assert!(
        error < 1e-12,
        "CPU f64 RMSE {error} exceeded machine-scale bound"
    );
}

#[test]
fn larger_vram_profiles_admit_larger_micro_batches() {
    let request = base_request(1_024, 4_194_304);
    let small = VramController::new(
        DeviceInventory::gpu(VramProfile::Gib4, VramProfile::Gib4.bytes()).expect("4 GiB"),
        3,
    )
    .expect("controller")
    .plan(&request)
    .expect("4 GiB plan");
    let large = VramController::new(
        DeviceInventory::gpu(VramProfile::Gib24, VramProfile::Gib24.bytes()).expect("24 GiB"),
        3,
    )
    .expect("controller")
    .plan(&request)
    .expect("24 GiB plan");

    assert_eq!(small.backend(), ComputeBackendKind::GpuStreamed);
    assert_eq!(large.backend(), ComputeBackendKind::GpuStreamed);
    assert!(
        large.batch_size() > small.batch_size(),
        "24 GiB batch {} should exceed 4 GiB batch {}",
        large.batch_size(),
        small.batch_size()
    );
    assert!(small.predicted_peak_bytes() <= VramProfile::Gib4.bytes());
}

#[test]
fn oom_retries_then_fall_back_to_cpu_without_dropping_work() {
    let controller = VramController::new(
        DeviceInventory::gpu(VramProfile::Gib6, VramProfile::Gib6.bytes()).expect("6 GiB"),
        2,
    )
    .expect("controller");
    let planned = controller
        .plan(&base_request(64, 1_048_576))
        .expect("initial plan");
    let recovered = controller
        .recover_from_oom(&planned)
        .expect("OOM is an expected state");
    assert_eq!(recovered.backend(), ComputeBackendKind::CpuF64Reference);
    assert_eq!(
        recovered.fallback(),
        Some(FallbackReason::OutOfMemoryRetryExhausted)
    );
    assert_eq!(recovered.batch_size(), planned.batch_size());
}

fn forbidden_request(
    placement: CorpusPlacement,
    retention: ObservationRetention,
    complexity: ModelComplexity,
    cutoff: CutoffPolicy,
    precision: PrecisionMode,
) -> WorkloadRequest {
    WorkloadRequest::new(
        8, 4, 8, 64, 2, placement, retention, complexity, cutoff, precision,
    )
    .expect("request")
}

#[test]
fn forbidden_memory_adaptations_fail_closed() {
    let controller = VramController::new(
        DeviceInventory::gpu(VramProfile::Gib8, VramProfile::Gib8.bytes()).expect("8 GiB"),
        1,
    )
    .expect("controller");

    assert_eq!(
        controller.plan(&forbidden_request(
            CorpusPlacement::FullCorpusOnDevice,
            ObservationRetention::KeepAll,
            ModelComplexity::KeepSpecified,
            CutoffPolicy::KeepCutoff,
            PrecisionMode::ReferenceF64,
        )),
        Err(ComputeBackendError::FullCorpusTensorRefused)
    );
    assert_eq!(
        controller.plan(&forbidden_request(
            CorpusPlacement::StreamedMicroBatches,
            ObservationRetention::DropToFit,
            ModelComplexity::KeepSpecified,
            CutoffPolicy::KeepCutoff,
            PrecisionMode::ReferenceF64,
        )),
        Err(ComputeBackendError::ObservationDropForbidden)
    );
    assert_eq!(
        controller.plan(&forbidden_request(
            CorpusPlacement::StreamedMicroBatches,
            ObservationRetention::KeepAll,
            ModelComplexity::ReduceToFit,
            CutoffPolicy::KeepCutoff,
            PrecisionMode::ReferenceF64,
        )),
        Err(ComputeBackendError::ComplexityReductionForbidden)
    );
    assert_eq!(
        controller.plan(&forbidden_request(
            CorpusPlacement::StreamedMicroBatches,
            ObservationRetention::KeepAll,
            ModelComplexity::KeepSpecified,
            CutoffPolicy::MoveToFit,
            PrecisionMode::ReferenceF64,
        )),
        Err(ComputeBackendError::CutoffMutationForbidden)
    );
    assert_eq!(
        controller.plan(&forbidden_request(
            CorpusPlacement::StreamedMicroBatches,
            ObservationRetention::KeepAll,
            ModelComplexity::KeepSpecified,
            CutoffPolicy::KeepCutoff,
            PrecisionMode::TransientMixed,
        )),
        Err(ComputeBackendError::UnsupportedPrecision)
    );
}

#[test]
fn telemetry_refuses_raw_source_text() {
    let telemetry = AllocationTelemetry::new(
        1_024,
        256,
        1,
        0,
        PrecisionMode::ReferenceF64,
        Some(FallbackReason::InsufficientVram),
    );
    assert_eq!(
        telemetry.attach_source_text("secret document body"),
        Err(ComputeBackendError::SourceTextInTelemetry)
    );
}
