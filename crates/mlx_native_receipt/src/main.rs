#![deny(missing_docs)]
// SAFETY: opaque-handle C FFI calls are confined to the probe body below;
// every mathematical operation remains safe Rust.
#![allow(unsafe_code)]
//! macOS-native MLX backend receipt probe.
//!
//! This binary executes an identified matrix product on MLX, compares it with
//! the Rust CPU reference, and emits a receipt only for the device that
//! actually executed. It is not an Event Lineage estimator receipt.

use mlx_native_receipt::ProbeReceipt;
#[cfg(target_os = "macos")]
use mlx_native_receipt::{RECEIPT_SCHEMA_VERSION, digest};

#[cfg(target_os = "macos")]
fn run() -> Result<ProbeReceipt, Box<dyn std::error::Error>> {
    let lhs = [1.0_f32, 2.0];
    let rhs = [3.0_f32, 4.0];
    let rust = [lhs[0] * rhs[0] + lhs[1] * rhs[1]];
    let output = mlx_cpu_matmul(&lhs, &rhs)?;
    let observed = output
        .iter()
        .zip(&rust)
        .map(|(left, right)| f64::from((left - right).abs()))
        .fold(0.0_f64, f64::max);
    if observed.to_bits() != 0.0_f64.to_bits() {
        return Err("MLX CPU parity failed".into());
    }
    let objective = lhs.iter().chain(&rhs).copied().collect::<Vec<_>>();
    let result = output;
    Ok(ProbeReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        backend_code: "mlx_cpu_macos_native",
        execution_environment_code: "macos_native",
        objective_sha256: digest(&objective),
        output_sha256: digest(&result),
        observed_maximum_difference: observed,
    })
}

#[cfg(target_os = "macos")]
fn mlx_cpu_matmul(lhs: &[f32; 2], rhs: &[f32; 2]) -> Result<Vec<f32>, String> {
    use mlx_sys::{
        mlx_array_data_float32, mlx_array_eval, mlx_array_free, mlx_array_new, mlx_array_new_data,
        mlx_device_free, mlx_device_new_type, mlx_device_type__MLX_CPU, mlx_dtype__MLX_FLOAT32,
        mlx_matmul, mlx_set_default_device, mlx_stream_free, mlx_stream_new_device,
    };
    use std::ffi::c_void;

    // SAFETY: every opaque MLX handle is created by the matching constructor,
    // checked for a successful status before dereference, and freed exactly
    // once after the evaluated scalar is copied into Rust-owned memory.
    unsafe {
        let device = mlx_device_new_type(mlx_device_type__MLX_CPU, 0);
        if mlx_set_default_device(device) != 0 {
            let _ = mlx_device_free(device);
            return Err("failed to select the MLX CPU device".into());
        }
        let stream = mlx_stream_new_device(device);
        let left = mlx_array_new_data(
            lhs.as_ptr().cast::<c_void>(),
            [1_i32, 2].as_ptr(),
            2,
            mlx_dtype__MLX_FLOAT32,
        );
        let right = mlx_array_new_data(
            rhs.as_ptr().cast::<c_void>(),
            [2_i32, 1].as_ptr(),
            2,
            mlx_dtype__MLX_FLOAT32,
        );
        let mut result = mlx_array_new();
        let operation_status = mlx_matmul(&raw mut result, left, right, stream);
        let evaluation_status = if operation_status == 0 {
            mlx_array_eval(result)
        } else {
            operation_status
        };
        let output = if evaluation_status == 0 {
            let pointer = mlx_array_data_float32(result);
            if pointer.is_null() {
                None
            } else {
                Some(vec![*pointer])
            }
        } else {
            None
        };
        let _ = mlx_array_free(result);
        let _ = mlx_array_free(right);
        let _ = mlx_array_free(left);
        let _ = mlx_stream_free(stream);
        let _ = mlx_device_free(device);
        output.ok_or_else(|| "MLX CPU matrix product failed".into())
    }
}

#[cfg(not(target_os = "macos"))]
fn run() -> Result<ProbeReceipt, Box<dyn std::error::Error>> {
    Err("macOS-native MLX receipt unavailable on this host".into())
}

fn emit_receipt(receipt: &ProbeReceipt) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string(receipt)?);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    emit_receipt(&run()?)
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::run;

    #[test]
    fn emits_only_an_exact_macos_native_mlx_cpu_receipt() {
        let receipt = run().expect("installed MLX CPU must execute the probe");
        assert_eq!(receipt.backend_code, "mlx_cpu_macos_native");
        assert_eq!(receipt.execution_environment_code, "macos_native");
        assert_eq!(
            receipt.observed_maximum_difference.to_bits(),
            0.0_f64.to_bits()
        );
        assert_eq!(receipt.objective_sha256.len(), 64);
        assert_eq!(receipt.output_sha256.len(), 64);
    }
}

#[cfg(test)]
#[cfg(not(target_os = "macos"))]
mod tests {
    use super::{emit_receipt, run};
    use mlx_native_receipt::{ProbeReceipt, RECEIPT_SCHEMA_VERSION};

    #[test]
    fn linux_host_refuses_macos_native_mlx_receipt() {
        let error = run().expect_err("linux host must refuse the macOS-native probe");
        assert!(
            error
                .to_string()
                .contains("macOS-native MLX receipt unavailable")
        );
    }

    #[test]
    fn linux_host_emits_a_constructed_receipt_without_running_mlx() {
        let receipt = ProbeReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            backend_code: "mlx_cpu_macos_native",
            execution_environment_code: "macos_native",
            objective_sha256: "a".repeat(64),
            output_sha256: "b".repeat(64),
            observed_maximum_difference: 0.0,
        };
        emit_receipt(&receipt).expect("receipt JSON must serialize");
    }
}
