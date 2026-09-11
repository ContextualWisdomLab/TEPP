"""Regression contract for data-only raw-string opening lines in LCOV classification."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import check_coverage as coverage_contract


class LcovRawOpeningContractTests(unittest.TestCase):
    """Keep raw-string data openers non-authored without hiding executable prefixes."""

    def test_data_only_raw_opener_is_filtered(self) -> None:
        """A whitespace-prefixed raw literal opener is data, not executable Rust."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "raw_argument.rs"
            source.write_text(
                "fn query() {\n"
                "    consume(\n"
                '        r##"\n'
                "        SELECT tenant_record_id\n"
                '        "##,\n'
                "    );\n"
                "}\n",
                encoding="utf-8",
            )

            self.assertFalse(
                coverage_contract.is_executable_source_line(str(source), 3),
                "the raw-string opener has no executable prefix on its source line",
            )

    def test_executable_prefix_before_raw_opener_remains_authored(self) -> None:
        """An assignment before a raw literal opener remains coverage-authoritative."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "raw_assignment.rs"
            source.write_text(
                "fn query() {\n"
                '    let text = r##"first\n'
                "        second\n"
                '        "##;\n'
                "    consume(text);\n"
                "}\n",
                encoding="utf-8",
            )

            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 2),
                "the assignment prefix is executable even though the literal continues",
            )


if __name__ == "__main__":
    unittest.main()
