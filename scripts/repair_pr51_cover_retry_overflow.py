"""Add the final OOM retry overflow coverage regression to PR 51 repair source."""

from pathlib import Path

path = Path("scripts/repair_pr51_apply_recovery.py")
text = path.read_text(encoding="utf-8")
marker = "    #[test]\n    fn oom_recovery_emits_retries_then_falls_back() {\n"
insertion = r'''    #[test]
    fn overflowing_oom_retry_peak_fails_closed() {
        let inventory =
            DeviceInventory::gpu(VramProfile::Gib24, VramProfile::Gib24.bytes()).expect("24");
        let controller = VramController::new(inventory, 1).expect("controller");
        let huge = WorkloadRequest::new(
            1,
            1,
            u64::MAX,
            u64::MAX,
            2,
            CorpusPlacement::StreamedMicroBatches,
            ObservationRetention::KeepAll,
            ModelComplexity::KeepSpecified,
            CutoffPolicy::KeepCutoff,
            PrecisionMode::ReferenceF64,
        )
        .expect("request");
        let initial = MicroBatchPlan::new(
            ComputeBackendKind::GpuStreamed,
            2,
            0,
            PrecisionMode::ReferenceF64,
            0,
            None,
        );
        assert_eq!(
            controller.recover_from_oom(&huge, &initial),
            Err(ComputeBackendError::InvalidBudget)
        );
    }

'''
if insertion in text:
    raise SystemExit(0)
if text.count(marker) != 1:
    raise SystemExit("expected one OOM recovery test marker")
path.write_text(text.replace(marker, insertion + marker, 1), encoding="utf-8")
