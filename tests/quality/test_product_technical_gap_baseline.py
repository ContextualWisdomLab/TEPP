"""Contracts for the live product/technical gap baseline."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import validate_documentation as docs


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
BASELINE_PATH = docs.PRODUCT_TECHNICAL_GAP_BASELINE
VALID_SHA = "c45be17a9dbce95ef81cee230e9d128abc7160ac"
VALID_HEAD = "a" * 40


def valid_baseline(*, count: int = 1, extra: str = "") -> str:
    """Return a structurally valid gap-baseline document."""

    return (
        "# Product and Technical Gap Baseline\n\n"
        "**Snapshot:** 2026-08-23T12:27:12Z\n"
        f"**Protected-main evidence:** `{VALID_SHA}`\n\n"
        "## Snapshot facts\n\n"
        "| Signal | Snapshot evidence | Delivery implication |\n"
        "|---|---:|---|\n"
        f"| Open pull requests | **{count}** | Queue only. |\n\n"
        "## Current open pull-request evidence\n\n"
        "| PR | Exact current head | Draft | Base | Title |\n"
        "|---:|---|:---:|---|---|\n"
        f"| #164 | `{VALID_HEAD}` | false | main | docs |\n\n"
        "## Buyer-gap register\n\n"
        "| ID | Closure evidence |\n"
        "|---|---|\n"
        "| GAP-015 | Merge after independent review. |\n"
        f"{extra}"
    )


class ProductTechnicalGapBaselineTests(unittest.TestCase):
    """Require the baseline to be mapped, dated, and honest about queued Checks."""

    def test_baseline_is_required_and_mapped(self) -> None:
        """The canonical map and required-file set both name the live register."""

        self.assertIn(BASELINE_PATH, docs.REQUIRED_FILES)
        self.assertIn(BASELINE_PATH, docs.CANONICAL_LINKS)
        documentation = (REPOSITORY_ROOT / "DOCUMENTATION.md").read_text(
            encoding="utf-8"
        )
        self.assertIn(f"]({BASELINE_PATH})", documentation)

    def test_live_repository_baseline_is_structurally_valid(self) -> None:
        """The committed register carries a dated SHA-bound inventory."""

        docs.validate_required_files(REPOSITORY_ROOT)
        docs.validate_documentation_map(REPOSITORY_ROOT)
        docs.validate_product_technical_gap_baseline(REPOSITORY_ROOT)

    def test_missing_baseline_fails_required_files_and_structure(self) -> None:
        """A tree that omits the register fails both required-file and structure checks."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with self.assertRaisesRegex(AssertionError, BASELINE_PATH):
                docs.validate_required_files(root)
            with self.assertRaisesRegex(AssertionError, BASELINE_PATH):
                docs.validate_product_technical_gap_baseline(root)

    def test_map_without_baseline_link_fails(self) -> None:
        """The documentation map must discover the register by markdown link."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "DOCUMENTATION.md").write_text(
                "[PRD](docs/product/prd-v0.4-approved.md)\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(AssertionError, BASELINE_PATH):
                docs.validate_documentation_map(root)

    def test_valid_fixture_passes_structure_validator(self) -> None:
        """A dated exact-head register with matching count is accepted."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / BASELINE_PATH
            path.parent.mkdir(parents=True)
            path.write_text(valid_baseline(), encoding="utf-8")
            docs.validate_product_technical_gap_baseline(root)

    def test_missing_snapshot_sha_closure_or_inventory_fails(self) -> None:
        """Structure validation refuses an undated or uninventoried register."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / BASELINE_PATH
            path.parent.mkdir(parents=True)
            path.write_text("# Empty gap register\n", encoding="utf-8")
            with self.assertRaisesRegex(AssertionError, "dated UTC snapshot"):
                docs.validate_product_technical_gap_baseline(root)
            with self.assertRaisesRegex(AssertionError, "protected-main SHA"):
                docs.validate_product_technical_gap_baseline(root)
            with self.assertRaisesRegex(AssertionError, "closure evidence"):
                docs.validate_product_technical_gap_baseline(root)
            with self.assertRaisesRegex(AssertionError, "exact-head"):
                docs.validate_product_technical_gap_baseline(root)

    def test_inventory_count_mismatch_fails(self) -> None:
        """Declared open-PR count must match exact-head inventory rows."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / BASELINE_PATH
            path.parent.mkdir(parents=True)
            path.write_text(valid_baseline(count=94), encoding="utf-8")
            with self.assertRaisesRegex(AssertionError, "does not match inventory"):
                docs.validate_product_technical_gap_baseline(root)

    def test_queued_checks_as_implemented_main_fails(self) -> None:
        """Queued Checks must not be promoted to protected-main maturity."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / BASELINE_PATH
            path.parent.mkdir(parents=True)
            path.write_text(
                valid_baseline(
                    extra="\nqueued Checks on this PR are implemented-main.\n"
                ),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(
                AssertionError, "queued Checks as implemented-main"
            ):
                docs.validate_product_technical_gap_baseline(root)

    def test_negated_queued_checks_wording_is_accepted(self) -> None:
        """Correct one-line negation must not be treated as a promotion claim."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / BASELINE_PATH
            path.parent.mkdir(parents=True)
            path.write_text(
                valid_baseline(
                    extra=(
                        "\nPassing or queued Checks on an open PR never "
                        "promote that PR to implemented-main.\n"
                    )
                ),
                encoding="utf-8",
            )
            docs.validate_product_technical_gap_baseline(root)

    def test_wrapped_queued_checks_promotion_still_fails(self) -> None:
        """A wrapped affirmative claim must not evade the promotion guard."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / BASELINE_PATH
            path.parent.mkdir(parents=True)
            path.write_text(
                valid_baseline(
                    extra="\nqueued Checks on this PR are\nimplemented-main.\n"
                ),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(
                AssertionError, "queued Checks as implemented-main"
            ):
                docs.validate_product_technical_gap_baseline(root)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
