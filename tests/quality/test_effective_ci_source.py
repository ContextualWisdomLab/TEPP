"""Offline source custody and substantive regression controls for centralized CI."""

from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import effective_ci_source as source
from scripts import check_workspace_contract as workspace
import test_ci_coverage_diagnostics as diagnostics
import test_workflow_queue_hygiene as queue

ROOT = Path(__file__).resolve().parents[2]


class EffectiveCiSourceTests(unittest.TestCase):
    """Keep source identity, inherited gates, and runner isolation fail closed."""

    def copy_evidence(self, root: Path) -> Path:
        """Create an offline checkout containing only caller and frozen evidence."""
        (root / ".github/workflows").mkdir(parents=True)
        shutil.copyfile(ROOT / ".github/workflows/ci.yml", root / ".github/workflows/ci.yml")
        fixture = root / source.FIXTURE_PATH
        shutil.copytree(ROOT / source.FIXTURE_PATH, fixture)
        return fixture

    def test_offline_source_is_complete_and_isolated(self) -> None:
        """The original Git objects resolve without Git metadata or network access."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.copy_evidence(root)
            text = source.effective_ci_source(root)
        self.assertIn(f"@{source.SUPPLIER_PIN}", text)
        self.assertEqual(text.count("      group: CWL CI isolated"), 4)
        self.assertEqual(text.count("      labels: [self-hosted, linux, x64, cwlab-ci-isolated]"), 4)
        self.assertIn("coverage report --fail-under=100", text)
        self.assertIn("coverage-branches.json --kind branches", text)
        self.assertIn('CARGO_NEXTEST_VERSION: "0.9.140"', text)
        self.assertIn('CARGO_DENY_VERSION: "0.19.7"', text)
        self.assertIn('CARGO_LLVM_COV_VERSION: "0.8.6"', text)

    def test_every_evidence_object_rejects_corruption_and_absence(self) -> None:
        """No missing or tampered commit, intermediate tree, or workflow is accepted."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = self.copy_evidence(root)
            for filename in ("commit.object", "root.tree", ".github.tree", "workflows.tree", "tepp-rust-foundation-ci.yml"):
                with self.subTest(filename=filename):
                    path = fixture / filename
                    original = path.read_bytes()
                    path.write_bytes(original + b"corrupt")
                    with self.assertRaisesRegex(ValueError, "Git object mismatch"):
                        source.effective_ci_source(root)
                    path.unlink()
                    with self.assertRaises(OSError):
                        source.effective_ci_source(root)
                    path.write_bytes(original)
            self.assertEqual(source.effective_ci_source(root), source.effective_ci_source(ROOT))

    def test_caller_identity_rejects_pin_dispatch_and_job_drift(self) -> None:
        """Valid-looking replacement pins, disabled calls, and extra jobs fail closed."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.copy_evidence(root)
            path = root / ".github/workflows/ci.yml"
            original = path.read_text(encoding="utf-8")
            variants = (
                original.replace(source.SUPPLIER_PIN, "a" * 40),
                original.replace("  foundation:", "  foundation:\n    if: false"),
                original.replace("contents: read", "contents: write"),
                original.replace("cancel-in-progress:", "removed-concurrency:"),
                original + "\n  competing-job:\n    runs-on: ubuntu-latest\n",
            )
            for text in variants:
                path.write_text(text, encoding="utf-8")
                with self.assertRaisesRegex(ValueError, "caller differs"):
                    source.effective_ci_source(root)
                self.assertTrue(any("supplier source validation failed" in error for error in workspace._validate_ci_contract(root)))

    def test_supplier_weakenings_are_rejected_by_custody(self) -> None:
        """Coverage, compiler, tool, PostgreSQL and isolation mutations reject offline."""
        changes = (
            ("--fail-under=100", "--fail-under=99"),
            ("coverage.lcov --kind lines --format lcov", "coverage.lcov --kind removed --format lcov"),
            ("coverage-branches.json --kind branches", "coverage-branches.json --kind removed"),
            ("nightly-2026-08-21", "nightly"),
            ("rustup toolchain install 1.98.0", "rustup toolchain install stable"),
            ('CARGO_LLVM_COV_VERSION: "0.8.6"', 'CARGO_LLVM_COV_VERSION: "latest"'),
            ("postgres:16.9-alpine", "postgres:latest"),
            ('TEPP_LIVE_POSTGRES: "1"', 'TEPP_LIVE_POSTGRES: "0"'),
            ("--features live-sqlx --test live_postgres", "--features live-sqlx --test removed"),
            ("CWL CI isolated", "CWL CI control"),
            ("cwlab-ci-isolated", "privileged"),
            ("x64,", ""),
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = self.copy_evidence(root)
            path = fixture / "tepp-rust-foundation-ci.yml"
            original = path.read_text(encoding="utf-8")
            for before, after in changes:
                with self.subTest(mutation=before):
                    self.assertIn(before, original)
                    path.write_text(original.replace(before, after), encoding="utf-8")
                    with self.assertRaisesRegex(ValueError, "blob Git object mismatch"):
                        source.effective_ci_source(root)
            path.write_text(original, encoding="utf-8")
            self.assertEqual(workspace._validate_ci_contract(root), ["rust-toolchain.toml is missing", "deny.toml is missing"])

    def test_original_assertions_reject_weakened_effective_bodies(self) -> None:
        """Exercise inherited checks themselves, independently of custody rejection."""
        original = source.effective_ci_source(ROOT)
        controls = (
            (diagnostics, diagnostics.CoverageDiagnosticsContractTests, "test_line_and_branch_failures_print_exact_missing_locations", "id: line-report", "id: removed"),
            (diagnostics, diagnostics.CoverageDiagnosticsContractTests, "test_line_and_branch_failures_print_exact_missing_locations", "id: branch-report", "id: removed"),
            (diagnostics, diagnostics.CoverageDiagnosticsContractTests, "test_line_gate_uses_lcov_authored_lines_and_keeps_region_evidence", "coverage.lcov --kind lines --format lcov", "coverage.lcov --kind removed"),
            (diagnostics, diagnostics.CoverageDiagnosticsContractTests, "test_live_postgres_job_is_gated_and_service_backed", "postgres:16.9-alpine", "postgres:latest"),
            (diagnostics, diagnostics.CoverageDiagnosticsContractTests, "test_live_postgres_job_is_gated_and_service_backed", 'TEPP_LIVE_POSTGRES: "1"', 'TEPP_LIVE_POSTGRES: "0"'),
            (diagnostics, diagnostics.CoverageDiagnosticsContractTests, "test_rust_toolchains_are_reproducible_and_reviewably_updated", "nightly-2026-08-21", "nightly"),
            (queue, queue.WorkflowQueueHygieneTests, "test_rust_ci_reuses_one_runner_for_both_coverage_reports", "  production-coverage:", "  line-coverage:"),
        )
        for module, case, method, before, after in controls:
            with self.subTest(method=method, mutation=before):
                self.assertIn(before, original)
                with mock.patch.object(module, "effective_ci_source", return_value=original.replace(before, after)):
                    with self.assertRaises(AssertionError):
                        getattr(case(method), method)()
        for command in workspace.REQUIRED_CI_SNIPPETS:
            with self.subTest(command=command):
                self.assertIn(command, original)
                with mock.patch.object(workspace, "effective_ci_source", return_value=original.replace(command, "REMOVED")):
                    self.assertIn(f"CI workflow is missing required command: {command}", workspace._validate_ci_contract(ROOT))

    def test_independent_sha256_and_tree_path_checks(self) -> None:
        """SHA-256 is independently enforced and tree lookup requires exact names."""
        with mock.patch.object(source, "SUPPLIER_SHA256", "0" * 64):
            with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
                source.effective_ci_source(ROOT)
        with self.assertRaises(ValueError):
            source._tree_child(b"", "40000", ".github")
