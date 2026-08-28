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

    def test_unique_branch_fold_overrides_phantom_json_totals(self) -> None:
        """Unique True/False arms, not LLVM totals, are the 100% branch contract.

        Nightly ``files[].summary.branches`` on #49 head ``1e3e2eb`` reported
        ``event_time.rs`` 505/506 while every unique ``files[].branches`` site
        had both arms taken after max-folding the two instantiations. Summary-only
        reports without branch arrays still fail closed on totals.
        """

        with tempfile.TemporaryDirectory() as temporary:
            summary_only = self.write_report(
                temporary,
                {
                    "data": [
                        {
                            "totals": {
                                "lines": {"count": 1, "covered": 1},
                                "branches": {"count": 4, "covered": 3},
                            }
                        }
                    ]
                },
            )
            with self.assertRaisesRegex(ValueError, "incomplete: 3/4"):
                coverage_contract.validate_report(summary_only, ["branches"])

            phantom_totals = self.write_report(
                temporary,
                {
                    "data": [
                        {
                            "totals": {
                                "lines": {"count": 1, "covered": 1},
                                "branches": {"count": 4, "covered": 3},
                            },
                            "files": [
                                {"filename": "crates/psychometric_core/src/causality.rs"},
                                {
                                    "filename": "crates/psychometric_core/src/error.rs",
                                    "branches": [],
                                },
                                {
                                    "filename": "crates/psychometric_core/src/event_time.rs",
                                    "branches": [
                                        [292, 8, 292, 28, 1, 13, 0, 0, 4],
                                        [292, 8, 292, 28, 0, 7, 0, 0, 4],
                                    ],
                                }
                            ],
                        }
                    ]
                },
            )
            self.assertEqual(
                coverage_contract.validate_report(phantom_totals, ["branches"]),
                ["branches coverage: PASS (2/2, 100%)"],
            )

            uncovered_true = self.write_report(
                temporary,
                {
                    "data": [
                        {
                            "totals": {
                                "lines": {"count": 1, "covered": 1},
                                "branches": {"count": 2, "covered": 2},
                            },
                            "files": [
                                {
                                    "filename": "src/lib.rs",
                                    "branches": [
                                        [10, 1, 10, 8, 0, 4, 0, 0, 4],
                                        [10, 1, 10, 8, 0, 2, 0, 0, 4],
                                    ],
                                }
                            ],
                        }
                    ]
                },
            )
            with self.assertRaisesRegex(ValueError, "incomplete: 1/2"):
                coverage_contract.validate_report(uncovered_true, ["branches"])

            uncovered_false = self.write_report(
                temporary,
                {
                    "data": [
                        {
                            "totals": {
                                "lines": {"count": 1, "covered": 1},
                                "branches": {"count": 2, "covered": 2},
                            },
                            "files": [
                                {
                                    "filename": "src/lib.rs",
                                    "branches": [[11, 1, 11, 8, 3, 0, 0, 0, 4]],
                                }
                            ],
                        }
                    ]
                },
            )
            with self.assertRaisesRegex(ValueError, "incomplete: 1/2"):
                coverage_contract.validate_report(uncovered_false, ["branches"])


    def test_live_sqlx_transport_branches_do_not_reenter_unique_fold(self) -> None:
        """sqlx_live.rs arms stay outside the unique-site branch contract.

        cargo llvm-cov already ignores that filename, but the JSON file
        arrays can still carry its live-server success-path arms. The fold
        must drop them so the gate matches the documented transport ignore.
        """

        self.assertTrue(
            coverage_contract.is_live_sqlx_transport_source(
                "/home/runner/work/TEPP/TEPP/crates/persistence_postgres/src/sqlx_live.rs"
            )
        )
        self.assertTrue(coverage_contract.is_live_sqlx_transport_source("sqlx_live.rs"))
        self.assertFalse(
            coverage_contract.is_live_sqlx_transport_source(
                "crates/event_core/src/criterion_posterior.rs"
            )
        )

        with tempfile.TemporaryDirectory() as temporary:
            mixed = self.write_report(
                temporary,
                {
                    "data": [
                        {
                            "totals": {
                                "lines": {"count": 1, "covered": 1},
                                "branches": {"count": 8, "covered": 2},
                            },
                            "files": [
                                {
                                    "filename": (
                                        "crates/persistence_postgres/src/sqlx_live.rs"
                                    ),
                                    "branches": [
                                        [25, 1, 25, 8, 0, 0, 0, 0, 4],
                                        [88, 1, 88, 8, 1, 0, 0, 0, 4],
                                    ],
                                },
                                {
                                    "filename": "crates/event_core/src/criterion_posterior.rs",
                                    "branches": [[108, 1, 108, 8, 2, 3, 0, 0, 4]],
                                },
                            ],
                        }
                    ]
                },
            )
            self.assertEqual(
                coverage_contract.validate_report(mixed, ["branches"]),
                ["branches coverage: PASS (2/2, 100%)"],
            )

            only_live = self.write_report(
                temporary,
                {
                    "data": [
                        {
                            "totals": {
                                "lines": {"count": 1, "covered": 1},
                                "branches": {"count": 2, "covered": 2},
                            },
                            "files": [
                                {
                                    "filename": "sqlx_live.rs",
                                    "branches": [[25, 1, 25, 8, 0, 0, 0, 0, 4]],
                                }
                            ],
                        }
                    ]
                },
            )
            self.assertEqual(
                coverage_contract.validate_report(only_live, ["branches"]),
                ["branches coverage: PASS (2/2, 100%)"],
            )

    def test_malformed_unique_branch_records_fail_closed(self) -> None:
        """Absent filenames, short tuples, and non-integer counts are rejected."""

        totals = {
            "lines": {"count": 1, "covered": 1},
            "branches": {"count": 2, "covered": 2},
        }
        malformed = (
            ([{"branches": [[10, 1, 10, 8, 1, 1, 0, 0, 4]]}], "contain a filename"),
            (
                [{"filename": "src/lib.rs", "branches": "wrong"}],
                "branches must be a list",
            ),
            (
                [{"filename": "src/lib.rs", "branches": [[10, 1, 10, 8, 1]]}],
                "branch record must contain nine values",
            ),
            (
                [{"filename": "src/lib.rs", "branches": [[10, 1, 10, 8, -1, 1, 0, 0, 4]]}],
                "branch counts must be non-negative integers",
            ),
            (
                [{"filename": "src/lib.rs", "branches": [[True, 1, 10, 8, 1, 1, 0, 0, 4]]}],
                "branch coordinates must be integers",
            ),
            (
                [{"filename": "src/lib.rs", "branches": [[10, 1, 10, 8, True, 1, 0, 0, 4]]}],
                "branch counts must be non-negative integers",
            ),
            (["src/lib.rs"], "file entry must be an object"),
        )
        with tempfile.TemporaryDirectory() as temporary:
            for index, (files, message) in enumerate(malformed):
                with self.subTest(message=message):
                    path = Path(temporary) / f"malformed-{index}.json"
                    path.write_text(
                        json.dumps({"data": [{"totals": totals, "files": files}]}),
                        encoding="utf-8",
                    )
                    with self.assertRaisesRegex(ValueError, message):
                        coverage_contract.load_totals(path)

            empty_arrays = self.write_report(
                temporary,
                {
                    "data": [
                        {
                            "totals": totals,
                            "files": [{"filename": "src/lib.rs", "branches": []}],
                        }
                    ]
                },
            )
            self.assertEqual(
                coverage_contract.load_totals(empty_arrays)["branches"],
                totals["branches"],
            )
    def test_full_branch_reports_merge_duplicate_instrumented_copies(self) -> None:
        """A source branch passes when either test binary covers each outcome."""

        payload = self.payload(branch_count=4, branch_covered=2)
        payload["data"][0]["files"] = [  # type: ignore[index]
            {
                "filename": "src/live.rs",
                "branches": [[10, 4, 10, 12, 1, 0, 0, 0, 4]],
            },
            {
                "filename": "src/live.rs",
                "branches": [[10, 4, 10, 12, 0, 1, 0, 0, 4]],
            },
        ]
        with tempfile.TemporaryDirectory() as temporary:
            path = self.write_report(temporary, payload)
            self.assertEqual(
                coverage_contract.load_totals(path)["branches"],
                {"count": 2, "covered": 2},
            )
            self.assertEqual(
                coverage_contract.validate_report(path, ["branches"]),
                ["branches coverage: PASS (2/2, 100%)"],
            )

    def test_full_branch_reports_fail_closed_on_malformed_records(self) -> None:
        """Malformed branch exports cannot weaken the coverage gate."""

        malformed_reports = (
            ([None], "file record must be an object"),
            ([{"filename": "", "branches": []}], "must contain a filename"),
            ([{"filename": "src.rs"}], "must contain branches"),
            ([{"filename": "src.rs", "branches": {}}], "branches must be a list"),
            ([{"filename": "src.rs", "branches": [[1, 2]]}], "record is malformed"),
            (
                [{"filename": "src.rs", "branches": [[True, 2, 3, 4, 1, 0]]}],
                "coordinates are invalid",
            ),
            (
                [{"filename": "src.rs", "branches": [[1.5, 2, 3, 4, 1, 0]]}],
                "coordinates are invalid",
            ),
            (
                [{"filename": "src.rs", "branches": [[1, 2, 3, 4, True, 0]]}],
                "counts are invalid",
            ),
            (
                [{"filename": "src.rs", "branches": [[-1, 2, 3, 4, 1, 0]]}],
                "coordinates are invalid",
            ),
            (
                [{"filename": "src.rs", "branches": [[1, 2, 3, 4, 0.5, 0]]}],
                "counts are invalid",
            ),
            (
                [{"filename": "src.rs", "branches": [[1, 2, 3, 4, -1, 0]]}],
                "counts are invalid",
            ),
        )
        for files, message in malformed_reports:
            with self.subTest(message=message):
                with self.assertRaisesRegex(ValueError, message):
                    coverage_contract.load_union_branch_totals(files)

    def test_union_branch_totals_accumulate_valid_records(self) -> None:
        """Valid records accumulate True/False counts per unique coordinate."""

        files = [
            {
                "filename": "src/live.rs",
                "branches": [[10, 4, 10, 12, 3, 0, 0, 0, 4]],
            },
            {
                # A second instrumented copy of the same coordinate unions its
                # outcomes with the first copy instead of double-counting.
                "filename": "src/live.rs",
                "branches": [[10, 4, 10, 12, 0, 2, 0, 0, 4]],
            },
            {
                # An empty branches array exercises the loop-exhaustion arc.
                "filename": "src/idle.rs",
                "branches": [],
            },
            {
                "filename": "src/other.rs",
                "branches": [[20, 8, 20, 16, 1, 1, 0, 0, 4]],
            },
        ]
        self.assertEqual(
            coverage_contract.load_union_branch_totals(files),
            {"count": 4, "covered": 4},
        )
        self.assertEqual(
            coverage_contract.load_union_branch_totals([]),
            {"count": 0, "covered": 0},
        )

    def test_lcov_authored_line_totals_and_incomplete_detection(self) -> None:
        """LCOV counts unique authored source lines and exposes zero-hit lines."""

        with tempfile.TemporaryDirectory() as temporary:
            complete = self.write_lcov(
                temporary,
                "TN:\nSF:src/lib.rs\nDA:10,2\nDA:11,1,checksum\nend_of_record\n",
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
                "SF:src/lib.rs\nDA:10,1\nDA:11,0\nend_of_record\n",
            )
            with self.assertRaisesRegex(ValueError, "incomplete: 1/2"):
                coverage_contract.validate_report(incomplete, ["lines"], "lcov")

    def test_malformed_lcov_reports_fail_closed(self) -> None:
        """Missing sources, invalid fields, duplicates, and empty reports fail."""

        malformed_reports = (
            ("", "no authored source lines"),
            ("SF:\nDA:1,1\n", "source path must not be empty"),
            ("DA:1,1\n", "must follow a source record"),
            ("SF:src/lib.rs\nDA:1\n", "must contain line and count"),
            ("SF:src/lib.rs\nDA:x,1\n", "must be integers"),
            ("SF:src/lib.rs\nDA:0,1\n", "invalid values"),
            ("SF:src/lib.rs\nDA:1,-1\n", "invalid values"),
            ("SF:src/lib.rs\nDA:1,1\nDA:1,1\n", "duplicate source line"),
            (
                "SF:src/lib.rs\nSF:src/other.rs\nDA:1,1\nend_of_record\n",
                "source record must end with end_of_record",
            ),
            (
                "end_of_record\nSF:src/lib.rs\nDA:1,1\nend_of_record\n",
                "end_of_record must close a source record",
            ),
            (
                "SF:src/lib.rs\nend_of_record\nDA:1,1\n",
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
                "SF:src/lib.rs\nDA:10,1\nend_of_record\n",
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
                "    x + 1,",  # 41 executable expression with trailing comma
                "    record_uncovered(),",  # 42 executable call with trailing comma
                '"standalone string literal",',  # 43 standalone string noise
                "} else {",  # 44 structural branch noise
                ")",  # 45 structural close noise
                '#[cfg(feature = "live-sqlx")]',  # 46 cfg attr
                "fn live_path() {",  # 47 fn
                "    live_body();",  # 48 executable active feature body
                "}",  # 49 brace
                '#[cfg(not(feature = "live-sqlx"))]',  # 50 not-feature attr
                "fn offline_path() {",  # 51 inside not-feature
                "    offline_body();",  # 52 inside not-feature
                "}",  # 53 inside not-feature close
                "#[cfg(test)]",  # 54
                "mod tests {",  # 55 cfg(test) mod
                "    #[test]",  # 56 inside test mod
                "    fn unit() {",  # 57 inside test mod
                "        assert_eq!(1, 1);",  # 58 inside test mod
                "    }",  # 59
                "}",  # 60
                "    executable_statement();",  # 61 executable
                "    append_value(",  # 62 multiline call opener
                "        value,",  # 63 trailing comma noise
                "    );",  # 64 call close
                "    values",  # 65 receiver line stays executable
                "        .iter()",  # 66 method-chain continuation
                "        .collect::<Vec<_>>()",  # 67 method-chain continuation
                "    });",  # 68 closure call close
                "(",  # 69 structural call opener
                ")",  # 70 structural call close
                "    Ok(())",  # 71 structural unit result
                "    NaruonLiveResponse {",  # 72 structural struct literal
                "pub(crate) fn crate_visible() {",  # 73 visibility-qualified fn
                "State::Accepted => {",  # 74 match-arm structure
                "State::Guarded(value) if valid(value) => {",  # 75 guarded arm is executable
                ")?;",  # 76 fallible multiline call close
                ") {",  # 77 multiline condition close
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

            expected_executable = {13, 40, 41, 42, 48, 61, 62, 65, 75}
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
                        "DA:61,1",
                        "DA:62,0",
                        "DA:66,0",
                        "DA:67,0",
                        "DA:1,0",
                        "DA:2,0",
                        "DA:51,0",
                        "end_of_record",
                        "",
                    ]
                ),
            )
            self.assertEqual(
                coverage_contract.load_lcov_line_totals(
                    lcov, repository_root=Path(temporary)
                ),
                {"lines": {"count": 2, "covered": 1}},
            )

    def test_lcov_rejects_source_paths_outside_repository(self) -> None:
        """Untrusted SF paths that escape the repository fail closed."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with tempfile.NamedTemporaryFile(
                mode="w",
                suffix=".rs",
                dir=root.resolve().parent,
                delete=False,
                encoding="utf-8",
            ) as outside_file:
                outside = Path(outside_file.name)
                outside_file.write("fn steal() {}\n")
            try:
                lcov = root / "escape.lcov"
                lcov.write_text(
                    f"SF:{outside}\nDA:1,0\nend_of_record\n",
                    encoding="utf-8",
                )
                with self.assertRaisesRegex(ValueError, "escapes repository root"):
                    coverage_contract.load_lcov_line_totals(
                        lcov, repository_root=root
                    )
                with self.assertRaisesRegex(ValueError, "escapes repository root"):
                    coverage_contract.resolve_repository_source_path(
                        "/etc/passwd", root
                    )
            finally:
                if outside.exists():
                    outside.unlink()

    def test_multiline_guard_arm_is_executable(self) -> None:
        """Retain the final expression of a multiline Rust match guard."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "multiline_guard.rs"
            source.write_text(
                "match state {\n"
                "    State::Ready(value)\n"
                "        if value.is_valid()\n"
                "        && value.is_fresh() => {\n"
                "            consume(value);\n"
                "        }\n"
                "        _ => {\n"
                "            ignore(value);\n"
                "        }\n"
                "}\n",
                encoding="utf-8",
            )

            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 4))
            self.assertFalse(coverage_contract.is_executable_source_line(str(source), 7))

            source.write_text(
                "match state {\n"
                "    State::Ready(value) if(value.is_valid()) => {\n"
                "        consume(value);\n"
                "    }\n"
                "}\n",
                encoding="utf-8",
            )
            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 2))

    def test_guard_after_brace_closing_pattern_is_executable(self) -> None:
        """Count a guard after a destructuring pattern that closes with a brace."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "destructured_guard.rs"
            source.write_text(
                "match state {\n"
                "    State::Ready { value }\n"
                "        if value.is_valid() => {\n"
                "            consume(value);\n"
                "        }\n"
                "}\n",
                encoding="utf-8",
            )

            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 3))

    def test_long_and_nested_match_guards_are_executable(self) -> None:
        """Track guard boundaries beyond the old scan window and nested arms."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "complex_guard.rs"
            long_guard = [
                "match state {",
                "    State::Ready(value)",
                "        if value.is_valid()",
                *[f"        && value.part_{index}()" for index in range(40)],
                "        && value.is_fresh() => {",
                "            consume(value);",
                "        }",
                "}",
            ]
            source.write_text("\n".join(long_guard) + "\n", encoding="utf-8")
            self.assertTrue(
                coverage_contract.is_executable_source_line(
                    str(source), len(long_guard) - 3
                )
            )

            source.write_text(
                "match state {\n"
                "    State::Ready(value)\n"
                "        if match value {\n"
                "            0 => true,\n"
                "            _ => false,\n"
                "        } && value.is_fresh() => {\n"
                "            consume(value);\n"
                "        }\n"
                "}\n",
                encoding="utf-8",
            )
            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 6))

            source.write_text(
                "match state {\n"
                "    State::Ready(value)\n"
                "        if match value {\n"
                "            0 => true,\n"
                "            _ => false,\n"
                "        }\n"
                "        && value.is_fresh() => {\n"
                "            consume(value);\n"
                "        }\n"
                "}\n",
                encoding="utf-8",
            )
            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 7))

    def test_previous_arm_body_does_not_make_next_label_executable(self) -> None:
        """Do not treat an ``if`` inside the preceding arm as a guard."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "previous_arm.rs"
            source.write_text(
                "match state {\n"
                "    State::Previous => {\n"
                "        if value.is_valid() {\n"
                "            consume(value);\n"
                "        }\n"
                "    }\n"
                "    State::Current => {\n"
                "        consume(value);\n"
                "    }\n"
                "}\n",
                encoding="utf-8",
            )

            self.assertFalse(coverage_contract.is_executable_source_line(str(source), 7))

            one_line_previous = Path(temporary) / "one_line_previous.rs"
            one_line_previous.write_text(
                "match state {\n"
                "    State::Previous => value,\n"
                "    State::Current => {\n"
                "        consume(value);\n"
                "    }\n"
                "}\n",
                encoding="utf-8",
            )
            self.assertFalse(
                coverage_contract.is_executable_source_line(
                    str(one_line_previous), 3
                )
            )

            second_guard = Path(temporary) / "second_guard.rs"
            second_guard.write_text(
                "match state {\n"
                "    State::First => value,\n"
                "    State::Ready(value)\n"
                "        if value.is_valid()\n"
                "        && value.is_fresh() => {\n"
                "            consume(value);\n"
                "        }\n"
                "}\n",
                encoding="utf-8",
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(second_guard), 5)
            )

            first_arm = Path(temporary) / "first_arm.rs"
            first_arm.write_text(
                "match state {\n"
                "    State::Current => {\n"
                "        consume(value);\n"
                "    }\n"
                "}\n",
                encoding="utf-8",
            )
            self.assertFalse(
                coverage_contract.is_executable_source_line(str(first_arm), 2)
            )
            self.assertFalse(
                coverage_contract._is_multiline_match_guard(  # noqa: SLF001
                    ["    if value.is_valid() { consume(value); }", "State::Current => {"],
                    2,
                )
            )
            self.assertFalse(
                coverage_contract._is_multiline_match_guard(
                    ["State::Current", "State::Current => {"],
                    2,
                )
            )
            self.assertFalse(
                coverage_contract._is_multiline_match_guard(
                    [
                        "match state {",
                        "    if previous_guard",
                        "    }",
                        "    let nested = match input {",
                        "        0 => {",
                    ],
                    5,
                )
            )

            guarded_after_block = Path(temporary) / "guarded_after_block.rs"
            guarded_after_block.write_text(
                "match state {\n"
                "    State::Previous => {\n"
                "        consume(value);\n"
                "    }\n"
                "    State::Ready(value)\n"
                "        if value.is_valid()\n"
                "        && value.is_fresh() => {\n"
                "        consume(value);\n"
                "    }\n"
                "}\n",
                encoding="utf-8",
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(
                    str(guarded_after_block), 7
                )
            )
            self.assertFalse(
                coverage_contract.is_executable_source_line(str(guarded_after_block), 2)
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
        # Attributes between #[cfg(test)] and the mod declaration belong to
        # the module: detection must survive them (real-world pattern in
        # network_analysis edges/stability test modules).
        with_attribute = [
            "fn production() {}",
            "#[cfg(test)]",
            "#[allow(clippy::float_cmp)]",
            "mod tests {",
            "    #[test]",
            "    fn inner() {}",
            "}",
            "fn after() {}",
        ]
        self.assertEqual(
            coverage_contract._cfg_test_module_line_numbers(with_attribute),
            {4, 5, 6, 7},
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

    def test_multiline_string_continuations_are_not_authored_lines(self) -> None:
        """Rust multiline string fragments are excluded from authored coverage."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "query.rs"
            source.write_text(
                'fn query() {\n'
                '    let sql = format!("SELECT id \\\n'
                '        FROM document_record \\\n'
                '        WHERE tenant_record_id = \'x\'");\n'
                '    execute(sql);\n'
                '}\n',
                encoding="utf-8",
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 2)
            )
            self.assertFalse(
                coverage_contract.is_executable_source_line(str(source), 3)
            )
            self.assertFalse(
                coverage_contract.is_executable_source_line(str(source), 4)
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 5)
            )

    def test_string_scanner_handles_comments_backslash_parity_and_methods(self) -> None:
        """Quoted comments and escaped delimiters do not corrupt source classification."""

        with tempfile.TemporaryDirectory() as temporary:
            backslash = "\\"
            source = Path(temporary) / "scanner.rs"
            source_lines = [
                "fn query() {",
                f'    let sql = "SELECT id {backslash}',
                '        FROM document";',
                r'    // comment contains one " quote',
                "    execute(sql);",
                f'    let even = "ends with two slashes {backslash * 2}";',
                "    execute(even);",
                r'    "literal".to_string();',
                "}",
            ]
            source.write_text("\n".join(source_lines) + "\n", encoding="utf-8")
            path = str(source)

            self.assertFalse(coverage_contract.is_executable_source_line(path, 3))
            self.assertTrue(coverage_contract.is_executable_source_line(path, 5))
            self.assertTrue(coverage_contract.is_executable_source_line(path, 7))
            self.assertTrue(coverage_contract.is_executable_source_line(path, 8))

            block_comment = [
                "/* comment starts",
                r'   comment has a " quote',
                "   still comment",
                "*/",
                "execute();",
            ]
            self.assertFalse(
                coverage_contract._line_in_multiline_string_literal(block_comment, 5)
            )
            nested_block = [
                "/* outer",
                "   /* inner */",
                r'   still outer with " quote',
                "*/",
                "execute();",
            ]
            nested_path = Path(temporary) / "nested.rs"
            nested_path.write_text("\n".join(nested_block) + "\n", encoding="utf-8")
            self.assertFalse(
                coverage_contract._line_in_multiline_string_literal(nested_block, 5)
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(nested_path), 5)
            )
            self.assertTrue(
                coverage_contract._is_standalone_string_literal(r'"escaped\\",')
            )
            self.assertFalse(
                coverage_contract._is_standalone_string_literal('"unfinished')
            )

            raw_string = [
                '    let text = r##"',
                '        a " quote in raw text',
                '    "##;',
                '    execute(text);',
            ]
            self.assertFalse(
                coverage_contract._line_in_multiline_string_literal(raw_string, 1)
            )
            self.assertTrue(
                coverage_contract._line_in_multiline_string_literal(raw_string, 2)
            )
            self.assertTrue(
                coverage_contract._line_in_multiline_string_literal(raw_string, 3)
            )
            self.assertFalse(
                coverage_contract._line_in_multiline_string_literal(raw_string, 4)
            )

            byte_raw_string = [
                '    let bytes = br#"',
                '        raw bytes',
                '    "#;',
            ]
            self.assertTrue(
                coverage_contract._line_in_multiline_string_literal(byte_raw_string, 2)
            )

            closing_with_code = [
                '    let text = "first',
                '    second"; execute(text);',
            ]
            closing_with_comment = [
                '    let text = "first',
                '    second"; // no executable suffix',
            ]
            closing_with_block_comment_and_code = [
                '    let text = "first',
                '    second"; /* note */ execute(text);',
            ]
            self.assertFalse(
                coverage_contract._line_in_multiline_string_literal(closing_with_code, 2)
            )
            self.assertTrue(
                coverage_contract._line_in_multiline_string_literal(closing_with_comment, 2)
            )
            self.assertFalse(
                coverage_contract._line_in_multiline_string_literal(
                    closing_with_block_comment_and_code, 2
                )
            )

            character_and_lifetime = [
                "fn query<'a>() {",
                "    let quote: char = '\"';",
                "    execute();",
                "}",
            ]
            self.assertFalse(
                coverage_contract._line_in_multiline_string_literal(
                    character_and_lifetime, 3
                )
            )
            self.assertIsNone(coverage_contract._character_literal_end("'", 0))
            self.assertEqual(
                coverage_contract._character_literal_end(r"'\''", 0), 4
            )
    def test_line_filter_excludes_literal_and_structural_continuations(self) -> None:
        """LCOV-only literal fragments and branch continuations are filtered."""

        lines = [
            "pub(crate) const fn accessor() -> u8 {",
            "    1",
            "}",
            "    if condition",
            "        || alternate",
            "    } else {",
            "        format!(",
            '            "first ' + "\\",
            '             second"',
            "        );",
            "    ));",
        ]
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "fixture.rs"
            source.write_text("\n".join(lines) + "\n", encoding="utf-8")
            path = str(source)
            self.assertFalse(coverage_contract.is_executable_source_line(path, 1))
            self.assertTrue(coverage_contract.is_executable_source_line(path, 2))
            self.assertFalse(coverage_contract.is_executable_source_line(path, 5))
            self.assertFalse(coverage_contract.is_executable_source_line(path, 6))
            self.assertFalse(coverage_contract.is_executable_source_line(path, 8))
            self.assertFalse(coverage_contract.is_executable_source_line(path, 9))
            self.assertFalse(coverage_contract.is_executable_source_line(path, 11))

    def test_line_filter_keeps_inline_functions_and_block_comment_followers(self) -> None:
        """Inline function bodies and code after quoted block comments stay visible."""

        lines = [
            "pub(crate) const fn enabled(value: u8) -> bool { value > 0 }",
            "pub(crate) fn declaration_only(value: u8) -> bool {",
            "    value > 0",
            "}",
            '/* block comment contains a " quote',
            "   nested /* comment */ still ends here */",
            "    record_after_block_comment();",
        ]
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "inline.rs"
            source.write_text("\n".join(lines) + "\n", encoding="utf-8")
            path = str(source)
            self.assertTrue(coverage_contract.is_executable_source_line(path, 1))
            self.assertFalse(coverage_contract.is_executable_source_line(path, 2))
            self.assertFalse(coverage_contract.is_executable_source_line(path, 5))
            self.assertFalse(coverage_contract.is_executable_source_line(path, 6))
            self.assertTrue(coverage_contract.is_executable_source_line(path, 7))

    def test_multiline_scanner_ignores_comments_char_literals_and_raw_strings(self) -> None:
        """Quote-like text cannot hide executable lines from the authored-line gate."""

        lines = [
            "fn escaped_quotes() {",
            '    let value = source.replace(\'"\', "&quot;"); // a " comment',
            "    executable_after_char_literal();",
            '    let raw = r##"payload " quoted"##;',
            "    raw_continuation_is_data",
            "    let lifetime = 'a; let multiline = r#\"first",
            "second\"#;",
            "    executable_after_raw_string();",
            "}",
        ]
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "quotes.rs"
            source.write_text("\n".join(lines) + "\n", encoding="utf-8")
            path = str(source)
            self.assertTrue(coverage_contract.is_executable_source_line(path, 3))
            self.assertTrue(coverage_contract.is_executable_source_line(path, 8))
            self.assertFalse(coverage_contract.is_executable_source_line(path, 7))

    def test_structural_comma_skips_blank_predecessors(self) -> None:
        """Blank predecessors do not invent a call opener for a trailing comma."""

        lines = [
            "",
            "    ",
            "    field_name,",
        ]
        self.assertFalse(
            coverage_contract._is_structural_comma_continuation(
                lines, 3, "field_name,"
            )
        )
        self.assertFalse(
            coverage_contract._is_structural_comma_continuation(
                ["field_name,"], 1, "field_name,"
            )
        )
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "blank_predecessor.rs"
            source.write_text("\n".join(lines) + "\n", encoding="utf-8")
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 3)
            )

    def test_structural_comma_after_blank_lines_still_sees_call_opener(self) -> None:
        """Empty lines between a call opener and an argument remain structural."""

        lines = [
            "record_value(",
            "",
            "    field_name,",
            ")",
        ]
        self.assertTrue(
            coverage_contract._is_structural_comma_continuation(
                lines, 3, "field_name,"
            )
        )
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "call_opener.rs"
            source.write_text("\n".join(lines) + "\n", encoding="utf-8")
            self.assertFalse(
                coverage_contract.is_executable_source_line(str(source), 3)
            )

    def test_multiline_string_scanner_handles_escaped_char_and_past_eof(self) -> None:
        """Escaped char literals keep later lines classified; past-EOF is closed."""

        lines = [
            "fn query() {",
            r"    let quote = '\'';",
            r"    let slash = '\\';",
            r"    let newline = '\n';",
            "    execute();",
            "}",
        ]
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "escaped_char.rs"
            source.write_text("\n".join(lines) + "\n", encoding="utf-8")
            path = str(source)
            self.assertFalse(coverage_contract._line_in_multiline_string(lines, 2))
            self.assertFalse(coverage_contract._line_in_multiline_string(lines, 5))
            self.assertTrue(coverage_contract.is_executable_source_line(path, 5))
            self.assertFalse(
                coverage_contract._line_in_multiline_string(lines, len(lines) + 1)
            )




    def test_union_branch_totals_merge_counts_across_binaries(self) -> None:
        """Valid records sum per coordinate across duplicate instrumented copies."""

        files = [
            {
                "filename": "src/lib.rs",
                "branches": [
                    [10, 1, 5, 6, 3, 0],
                    [20, 2, 7, 8, 0, 4],
                ],
            },
            {
                "filename": "src/lib.rs",
                "branches": [
                    [10, 1, 5, 6, 1, 2],
                ],
            },
        ]
        self.assertEqual(
            coverage_contract.load_union_branch_totals(files),
            {"count": 4, "covered": 3},
        )

    def test_blank_history_comma_scan_exhaustion(self) -> None:
        """Blank-only preceding lines exhaust the reverse scan and stay unproven."""

        lines = ["", "   ", "    ,"]
        self.assertFalse(
            coverage_contract._is_structural_comma_continuation(lines, 3, ",")
        )

    def test_char_literal_with_escaped_backslash_keeps_scanner_exact(self) -> None:
        """An escaped-backslash char literal cannot flip the multiline verdict."""

        lines = [
            "const slash: char = '\\\\';",
            'static tail: &str = "open',
            '    tail";',
        ]
        self.assertTrue(coverage_contract._line_in_multiline_string(lines, 3))
        self.assertFalse(coverage_contract._line_in_multiline_string(lines, 2))

    def test_multiline_string_empty_lines_returns_false(self) -> None:
        """An empty source produces no multiline-string continuations."""

        self.assertFalse(
            coverage_contract._line_in_multiline_string([], 1)
        )

    def test_structural_comma_continuation_edge_cases(self) -> None:
        """Exercise structural comma continuation detection edge branches."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "commas.rs"
            # A comma-terminated line whose preceding lines are entirely blank
            # exhausts the previous-line scan (arcs 211->215 and 212->211).
            source.write_text("\n\nfoo,\n", encoding="utf-8")
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 3)
            )

            # A blank candidate between the comma line and its previous
            # non-empty line is skipped by the same scan.
            source.write_text("bar(\n\n    baz,\n", encoding="utf-8")
            self.assertFalse(
                coverage_contract.is_executable_source_line(str(source), 3)
            )

    def test_multiline_string_scanner_covers_escaped_char_literals(self) -> None:
        """An escaped character inside a char literal keeps the scanner in loop.

        The backslash inside a character literal must clear through the
        escape-tracking branch so a following quote cannot close the literal
        early; this exercises the scanner's escaped-character arc (318->311).
        """

        lines = ["fn f() {", r"    let newline = '\n';", "}"]
        self.assertFalse(
            coverage_contract._line_in_multiline_string(lines, 3)
        )


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
