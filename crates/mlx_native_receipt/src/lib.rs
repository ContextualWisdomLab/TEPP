#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! macOS-native MLX backend receipt probe library.
//!
//! This crate exposes the receipt schema and digest helpers shared between
//! the MLX probe binary and any downstream consumer that needs to verify a
//! hardware-execution receipt without re-deriving its canonical encoding.

use serde::Serialize;
use sha2::{Digest, Sha256};

/// Schema version for every emitted MLX execution receipt.
pub const RECEIPT_SCHEMA_VERSION: &str = "mlx_native_receipt.v1";

/// Canonical JSON payload for one identified MLX execution receipt.
#[derive(Serialize)]
pub struct ProbeReceipt {
    /// Wire schema version tag.
    pub schema_version: &'static str,
    /// Backend that actually executed the objective.
    pub backend_code: &'static str,
    /// Execution environment classification.
    pub execution_environment_code: &'static str,
    /// SHA-256 over the little-endian f32 objective operands.
    pub objective_sha256: String,
    /// SHA-256 over the little-endian f32 result values.
    pub output_sha256: String,
    /// Largest element-wise absolute difference between CPU and GPU results.
    pub observed_maximum_difference: f64,
}

/// Digest a slice of f32 values into their canonical SHA-256 hex encoding.
#[must_use]
pub fn digest(values: &[f32]) -> String {
    let mut hasher = Sha256::new();
    for value in values {
        hasher.update(value.to_le_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_version_is_stable() {
        assert_eq!(super::RECEIPT_SCHEMA_VERSION, "mlx_native_receipt.v1");
    }

    #[test]
    fn digest_is_deterministic_for_known_input() {
        let first = super::digest(&[1.0_f32, 2.0]);
        let second = super::digest(&[1.0_f32, 2.0]);
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }
}
