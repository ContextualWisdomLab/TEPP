//! Integration contract for the `mlx_native_receipt` package identity.

#[test]
fn package_identity_is_stable() {
    let observed = std::hint::black_box(env!("CARGO_PKG_NAME"));
    assert_eq!(observed, "mlx_native_receipt");
}

#[test]
fn packaged_binary_obeys_host_mlx_probe_contract() {
    let exe = env!("CARGO_BIN_EXE_mlx_native_receipt");
    let output = std::process::Command::new(exe)
        .output()
        .expect("spawn mlx_native_receipt");
    let stderr = String::from_utf8_lossy(&output.stderr);
    #[cfg(not(target_os = "macos"))]
    {
        assert!(
            !output.status.success(),
            "linux host must refuse the packaged probe"
        );
        assert!(
            stderr.contains("macOS-native MLX receipt unavailable"),
            "stderr={stderr}"
        );
    }
    #[cfg(target_os = "macos")]
    {
        assert!(
            output.status.success(),
            "macos host must emit the packaged probe stderr={stderr}"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("mlx_cpu_macos_native"), "stdout={stdout}");
    }
}
