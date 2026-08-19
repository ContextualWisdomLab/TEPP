//! VRAM budget, executable OOM retry, and CPU `f64` reference contracts.
#![allow(clippy::cast_precision_loss)]

use compute_backend::{
    AllocationTelemetry, ComputeBackendError, ComputeBackendKind, CorpusPlacement, CutoffPolicy,
    DeviceInventory, FallbackReason, ModelComplexity, ObservationRetention, PrecisionMode,
    VramController, VramProfile, WorkloadRequest, require_cpu_gpu_parity, streamed_weighted_sum,
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
fn compensated_reference_recovers_cancellation_and_known_total() {
    // CPU-reference evidence only; this crate has no GPU execution path yet.
    let weights = [0.25_f64, 0.25, 0.25, 0.25];
    let values = [4.0_f64, 8.0, 12.0, 16.0];
    let recovered = streamed_weighted_sum(&weights, &values).expect("finite reference");
    let error = rmse(&[10.0], &[recovered]);
    assert!(error < 1e-12, "CPU f64 RMSE {error} exceeded bound");

    let cancellation = streamed_weighted_sum(&[1.0, 1.0, 1.0], &[1e16, 1.0, -1e16])
        .expect("compensated cancellation");
    assert!((cancellation - 1.0).abs() < 1e-15);
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
    assert!(large.batch_size() > small.batch_size());
    assert_eq!(small.oom_retry_count(), 0);
    assert_eq!(large.oom_retry_count(), 0);
}

#[test]
fn each_oom_returns_a_smaller_gpu_plan_before_cpu_fallback() {
    let controller = VramController::new(
        DeviceInventory::gpu(VramProfile::Gib6, VramProfile::Gib6.bytes()).expect("6 GiB"),
        2,
    )
    .expect("controller");
    let request = base_request(64, 1_048_576);
    let initial = controller.plan(&request).expect("initial plan");
    let retry_one = controller
        .recover_from_oom(&request, &initial)
        .expect("first retry plan");
    assert_eq!(retry_one.backend(), ComputeBackendKind::GpuStreamed);
    assert_eq!(retry_one.batch_size(), initial.batch_size() / 2);
    assert_eq!(retry_one.oom_retry_count(), 1);
    assert!(retry_one.predicted_peak_bytes() < initial.predicted_peak_bytes());

    let retry_two = controller
        .recover_from_oom(&request, &retry_one)
        .expect("second retry plan");
    assert_eq!(retry_two.backend(), ComputeBackendKind::GpuStreamed);
    assert_eq!(retry_two.batch_size(), retry_one.batch_size() / 2);
    assert_eq!(retry_two.oom_retry_count(), 2);

    let fallback = controller
        .recover_from_oom(&request, &retry_two)
        .expect("bounded fallback");
    assert_eq!(fallback.backend(), ComputeBackendKind::CpuF64Reference);
    assert_eq!(
        fallback.fallback(),
        Some(FallbackReason::OutOfMemoryRetryExhausted)
    );
    assert_eq!(fallback.batch_size(), request.requested_batch());
    assert_eq!(fallback.oom_retry_count(), 3);
}

#[test]
fn streamed_cardinality_does_not_require_a_hypothetical_full_tensor() {
    let request = WorkloadRequest::new(
        u64::MAX,
        u64::MAX,
        8,
        0,
        1,
        CorpusPlacement::StreamedMicroBatches,
        ObservationRetention::KeepAll,
        ModelComplexity::KeepSpecified,
        CutoffPolicy::KeepCutoff,
        PrecisionMode::ReferenceF64,
    )
    .expect("streamed dimensions are independently representable");
    assert_eq!(request.document_count(), u64::MAX);
    assert_eq!(request.topic_count(), u64::MAX);
}

#[test]
fn parity_rejects_negative_tolerance() {
    assert_eq!(
        require_cpu_gpu_parity(1.0, 1.0, -0.1),
        Err(ComputeBackendError::InvalidTolerance)
    );
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

    for (request, expected) in [
        (
            forbidden_request(
                CorpusPlacement::FullCorpusOnDevice,
                ObservationRetention::KeepAll,
                ModelComplexity::KeepSpecified,
                CutoffPolicy::KeepCutoff,
                PrecisionMode::ReferenceF64,
            ),
            ComputeBackendError::FullCorpusTensorRefused,
        ),
        (
            forbidden_request(
                CorpusPlacement::StreamedMicroBatches,
                ObservationRetention::DropToFit,
                ModelComplexity::KeepSpecified,
                CutoffPolicy::KeepCutoff,
                PrecisionMode::ReferenceF64,
            ),
            ComputeBackendError::ObservationDropForbidden,
        ),
        (
            forbidden_request(
                CorpusPlacement::StreamedMicroBatches,
                ObservationRetention::KeepAll,
                ModelComplexity::ReduceToFit,
                CutoffPolicy::KeepCutoff,
                PrecisionMode::ReferenceF64,
            ),
            ComputeBackendError::ComplexityReductionForbidden,
        ),
        (
            forbidden_request(
                CorpusPlacement::StreamedMicroBatches,
                ObservationRetention::KeepAll,
                ModelComplexity::KeepSpecified,
                CutoffPolicy::MoveToFit,
                PrecisionMode::ReferenceF64,
            ),
            ComputeBackendError::CutoffMutationForbidden,
        ),
        (
            forbidden_request(
                CorpusPlacement::StreamedMicroBatches,
                ObservationRetention::KeepAll,
                ModelComplexity::KeepSpecified,
                CutoffPolicy::KeepCutoff,
                PrecisionMode::TransientMixed,
            ),
            ComputeBackendError::UnsupportedPrecision,
        ),
    ] {
        assert_eq!(controller.plan(&request), Err(expected));
    }
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
