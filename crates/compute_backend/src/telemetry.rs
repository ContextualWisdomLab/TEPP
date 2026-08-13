//! Allocation telemetry that must not carry source text.

use crate::error::ComputeBackendError;
use crate::plan::FallbackReason;
use crate::request::PrecisionMode;

/// Resource telemetry for one planning or retry decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AllocationTelemetry {
    allocated_bytes: u64,
    transfer_bytes: u64,
    retry_count: u32,
    kernel_launches: u32,
    precision: PrecisionMode,
    fallback: Option<FallbackReason>,
}

impl AllocationTelemetry {
    /// Record allocation, transfer, retry, kernel, precision, and fallback.
    #[must_use]
    pub const fn new(
        allocated_bytes: u64,
        transfer_bytes: u64,
        retry_count: u32,
        kernel_launches: u32,
        precision: PrecisionMode,
        fallback: Option<FallbackReason>,
    ) -> Self {
        Self {
            allocated_bytes,
            transfer_bytes,
            retry_count,
            kernel_launches,
            precision,
            fallback,
        }
    }

    /// Refuse to attach raw source text to telemetry.
    ///
    /// # Errors
    ///
    /// Always returns [`ComputeBackendError::SourceTextInTelemetry`].
    pub fn attach_source_text(&self, _source_text: &str) -> Result<(), ComputeBackendError> {
        let _ = self.allocated_bytes;
        Err(ComputeBackendError::SourceTextInTelemetry)
    }

    /// Return allocated bytes.
    #[must_use]
    pub const fn allocated_bytes(self) -> u64 {
        self.allocated_bytes
    }

    /// Return transfer bytes.
    #[must_use]
    pub const fn transfer_bytes(self) -> u64 {
        self.transfer_bytes
    }

    /// Return OOM retry count.
    #[must_use]
    pub const fn retry_count(self) -> u32 {
        self.retry_count
    }

    /// Return recorded kernel launches.
    #[must_use]
    pub const fn kernel_launches(self) -> u32 {
        self.kernel_launches
    }

    /// Return recorded precision.
    #[must_use]
    pub const fn precision(self) -> PrecisionMode {
        self.precision
    }

    /// Return recorded fallback reason.
    #[must_use]
    pub const fn fallback(self) -> Option<FallbackReason> {
        self.fallback
    }
}

#[cfg(test)]
mod tests {
    use super::AllocationTelemetry;
    use crate::plan::FallbackReason;
    use crate::request::PrecisionMode;

    #[test]
    fn telemetry_accessors_exclude_source_text() {
        let telemetry = AllocationTelemetry::new(
            8,
            4,
            2,
            1,
            PrecisionMode::TransientMixed,
            Some(FallbackReason::DeviceUnavailable),
        );
        assert_eq!(telemetry.allocated_bytes(), 8);
        assert_eq!(telemetry.transfer_bytes(), 4);
        assert_eq!(telemetry.retry_count(), 2);
        assert_eq!(telemetry.kernel_launches(), 1);
        assert_eq!(telemetry.precision(), PrecisionMode::TransientMixed);
        assert_eq!(
            telemetry.fallback(),
            Some(FallbackReason::DeviceUnavailable)
        );
    }
}
