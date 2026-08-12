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
    def write_lcov(directory: str, content: str) -> Path:
        """Write an LCOV report and return its path."""

        path = Path(directory) / "coverage.lcov"
        path.write_text(content, encoding="utf-8")
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

    def test_lcov_authored_line_totals_and_incomplete_detection(self) -> None:
        """LCOV counts unique authored source lines and exposes zero-hit lines."""

        with tempfile.TemporaryDirectory() as temporary:
            complete = self.write_lcov(
                temporary,
                "TN:\nSF:/workspace/src/lib.rs\nDA:10,2\nDA:11,1,checksum\nend_of_record\n",
            )
            self.assertEqual(
                coverage_contract.load_lcov_line_totals(complete),
                {"lines": {"count": 2, "covered": 2}},
            )
            self.assertEqual(
                coverage_contract.validate_report(complete, ["lines"], "lcov"),
                ["lines coverage: PASS (2/2, 100%)"],
            )

            incomplete = self.write_lcov(
                temporary,
                "SF:/workspace/src/lib.rs\nDA:10,1\nDA:11,0\nend_of_record\n",
            )
            with self.assertRaisesRegex(ValueError, "incomplete: 1/2"):
                coverage_contract.validate_report(incomplete, ["lines"], "lcov")

    def test_malformed_lcov_reports_fail_closed(self) -> None:
        """Missing sources, invalid fields, duplicates, and empty reports fail."""

        malformed_reports = (
            ("", "no authored source lines"),
            ("SF:\nDA:1,1\n", "source path must not be empty"),
            ("DA:1,1\n", "must follow a source record"),
            ("SF:/src/lib.rs\nDA:1\n", "must contain line and count"),
            ("SF:/src/lib.rs\nDA:x,1\n", "must be integers"),
            ("SF:/src/lib.rs\nDA:0,1\n", "invalid values"),
            ("SF:/src/lib.rs\nDA:1,-1\n", "invalid values"),
            ("SF:/src/lib.rs\nDA:1,1\nDA:1,1\n", "duplicate source line"),
            (\n                "SF:/src/lib.rs\nSF:/src/other.rs\nDA:1,1\nend_of_record\n",\n                "source record must end with end_of_record",\n            ),\n            (\n                "end_of_record\nSF:/src/lib.rs\nDA:1,1\nend_of_record\n",\n                "end_of_record must close a source record",\n            ),\n            (
                "SF:/src/lib.rs\nend_of_record\nDA:1,1\n",
                "must follow a source record",
            ),
        )
        with tempfile.TemporaryDirectory() as temporary:
            for index, (content, message) in enumerate(malformed_reports):
                with self.subTest(content=content):
                    path = Path(temporary) / f"invalid-{index}.lcov"
                    path.write_text(content, encoding="utf-8")
                    with self.assertRaisesRegex(ValueError, message):
                        coverage_contract.load_lcov_line_totals(path)

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

            lcov_path = self.write_lcov(
                temporary,
                "SF:/workspace/src/lib.rs\nDA:10,1\nend_of_record\n",
            )
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(
                    coverage_contract.main(
                        [str(lcov_path), "--kind", "lines", "--format", "lcov"]
                    ),
                    0,
                )
            with self.assertRaisesRegex(ValueError, "exactly the lines kind"):
                coverage_contract.validate_report(lcov_path, ["branches"], "lcov")
            with self.assertRaisesRegex(ValueError, "unsupported"):
                coverage_contract.validate_report(path, ["lines"], "unknown")

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
            self.assertEqual(namespace.report_format, "json")
            explicit = parser.parse_args(
                [str(path), "--kind", "lines", "--format", "lcov"]
            )
            self.assertEqual(explicit.report_format, "lcov")
            with mock.patch.object(
                sys, "argv", ["checker", str(path), "--kind", "lines"]
            ):
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(coverage_contract.main(None), 0)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
