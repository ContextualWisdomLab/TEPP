"""Tests for exact LLVM coverage report enforcement."""

from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import check_coverage as coverage_contract


class CoverageContractTests(unittest.TestCase):
    """Exercise valid, incomplete, and malformed LLVM coverage reports."""

    @staticmethod
    def write_report(directory: str, payload: object) -> Path:
        """Write *payload* as JSON and return its path."""

        path = Path(directory) / "coverage.json"
        path.write_text(json.dumps(payload), encoding="utf-8")
        return path

    @staticmethod
    def payload(
        *,
        line_count: int = 2,
        line_covered: int = 2,
        branch_count: int = 1,
        branch_covered: int = 1,
    ) -> dict[str, object]:
        """Return a minimal LLVM coverage payload."""

        return {
            "data": [
                {
                    "totals": {
                        "lines": {"count": line_count, "covered": line_covered},
                        "branches": {
                            "count": branch_count,
                            "covered": branch_covered,
                        },
                    }
                }
            ]
        }

    def test_complete_and_zero_denominator_coverage(self) -> None:
        """Exact coverage passes and empty foundation code is explicit."""

        totals = self.payload()["data"][0]["totals"]  # type: ignore[index]
        self.assertEqual(
            coverage_contract.validate_kind(totals, "lines"),  # type: ignore[arg-type]
            "lines coverage: PASS (2/2, 100%)",
        )
        zero_totals = self.payload(
            line_count=0,
            line_covered=0,
            branch_count=0,
            branch_covered=0,
        )["data"][0]["totals"]  # type: ignore[index]
        self.assertIn(
            "0 executable units",
            coverage_contract.validate_kind(zero_totals, "branches"),  # type: ignore[arg-type]
        )

    def test_incomplete_and_malformed_summaries_fail(self) -> None:
        """Missing, nonnumeric, impossible, and incomplete counts are rejected."""

        with self.assertRaisesRegex(ValueError, "do not contain lines"):
            coverage_contract.validate_kind({}, "lines")
        with self.assertRaisesRegex(ValueError, "must be integers"):
            coverage_contract.validate_kind(
                {"lines": {"count": "one", "covered": 1}}, "lines"
            )
        for count, covered in ((-1, 0), (1, -1), (1, 2)):
            with self.subTest(count=count, covered=covered):
                with self.assertRaisesRegex(ValueError, "counts are invalid"):
                    coverage_contract.validate_kind(
                        {"lines": {"count": count, "covered": covered}}, "lines"
                    )
        with self.assertRaisesRegex(ValueError, "incomplete"):
            coverage_contract.validate_kind(
                {"lines": {"count": 2, "covered": 1}}, "lines"
            )
        with self.assertRaisesRegex(ValueError, "do not contain branches"):
            coverage_contract.validate_kind({"branches": []}, "branches")

    def test_report_shape_validation(self) -> None:
        """LLVM JSON must contain one data object with a totals mapping."""

        with tempfile.TemporaryDirectory() as temporary:
            for payload in (
                {},
                {"data": "wrong"},
                {"data": []},
                {"data": [{}, {}]},
            ):
                with self.subTest(payload=payload):
                    path = self.write_report(temporary, payload)
                    with self.assertRaisesRegex(ValueError, "one data entry"):
                        coverage_contract.load_totals(path)
            path = self.write_report(temporary, {"data": [{"totals": []}]})
            with self.assertRaisesRegex(ValueError, "contain totals"):
                coverage_contract.load_totals(path)

    def test_validate_report_and_main(self) -> None:
        """The CLI validates requested kinds and reports stable diagnostics."""

        with tempfile.TemporaryDirectory() as temporary:
            path = self.write_report(temporary, self.payload())
            self.assertEqual(
                coverage_contract.validate_report(path, ["lines", "branches"]),
                [
                    "lines coverage: PASS (2/2, 100%)",
                    "branches coverage: PASS (1/1, 100%)",
                ],
            )
            standard_output = io.StringIO()
            with contextlib.redirect_stdout(standard_output):
                self.assertEqual(
                    coverage_contract.main(
                        [str(path), "--kind", "lines", "--kind", "branches"]
                    ),
                    0,
                )
            self.assertIn("branches coverage", standard_output.getvalue())

            invalid_path = Path(temporary) / "invalid.json"
            invalid_path.write_text("{", encoding="utf-8")
            standard_error = io.StringIO()
            with contextlib.redirect_stderr(standard_error):
                self.assertEqual(
                    coverage_contract.main([str(invalid_path), "--kind", "lines"]),
                    1,
                )
            self.assertIn("FAIL", standard_error.getvalue())

            missing_path = Path(temporary) / "missing.json"
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(
                    coverage_contract.main([str(missing_path), "--kind", "lines"]),
                    1,
                )

            incomplete_path = self.write_report(
                temporary, self.payload(line_covered=1)
            )
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(
                    coverage_contract.main([str(incomplete_path), "--kind", "lines"]),
                    1,
                )

    def test_parser_and_default_argument_source(self) -> None:
        """The parser contract and sys.argv execution path remain usable."""

        parser = coverage_contract.build_parser()
        with tempfile.TemporaryDirectory() as temporary:
            path = self.write_report(temporary, self.payload())
            namespace = parser.parse_args([str(path), "--kind", "lines"])
            self.assertEqual(namespace.report, path)
            self.assertEqual(namespace.kinds, ["lines"])
            with mock.patch.object(
                sys, "argv", ["checker", str(path), "--kind", "lines"]
            ):
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(coverage_contract.main(None), 0)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
