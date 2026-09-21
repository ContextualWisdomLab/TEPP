"""Regression contracts for authored LCOV multiline-data classification."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import check_coverage as coverage_contract


class LcovMultilineSuffixContractTests(unittest.TestCase):
    """Keep executable suffixes visible without rescanning one source per DA row."""

    def test_string_closing_line_with_code_remains_authored(self) -> None:
        """Code after a multiline string close must remain coverage-authoritative."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "string_suffix.rs"
            source.write_text(
                'fn query() {\n'
                '    let text = "first\n'
                '    second"; execute(text);\n'
                '}\n',
                encoding="utf-8",
            )

            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 3),
                "the closing line performs execute(text) after the literal closes",
            )

    def test_block_comment_closing_line_with_code_remains_authored(self) -> None:
        """Code after a multiline block-comment close must remain authored."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "comment_suffix.rs"
            source.write_text(
                "fn query() {\n"
                "    /* explanation starts\n"
                "       still explanation */ execute();\n"
                "}\n",
                encoding="utf-8",
            )

            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 3),
                "the closing comment line performs execute() after */",
            )

    def test_multiline_classification_does_not_repeat_for_da_rows(self) -> None:
        """One source-wide multiline scan must serve every LCOV DA row."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "sample.rs"
            source.write_text(
                "fn sample() {\n"
                "    first();\n"
                "    second();\n"
                "    third();\n"
                "}\n",
                encoding="utf-8",
            )
            report = root / "coverage.lcov"
            report.write_text(
                f"SF:{source}\n"
                "DA:2,1\n"
                "DA:3,1\n"
                "DA:4,1\n"
                "end_of_record\n",
                encoding="utf-8",
            )

            original_scan = coverage_contract._line_in_multiline_string_literal
            with mock.patch.object(
                coverage_contract,
                "_line_in_multiline_string_literal",
                wraps=original_scan,
            ) as scan:
                self.assertEqual(
                    coverage_contract.load_lcov_line_totals(report, repository_root=root),
                    {"lines": {"count": 3, "covered": 3}},
                )

            self.assertLessEqual(
                scan.call_count,
                1,
                "multiline classification must not rescan a source for each DA row",
            )


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
