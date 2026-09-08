"""Regression contract for authored LCOV lines after multiline data closes."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import check_coverage as coverage_contract


class LcovMultilineSuffixContractTests(unittest.TestCase):
    """Keep executable suffixes in the authored-line denominator."""

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


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
