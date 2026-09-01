"""Regression tests for exact priority-inventory heading recognition."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import validate_documentation as docs


BASE = """# Product and Technical Gap Baseline

**Snapshot:** 2026-09-01T10:49:02Z
**Protected-main evidence:** `1bc02f580cf48e1d39da239f0e818453437c31c3`

## Snapshot facts

| Signal | Snapshot evidence | Delivery implication |
|---|---:|---|
| Open pull requests | **139** | Queue only. |

{heading}

| PR | Exact current head | Draft | Base | Title |
|---:|---|:---:|---|---|
| #441 | `6f483224b3a03e8237c6f4f098a8b0e85e0a91f5` | false | main | repair |

## Operator-gap register

| ID | Closure evidence |
|---|---|
| GAP-001 | Closure evidence exists. |
"""


class PriorityInventoryHeadingTests(unittest.TestCase):
    """Require the canonical level-two heading exactly, not a substring match."""

    def _write(self, heading: str) -> Path:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        path = root / docs.PRODUCT_TECHNICAL_GAP_BASELINE
        path.parent.mkdir(parents=True)
        path.write_text(BASE.format(heading=heading), encoding="utf-8")
        return root

    def test_exact_level_two_heading_passes(self) -> None:
        """The canonical priority-inventory heading is accepted."""

        root = self._write("## Current priority open pull-request evidence")
        docs.validate_product_technical_gap_baseline(root)

    def test_level_three_near_match_fails(self) -> None:
        """A level-three heading cannot masquerade as the canonical inventory."""

        root = self._write("### Current priority open pull-request evidence")
        with self.assertRaisesRegex(AssertionError, "no exact-head rows"):
            docs.validate_product_technical_gap_baseline(root)

    def test_suffixed_level_two_near_match_fails(self) -> None:
        """A suffixed level-two heading is not the canonical inventory authority."""

        root = self._write("## Current priority open pull-request evidence (historical)")
        with self.assertRaisesRegex(AssertionError, "no exact-head rows"):
            docs.validate_product_technical_gap_baseline(root)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
