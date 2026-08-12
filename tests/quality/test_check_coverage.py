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
            (
                "SF:/src/lib.rs\nSF:/src/other.rs\nDA:1,1\nend_of_record\n",
                "source record must end with end_of_record",
            ),
            (
                "end_of_record\nSF:/src/lib.rs\nDA:1,1\nend_of_record\n",
                "end_of_record must close a source record",
            ),
            (
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

    def test_executable_source_line_filters_noise_records(self) -> None:
        """Non-executable LLVM DA rows are excluded from the authored-line gate."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "sample.rs"
            # Fixed layout so line numbers map to explicit expectations below.
            source_lines = [
                "",  # 1 empty
                "//! crate docs",  # 2 comment
                "use std::io;",  # 3 use
                "pub use crate::inner;",  # 4 pub use
                "mod inner;",  # 5 mod
                "pub mod outer;",  # 6 pub mod
                "impl Foo {",  # 7 impl
                "impl<T> Foo<T> {",  # 8 generic impl noise
                "    /// method docs",  # 9 doc
                "    pub fn bar(",  # 10 pub fn
                "        x: i32,",  # 11 signature arg
                "    ) -> Result<Self, Error> {",  # 12 ) ->
                "        let value = x;",  # 13 executable
                "        Ok(Self { value })",  # 14 Ok(Self
                "    }",  # 15 brace
                "}",  # 16 brace
                "pub struct Foo {",  # 17 pub struct
                "    value: i32,",  # 18 field comma
                "}",  # 19 brace
                "pub enum Kind {",  # 20 pub enum
                "    A,",  # 21 variant comma
                "    B,",  # 22 variant comma
                "}",  # 23 brace
                "fn helper() {}",  # 24 fn
                "struct Private;",  # 25 struct
                "enum Local { X }",  # 26 enum
                "#[derive(Debug)]",  # 27 attr
                "#![allow(dead_code)]",  # 28 inner attr
                "{",  # 29
                "}",  # 30
                "},",  # 31
                ");",  # 32
                "];",  # 33
                "();",  # 34
                "};",  # 35
                "Ok(Self)",  # 36
                ")}",  # 37
                "})",  # 38
                "})",  # 39
                "    return value,",  # 40 executable (return keeps it)
                "    x + 1,",  # 41 trailing comma noise
                '#[cfg(feature = "live-sqlx")]',  # 42 cfg attr
                "fn live_path() {",  # 43 fn
                "    live_body();",  # 44 executable active feature body
                "}",  # 45 brace
                '#[cfg(not(feature = "live-sqlx"))]',  # 46 not-feature attr
                "fn offline_path() {",  # 47 inside not-feature
                "    offline_body();",  # 48 inside not-feature
                "}",  # 49 inside not-feature close
                "#[cfg(test)]",  # 50
                "mod tests {",  # 51 cfg(test) mod
                "    #[test]",  # 52 inside test mod
                "    fn unit() {",  # 53 inside test mod
                "        assert_eq!(1, 1);",  # 54 inside test mod
                "    }",  # 55
                "}",  # 56
                "    executable_statement();",  # 57 executable
            ]
            source.write_text("\n".join(source_lines) + "\n", encoding="utf-8")
            path = str(source)

            self.assertTrue(
                coverage_contract.is_executable_source_line(
                    str(Path(temporary) / "missing.rs"), 1
                )
            )
            self.assertFalse(coverage_contract.is_executable_source_line(path, 0))
            self.assertFalse(
                coverage_contract.is_executable_source_line(path, len(source_lines) + 5)
            )

            expected_executable = {13, 40, 44, 57}
            for line_number in range(1, len(source_lines) + 1):
                is_exec = coverage_contract.is_executable_source_line(path, line_number)
                if line_number in expected_executable:
                    self.assertTrue(
                        is_exec,
                        msg=f"line {line_number} should be executable: {source_lines[line_number - 1]!r}",
                    )
                else:
                    self.assertFalse(
                        is_exec,
                        msg=f"line {line_number} should be filtered: {source_lines[line_number - 1]!r}",
                    )

            lcov = self.write_lcov(
                temporary,
                "\n".join(
                    [
                        f"SF:{path}",
                        "DA:57,1",
                        "DA:1,0",
                        "DA:2,0",
                        "DA:48,0",
                        "end_of_record",
                        "",
                    ]
                ),
            )
            self.assertEqual(
                coverage_contract.load_lcov_line_totals(lcov),
                {"lines": {"count": 1, "covered": 1}},
            )

    def test_cfg_test_and_not_feature_block_helpers(self) -> None:
        """cfg(test) modules and cfg(not(feature)) blocks are fully recognized."""

        lines = [
            "fn prod() {}",
            "#[cfg(test)]",
            "",
            "mod tests {",
            "    fn inner() {}",
            "}",
            '#[cfg(not(feature = "x"))]',
            "fn alternate() {",
            "    body();",
            "}",
            "fn after() {}",
            "#[cfg(test)]",
            "fn not_a_module() {}",
            "#[cfg(test)]",
        ]
        test_lines = coverage_contract._cfg_test_module_line_numbers(lines)
        self.assertEqual(test_lines, {4, 5, 6})
        self.assertTrue(coverage_contract._line_in_cfg_not_feature_block(lines, 8))
        self.assertTrue(coverage_contract._line_in_cfg_not_feature_block(lines, 9))
        self.assertFalse(coverage_contract._line_in_cfg_not_feature_block(lines, 1))
        self.assertFalse(coverage_contract._line_in_cfg_not_feature_block(lines, 11))
        open_only = [
            '#[cfg(not(feature = "x"))]',
            "fn unfinished()",
        ]
        self.assertTrue(
            coverage_contract._line_in_cfg_not_feature_block(open_only, 2)
        )
        self.assertEqual(coverage_contract._cfg_test_module_line_numbers([]), set())
        # Nested braces inside cfg(test) mod must fully close before exit.
        nested = [
            "#[cfg(test)]",
            "mod tests {",
            "    fn nested() {",
            "        let x = 1;",
            "    }",
            "}",
            "fn production() {}",
        ]
        self.assertEqual(
            coverage_contract._cfg_test_module_line_numbers(nested),
            {2, 3, 4, 5, 6},
        )
        # Unclosed modules/blocks exit by EOF without a balanced close.
        unclosed_mod = [
            "#[cfg(test)]",
            "mod tests {",
            "    fn dangling() {}",
        ]
        self.assertEqual(
            coverage_contract._cfg_test_module_line_numbers(unclosed_mod),
            {2, 3},
        )
        unclosed_not_feature = [
            '#[cfg(not(feature = "x"))]',
            "fn dangling() {",
            "    body();",
        ]
        self.assertTrue(
            coverage_contract._line_in_cfg_not_feature_block(unclosed_not_feature, 2)
        )
        self.assertTrue(
            coverage_contract._line_in_cfg_not_feature_block(unclosed_not_feature, 3)
        )
        self.assertFalse(
            coverage_contract._line_in_cfg_not_feature_block(unclosed_not_feature, 99)
        )


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
