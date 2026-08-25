//! Fail-closed VRAM and compute-backend errors.

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
            (
                ComputeBackendError::OutOfMemory,
                "device out of memory",
                true,
            ),
            (
                ComputeBackendError::DeviceLoss,
                "compute device lost",
                false,
            ),
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
