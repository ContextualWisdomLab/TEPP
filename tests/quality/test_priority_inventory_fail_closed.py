"""Fail-closed regressions for the operator priority pull-request inventory."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import validate_documentation as docs


VALID_SHA = "c45be17a9dbce95ef81cee230e9d128abc7160ac"
VALID_HEAD = "a" * 40


def baseline_with_rows(*rows: str) -> str:
    """Build the minimum valid register around caller-supplied priority rows."""

    return (
        "# Product and Technical Gap Baseline\n\n"
        "**Snapshot:** 2026-09-02T00:00:00Z\n"
        f"**Protected-main evidence:** `{VALID_SHA}`\n\n"
        "## Snapshot facts\n\n"
        "| Signal | Snapshot evidence | Delivery implication |\n"
        "|---|---:|---|\n"
        "| Open pull requests | **2** | Queue only. |\n\n"
        "## Current priority open pull-request evidence\n\n"
        "| PR | Exact current head | Draft | Base | Title |\n"
        "|---:|---|:---:|---|---|\n"
        + "".join(rows)
        + "\n## Operator-gap register\n\n"
        "| ID | Closure evidence |\n"
        "|---|---|\n"
        "| GAP-015 | Exact-head evidence required. |\n"
    )


class PriorityInventoryFailClosedTests(unittest.TestCase):
    """Reject malformed rows even when another priority row parses successfully."""

    def test_valid_row_cannot_hide_malformed_head_row(self) -> None:
        """Every data row under the canonical priority heading must parse exactly."""

        fixture = baseline_with_rows(
            f"| #164 | `{VALID_HEAD}` | false | main | valid |\n",
            "| #165 | `not-a-40-character-sha` | false | main | malformed |\n",
        )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / docs.PRODUCT_TECHNICAL_GAP_BASELINE
            path.parent.mkdir(parents=True)
            path.write_text(fixture, encoding="utf-8")
            with self.assertRaisesRegex(
                AssertionError, "malformed priority inventory row"
            ):
                docs.validate_product_technical_gap_baseline(root)

    def test_valid_row_cannot_hide_invalid_draft_or_missing_columns(self) -> None:
        """Invalid enums and truncated Markdown rows are also malformed inventory data."""

        fixture = baseline_with_rows(
            f"| #164 | `{VALID_HEAD}` | false | main | valid |\n",
            f"| #165 | `{VALID_HEAD}` | maybe | main | invalid draft |\n",
            f"| #166 | `{VALID_HEAD}` | true | main |\n",
        )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / docs.PRODUCT_TECHNICAL_GAP_BASELINE
            path.parent.mkdir(parents=True)
            path.write_text(fixture, encoding="utf-8")
            with self.assertRaisesRegex(
                AssertionError, "malformed priority inventory row"
            ):
                docs.validate_product_technical_gap_baseline(root)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
