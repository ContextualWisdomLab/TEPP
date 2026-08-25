#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! VRAM-budgeted compute planning with a CPU `f64` reference path.
//!
//! This crate plans streamed GPU work under 4/6/8/12/24-GiB profiles and
//! treats out-of-memory as an expected operating state. It does not claim a
//! live accelerator lane. Full document-by-topic tensors are refused, and
//! memory pressure may not drop observations, shrink the model, or move a
//! knowledge cutoff.

mod controller;
mod error;
mod inventory;
mod plan;
mod profile;
mod reference;
mod request;
mod telemetry;

/// VRAM controller that autotunes batches and falls back to CPU.
pub use controller::VramController;
/// Fail-closed compute-backend errors.
pub use error::ComputeBackendError;
/// Typed backend-initialization failure.
pub use error::refuse_uninitialized_backend;
/// Typed device-loss failure.
pub use error::report_device_loss;
/// Typed out-of-memory operating state.
pub use error::report_out_of_memory;
/// Observed accelerator inventory.
pub use inventory::DeviceInventory;
/// Reserved unused device bytes.
pub use inventory::SafetyReserve;
/// Usable bytes after the safety reserve.
pub use inventory::VramBudget;
/// Selected execution backend.
pub use plan::ComputeBackendKind;
/// Why a plan left the accelerator.
pub use plan::FallbackReason;
/// Planned micro-batch.
pub use plan::MicroBatchPlan;
/// Predict peak bytes for a micro-batch.
pub use plan::predicted_peak_bytes;
/// Accepted device-class profile.
pub use profile::VramProfile;
/// Compare a candidate quantity to the CPU `f64` reference.
pub use reference::require_cpu_gpu_parity;
/// Reject a non-finite diagnostic quantity.
pub use reference::require_finite;
/// CPU `f64` streamed weighted sum.
pub use reference::streamed_weighted_sum;
/// Memory-adaptation policies grouped for safe workload construction.
pub use request::AdaptationPolicy;
/// Corpus placement policy.
pub use request::CorpusPlacement;
/// Cutoff-mutation policy.
pub use request::CutoffPolicy;
/// Model-complexity policy.
pub use request::ModelComplexity;
/// Observation-retention policy.
pub use request::ObservationRetention;
/// Transient versus diagnostic precision.
pub use request::PrecisionMode;
/// Streamed workload request.
pub use request::WorkloadRequest;
/// Resource telemetry without source text.
pub use telemetry::AllocationTelemetry;
