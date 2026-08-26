#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! macOS-native MLX backend receipt probe library.
//!
//! This crate exposes the receipt schema and digest helpers shared between
//! the MLX probe binary and any downstream consumer that needs to verify a
//! hardware-execution receipt without re-deriving its canonical encoding.

/// Schema version for every emitted MLX execution receipt.
pub const RECEIPT_SCHEMA_VERSION: &str = "mlx_native_receipt.v1";

#[cfg(test)]
mod tests {
    #[test]
    fn schema_version_is_stable() {
        assert_eq!(super::RECEIPT_SCHEMA_VERSION, "mlx_native_receipt.v1");
    }
}
