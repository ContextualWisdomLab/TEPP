//! OOM recovery must consume only a plan issued for the same workload.
//!
//! A plan carries a batch size and retry counter derived from one request.
//! Recovering it against another request mixes two workloads: the retry is
//! sized from the issuing request while the peak and fallback use the other.

use compute_backend::{
    AdaptationPolicy, ComputeBackendError, ComputeBackendKind, CorpusPlacement, CutoffPolicy,
    DeviceInventory, FallbackReason, ModelComplexity, ObservationRetention, PrecisionMode,
    VramController, VramProfile, WorkloadRequest,
};

fn request(batch: u32, bytes_per_observation: u64) -> WorkloadRequest {
    workload(100, 3, bytes_per_observation, 64, batch)
}

fn workload(
    documents: u64,
    topics: u64,
    bytes_per_observation: u64,
    working_set_bytes: u64,
    batch: u32,
) -> WorkloadRequest {
    WorkloadRequest::new(
        documents,
        topics,
        bytes_per_observation,
        working_set_bytes,
        batch,
        AdaptationPolicy {
            corpus_placement: CorpusPlacement::StreamedMicroBatches,
            observation_retention: ObservationRetention::KeepAll,
            model_complexity: ModelComplexity::KeepSpecified,
            cutoff_policy: CutoffPolicy::KeepCutoff,
        },
        PrecisionMode::ReferenceF64,
    )
    .expect("valid request")
}

fn controller(max_retries: u32) -> VramController {
    let inventory =
        DeviceInventory::gpu(VramProfile::Gib4, VramProfile::Gib4.bytes()).expect("inventory");
    VramController::new(inventory, max_retries).expect("controller")
}

#[test]
fn same_workload_retry_and_fallback_are_unchanged() {
    let c = controller(1);
    let a = request(64, 8);
    let first = c.plan(&a).expect("plan");
    assert_eq!(first.backend(), ComputeBackendKind::GpuStreamed);
    let retry = c.recover_from_oom(&a, &first).expect("retry");
    assert_eq!(retry.backend(), ComputeBackendKind::GpuStreamed);
    assert_eq!(retry.batch_size(), 32);
    assert_eq!(retry.oom_retry_count(), 1);
    let fallback = c.recover_from_oom(&a, &retry).expect("fallback");
    assert_eq!(fallback.backend(), ComputeBackendKind::CpuF64Reference);
    assert_eq!(
        fallback.fallback(),
        Some(FallbackReason::OutOfMemoryRetryExhausted)
    );
    assert_eq!(fallback.batch_size(), a.requested_batch());
}

#[test]
fn equal_valued_workload_is_the_same_workload() {
    let c = controller(1);
    let first = c.plan(&request(64, 8)).expect("plan");
    let retry = c
        .recover_from_oom(&request(64, 8), &first)
        .expect("equal workload");
    assert_eq!(retry.batch_size(), 32);
}

#[test]
fn foreign_plan_with_smaller_cap_is_refused() {
    let c = controller(3);
    let foreign = c.plan(&request(64, 8)).expect("plan for a");
    let result = c.recover_from_oom(&request(2, 8), &foreign);
    assert_eq!(result, Err(ComputeBackendError::PlanWorkloadMismatch));
}

#[test]
fn foreign_plan_is_refused_before_retry_exhaustion_fallback() {
    let c = controller(0);
    let foreign = c.plan(&request(64, 8)).expect("plan for a");
    let result = c.recover_from_oom(&request(2, 8), &foreign);
    assert_eq!(result, Err(ComputeBackendError::PlanWorkloadMismatch));
}

#[test]
fn foreign_plan_differing_only_in_observation_bytes_is_refused() {
    let c = controller(3);
    let foreign = c.plan(&request(64, 8)).expect("plan for a");
    let result = c.recover_from_oom(&request(64, 16), &foreign);
    assert_eq!(result, Err(ComputeBackendError::PlanWorkloadMismatch));
}

#[test]
fn foreign_plan_differing_in_one_other_field_is_refused() {
    let c = controller(3);
    let issued_for = workload(100, 3, 8, 64, 64);
    let foreign = c.plan(&issued_for).expect("plan");
    assert_eq!(foreign.backend(), ComputeBackendKind::GpuStreamed);
    for other in [
        workload(100, 3, 8, 128, 64),
        workload(101, 3, 8, 64, 64),
        workload(100, 4, 8, 64, 64),
    ] {
        assert_eq!(
            c.recover_from_oom(&other, &foreign),
            Err(ComputeBackendError::PlanWorkloadMismatch),
            "{other:?}"
        );
    }
}

#[test]
fn foreign_cpu_plan_is_refused_before_retry_budget() {
    let inventory = DeviceInventory::cpu_only(VramProfile::Gib4);
    let c = VramController::new(inventory, 3).expect("controller");
    let a = request(64, 8);
    let cpu = c.plan(&a).expect("cpu plan");
    assert_eq!(cpu.backend(), ComputeBackendKind::CpuF64Reference);
    assert_eq!(cpu.fallback(), Some(FallbackReason::DeviceUnavailable));
    assert_eq!(
        c.recover_from_oom(&request(2, 8), &cpu),
        Err(ComputeBackendError::PlanWorkloadMismatch)
    );
    assert_eq!(
        c.recover_from_oom(&a, &cpu),
        Err(ComputeBackendError::RetryBudgetExceeded)
    );
}

#[test]
fn invalid_request_is_refused_before_workload_comparison() {
    let c = controller(3);
    let foreign = c.plan(&request(64, 8)).expect("plan");
    let forbidden = WorkloadRequest::new(
        100,
        3,
        8,
        64,
        64,
        AdaptationPolicy {
            corpus_placement: CorpusPlacement::FullCorpusOnDevice,
            observation_retention: ObservationRetention::KeepAll,
            model_complexity: ModelComplexity::KeepSpecified,
            cutoff_policy: CutoffPolicy::KeepCutoff,
        },
        PrecisionMode::ReferenceF64,
    )
    .expect("constructible request");
    assert_eq!(
        c.recover_from_oom(&forbidden, &foreign),
        Err(ComputeBackendError::FullCorpusTensorRefused)
    );
}
