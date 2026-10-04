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
    let output = mlx_cpu_matmul(lhs, rhs)?;
    receipt_from_output(lhs, rhs, &output)
}

/// Bind an evaluated MLX output to the exact CPU reference objective.
#[cfg(target_os = "macos")]
fn receipt_from_output(
    lhs: [f32; 2],
    rhs: [f32; 2],
    output: &[f32],
) -> Result<ProbeReceipt, Box<dyn std::error::Error>> {
    let rust = [lhs[0] * rhs[0] + lhs[1] * rhs[1]];
    if output.len() != rust.len() || output.iter().any(|value| !value.is_finite()) {
        return Err("MLX CPU parity failed".into());
    }
    let observed = output
        .iter()
        .zip(&rust)
        .map(|(left, right)| f64::from((left - right).abs()))
        .fold(0.0_f64, f64::max);
    if observed.to_bits() != 0.0_f64.to_bits() {
        return Err("MLX CPU parity failed".into());
    }
    let objective = lhs.iter().chain(&rhs).copied().collect::<Vec<_>>();
    Ok(ProbeReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        backend_code: "mlx_cpu_macos_native",
        execution_environment_code: "macos_native",
        objective_sha256: digest(&objective),
        output_sha256: digest(output),
        observed_maximum_difference: observed,
    })
}

/// Exact native calls whose failure results control probe settlement.
#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct MlxProbeCalls {
    /// Select the native CPU device before allocating array handles.
    select_device: unsafe extern "C" fn(mlx_sys::mlx_device) -> std::ffi::c_int,
    /// Submit the fixed matrix product to the native stream.
    matmul: unsafe extern "C" fn(
        *mut mlx_sys::mlx_array,
        mlx_sys::mlx_array,
        mlx_sys::mlx_array,
        mlx_sys::mlx_stream,
    ) -> std::ffi::c_int,
    /// Materialize the result only after successful submission.
    evaluate: unsafe extern "C" fn(mlx_sys::mlx_array) -> std::ffi::c_int,
    /// Borrow scalar storage only after successful materialization.
    data: unsafe extern "C" fn(mlx_sys::mlx_array) -> *const f32,
    /// Settle one owned input or result array.
    free_array: unsafe extern "C" fn(mlx_sys::mlx_array) -> std::ffi::c_int,
    /// Settle the owned stream after its arrays.
    free_stream: unsafe extern "C" fn(mlx_sys::mlx_stream) -> std::ffi::c_int,
    /// Settle the owned device on success and refusal.
    free_device: unsafe extern "C" fn(mlx_sys::mlx_device) -> std::ffi::c_int,
}

#[cfg(target_os = "macos")]
impl MlxProbeCalls {
    /// Retain the pinned MLX C ABI at the production entry point.
    fn native() -> Self {
        Self {
            select_device: mlx_sys::mlx_set_default_device,
            matmul: mlx_sys::mlx_matmul,
            evaluate: mlx_sys::mlx_array_eval,
            data: mlx_sys::mlx_array_data_float32,
            free_array: mlx_sys::mlx_array_free,
            free_stream: mlx_sys::mlx_stream_free,
            free_device: mlx_sys::mlx_device_free,
        }
    }
}

#[cfg(target_os = "macos")]
fn mlx_cpu_matmul(lhs: [f32; 2], rhs: [f32; 2]) -> Result<Vec<f32>, String> {
    mlx_cpu_matmul_with_calls(&lhs, &rhs, MlxProbeCalls::native())
}

/// Evaluate and settle the same owned native handles on every status path.
#[cfg(target_os = "macos")]
fn mlx_cpu_matmul_with_calls(
    lhs: &[f32; 2],
    rhs: &[f32; 2],
    calls: MlxProbeCalls,
) -> Result<Vec<f32>, String> {
    use mlx_sys::{
        mlx_array_new, mlx_array_new_data, mlx_device_new_type, mlx_device_type__MLX_CPU,
        mlx_dtype__MLX_FLOAT32, mlx_stream_new_device,
    };
    use std::ffi::c_void;

    // SAFETY: every opaque MLX handle is created by the matching constructor,
    // checked for a successful status before dereference, and freed exactly
    // once after the evaluated scalar is copied into Rust-owned memory.
    unsafe {
        let device = mlx_device_new_type(mlx_device_type__MLX_CPU, 0);
        if (calls.select_device)(device) != 0 {
            let _ = (calls.free_device)(device);
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
        let operation_status = (calls.matmul)(&raw mut result, left, right, stream);
        let evaluation_status = if operation_status == 0 {
            (calls.evaluate)(result)
        } else {
            operation_status
        };
        let output = if evaluation_status == 0 {
            let pointer = (calls.data)(result);
            if pointer.is_null() {
                None
            } else {
                Some(vec![*pointer])
            }
        } else {
            None
        };
        let _ = (calls.free_array)(result);
        let _ = (calls.free_array)(right);
        let _ = (calls.free_array)(left);
        let _ = (calls.free_stream)(stream);
        let _ = (calls.free_device)(device);
        output.ok_or_else(|| "MLX CPU matrix product failed".into())
    }
}

#[cfg(not(target_os = "macos"))]
fn run() -> Result<ProbeReceipt, Box<dyn std::error::Error>> {
    Err("macOS-native MLX receipt unavailable on this host".into())
}

fn emit_receipt(receipt: &ProbeReceipt) -> Result<(), Box<dyn std::error::Error>> {
    if !receipt.observed_maximum_difference.is_finite() {
        return Err("observed_maximum_difference must be finite".into());
    }
    println!("{}", serde_json::to_string(receipt)?);
    Ok(())
}

/// Emit a receipt from an already-evaluated probe result.
fn execute_probe_from(
    result: Result<ProbeReceipt, Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    emit_receipt(&result?)
}

/// Run the host probe and emit one receipt, or fail closed.
fn execute_probe() -> Result<(), Box<dyn std::error::Error>> {
    execute_probe_from(run())
}

#[cfg(not(test))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    execute_probe()
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::run;

    /// The pinned native scheduler mutates shared stream vectors without a lock.
    static NATIVE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Serialize native test journeys while retaining normal parallel Rust tests.
    fn native_test_guard() -> std::sync::MutexGuard<'static, ()> {
        NATIVE_TEST_LOCK
            .lock()
            .expect("native test mutex must remain healthy")
    }

    #[test]
    fn refuses_non_finite_backend_output_before_parity_reduction() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let error = super::receipt_from_output([1.0, 2.0], [3.0, 4.0], &[value])
                .expect_err("non-finite backend output must not become a parity receipt");
            assert_eq!(error.to_string(), "MLX CPU parity failed");
        }
    }

    #[test]
    fn refuses_incomplete_backend_output() {
        for output in [&[][..], &[11.0, 11.0][..]] {
            assert!(super::receipt_from_output([1.0, 2.0], [3.0, 4.0], output).is_err());
        }
    }

    #[test]
    fn finite_output_preserves_exact_parity_and_digest() {
        let receipt = super::receipt_from_output([1.0, 2.0], [3.0, 4.0], &[11.0])
            .expect("exact finite scalar must pass");
        assert_eq!(receipt.output_sha256, mlx_native_receipt::digest(&[11.0]));
        assert_eq!(
            receipt.observed_maximum_difference.to_bits(),
            0.0_f64.to_bits()
        );
        assert!(super::receipt_from_output([1.0, 2.0], [3.0, 4.0], &[12.0]).is_err());
    }

    #[test]
    fn rejects_non_finite_receipt_before_json_publication() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let receipt = mlx_native_receipt::ProbeReceipt {
                schema_version: mlx_native_receipt::RECEIPT_SCHEMA_VERSION,
                backend_code: "mlx_cpu_macos_native",
                execution_environment_code: "macos_native",
                objective_sha256: "a".repeat(64),
                output_sha256: "b".repeat(64),
                observed_maximum_difference: value,
            };
            assert_eq!(
                super::emit_receipt(&receipt)
                    .expect_err("non-finite receipt must not publish JSON")
                    .to_string(),
                "observed_maximum_difference must be finite"
            );
        }
    }

    #[test]
    fn device_selection_failure_refuses_a_backend_result() {
        unsafe extern "C" fn refused_device(_device: mlx_sys::mlx_device) -> std::ffi::c_int {
            1
        }
        let _guard = native_test_guard();
        let mut calls = super::MlxProbeCalls::native();
        calls.select_device = refused_device;
        assert_eq!(
            super::mlx_cpu_matmul_with_calls(&[1.0, 2.0], &[3.0, 4.0], calls),
            Err("failed to select the MLX CPU device".to_owned())
        );
    }

    thread_local! {
        static COUNTS: std::cell::Cell<[usize; 5]> = const {
            std::cell::Cell::new([0; 5])
        };
    }

    /// Record one native call within the current test thread.
    fn count(index: usize) {
        COUNTS.with(|counts| {
            let mut value = counts.get();
            value[index] += 1;
            counts.set(value);
        });
    }

    #[test]
    fn native_handles_are_settled_on_every_backend_status_path() {
        unsafe extern "C" fn free_array(array: mlx_sys::mlx_array) -> std::ffi::c_int {
            count(0);
            // SAFETY: the runner owns this native handle and settles it once.
            unsafe { mlx_sys::mlx_array_free(array) }
        }
        unsafe extern "C" fn free_stream(stream: mlx_sys::mlx_stream) -> std::ffi::c_int {
            count(1);
            // SAFETY: the runner owns this native handle and settles it once.
            unsafe { mlx_sys::mlx_stream_free(stream) }
        }
        unsafe extern "C" fn free_device(device: mlx_sys::mlx_device) -> std::ffi::c_int {
            count(2);
            // SAFETY: the runner owns this native handle and settles it once.
            unsafe { mlx_sys::mlx_device_free(device) }
        }
        unsafe extern "C" fn evaluate(array: mlx_sys::mlx_array) -> std::ffi::c_int {
            count(3);
            // SAFETY: this is the live native result supplied by the runner.
            unsafe { mlx_sys::mlx_array_eval(array) }
        }
        unsafe extern "C" fn data(array: mlx_sys::mlx_array) -> *const f32 {
            count(4);
            // SAFETY: the runner evaluated this live array before reading it.
            unsafe { mlx_sys::mlx_array_data_float32(array) }
        }
        unsafe extern "C" fn refused_device(_device: mlx_sys::mlx_device) -> std::ffi::c_int {
            1
        }
        unsafe extern "C" fn refused_matmul(
            _result: *mut mlx_sys::mlx_array,
            _left: mlx_sys::mlx_array,
            _right: mlx_sys::mlx_array,
            _stream: mlx_sys::mlx_stream,
        ) -> std::ffi::c_int {
            1
        }
        unsafe extern "C" fn refused_evaluation(_array: mlx_sys::mlx_array) -> std::ffi::c_int {
            count(3);
            1
        }
        unsafe extern "C" fn null_data(_array: mlx_sys::mlx_array) -> *const f32 {
            count(4);
            std::ptr::null()
        }
        let _guard = native_test_guard();
        let native = super::MlxProbeCalls {
            evaluate,
            data,
            free_array,
            free_stream,
            free_device,
            ..super::MlxProbeCalls::native()
        };
        let cases = [
            (native, Ok(vec![11.0]), [3, 1, 1, 1, 1]),
            (
                super::MlxProbeCalls {
                    select_device: refused_device,
                    ..native
                },
                Err("failed to select the MLX CPU device".to_owned()),
                [0, 0, 1, 0, 0],
            ),
            (
                super::MlxProbeCalls {
                    matmul: refused_matmul,
                    ..native
                },
                Err("MLX CPU matrix product failed".to_owned()),
                [3, 1, 1, 0, 0],
            ),
            (
                super::MlxProbeCalls {
                    evaluate: refused_evaluation,
                    ..native
                },
                Err("MLX CPU matrix product failed".to_owned()),
                [3, 1, 1, 1, 0],
            ),
            (
                super::MlxProbeCalls {
                    data: null_data,
                    ..native
                },
                Err("MLX CPU matrix product failed".to_owned()),
                [3, 1, 1, 1, 1],
            ),
        ];
        for (calls, expected, counts) in cases {
            COUNTS.with(|counter| counter.set([0; 5]));
            assert_eq!(
                super::mlx_cpu_matmul_with_calls(&[1.0, 2.0], &[3.0, 4.0], calls),
                expected
            );
            assert_eq!(COUNTS.with(std::cell::Cell::get), counts);
        }
    }

    #[test]
    fn emits_only_an_exact_macos_native_mlx_cpu_receipt() {
        let _guard = native_test_guard();
        let receipt = run().expect("installed MLX CPU must execute the probe");
        assert_eq!(receipt.backend_code, "mlx_cpu_macos_native");
        assert_eq!(receipt.execution_environment_code, "macos_native");
        assert_eq!(
            receipt.observed_maximum_difference.to_bits(),
            0.0_f64.to_bits()
        );
        assert_eq!(receipt.objective_sha256.len(), 64);
        assert_eq!(receipt.output_sha256.len(), 64);
        super::execute_probe_from(Ok(receipt)).expect("macos receipt must emit");
        super::execute_probe().expect("macos host must emit the probe");
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
        super::execute_probe_from(Ok(receipt)).expect("finite receipt must emit");
    }

    #[test]
    fn linux_host_execute_probe_fails_closed() {
        let error = super::execute_probe().expect_err("linux host must refuse the probe");
        assert!(
            error
                .to_string()
                .contains("macOS-native MLX receipt unavailable")
        );
        let refused = super::execute_probe_from(Err("probe refused".into()));
        assert!(refused.is_err());
    }

    #[test]
    fn linux_host_rejects_non_finite_receipt_json() {
        let receipt = ProbeReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            backend_code: "mlx_cpu_macos_native",
            execution_environment_code: "macos_native",
            objective_sha256: "a".repeat(64),
            output_sha256: "b".repeat(64),
            observed_maximum_difference: f64::NAN,
        };
        emit_receipt(&receipt).expect_err("NaN must fail closed on JSON emit");
        let infinite = ProbeReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            backend_code: "mlx_cpu_macos_native",
            execution_environment_code: "macos_native",
            objective_sha256: "a".repeat(64),
            output_sha256: "b".repeat(64),
            observed_maximum_difference: f64::INFINITY,
        };
        emit_receipt(&infinite).expect_err("Infinity must fail closed on JSON emit");
    }
}
