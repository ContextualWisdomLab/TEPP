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
        self.assertIn("steps.line-report.outcome == 'success'", workflow)
        self.assertIn("id: branch-report", workflow)
        self.assertIn(
            "cargo +nightly-2026-08-01 llvm-cov report --branch --text --show-missing-lines",
            workflow,
        )
        self.assertIn("steps.branch-report.outcome == 'success'", workflow)

    def test_line_gate_measures_authored_source_not_macro_expansions(self) -> None:
        """Generated derive expansion regions cannot dilute authored line coverage."""

        workflow = CI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn(
            "--json --summary-only --skip-expansions --output-path coverage.json",
            workflow,
        )


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
