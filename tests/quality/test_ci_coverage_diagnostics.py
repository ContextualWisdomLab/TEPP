"""Regression tests for exact Rust coverage diagnostics in CI."""

from __future__ import annotations

import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
CI_WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "ci.yml"


class CoverageDiagnosticsContractTests(unittest.TestCase):
    """Keep failed 100% gates actionable without weakening them."""

    def test_line_and_branch_failures_print_exact_missing_locations(self) -> None:
        """Both LLVM coverage lanes retain same-run missing-location reports."""

        workflow = CI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("id: line-report", workflow)
        self.assertIn("cargo llvm-cov report --text --show-missing-lines", workflow)
        self.assertIn("--show-instantiations", workflow)
        self.assertIn(
            'LLVM_COV_FLAGS="--show-line-counts-or-regions" cargo llvm-cov report',
            workflow,
        )
        self.assertIn("steps.line-report.outcome == 'success'", workflow)
        self.assertIn("id: branch-report", workflow)
        self.assertIn(
            "cargo +nightly-2026-08-21 llvm-cov report --branch --text --show-missing-lines",
            workflow,
        )
        self.assertIn("steps.branch-report.outcome == 'success'", workflow)

    def test_line_gate_uses_lcov_authored_lines_and_keeps_region_evidence(self) -> None:
        """Authored lines gate on LCOV while full JSON retains hidden-region evidence."""

        workflow = CI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn(
            "cargo llvm-cov --workspace --all-features --json --output-path coverage.json",
            workflow,
        )
        self.assertNotIn(
            "cargo llvm-cov --workspace --all-features --json --summary-only",
            workflow,
        )
        self.assertIn(
            "cargo llvm-cov report --lcov --output-path coverage.lcov",
            workflow,
        )
        self.assertIn(
            "python3 scripts/check_coverage.py coverage.lcov --kind lines --format lcov",
            workflow,
        )
        self.assertNotIn(
            "python3 scripts/check_coverage.py coverage.json --kind lines",
            workflow,
        )
        self.assertIn("UNCOVERED_REGION", workflow)
        self.assertIn("UNCOVERED_FUNCTION", workflow)

    def test_live_postgres_job_is_gated_and_service_backed(self) -> None:
        """Live SQLx evidence requires a Postgres service and explicit env gate."""

        workflow = CI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("live-postgres:", workflow)
        self.assertIn("name: Live PostgreSQL integration", workflow)
        self.assertIn("image: postgres:16.9-alpine", workflow)
        self.assertIn('TEPP_LIVE_POSTGRES: "1"', workflow)
        self.assertIn("DATABASE_URL: postgres://tepp:tepp_ci@localhost:5432/tepp", workflow)
        self.assertIn(
            "cargo test -p persistence_postgres --features live-sqlx --test live_postgres",
            workflow,
        )
        self.assertIn("migrations/**", workflow)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
