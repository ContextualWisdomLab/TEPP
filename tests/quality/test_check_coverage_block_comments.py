"""Regression contracts for coverage match-arm bodies after Rust block comments."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import check_coverage as coverage_contract


class CoverageBlockCommentRegressionTests(unittest.TestCase):
    """Keep executable match-arm literals in the authored-line denominator."""

    def test_match_arm_literal_after_single_line_block_comment_is_executable(self) -> None:
        """A one-line block comment cannot hide the arm's literal body."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "single_line_block_comment.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Commented => {\n'
                '        /* why this arm exists */\n'
                '        "arm body after block comment"\n'
                '    }\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 4)
            )

    def test_match_arm_literal_after_multiline_block_comment_is_executable(self) -> None:
        """A multi-line block comment cannot become the preceding code token."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "multiline_block_comment.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Commented => {\n'
                '        /* why this arm\n'
                '           spans another line\n'
                '           exists */\n'
                '        "arm body after multiline block comment"\n'
                '    }\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 6)
            )

    def test_lcov_retains_match_arm_literal_after_block_comment(self) -> None:
        """LCOV denominator keeps the literal even when LLVM reports zero hits."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "lcov_block_comment.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Commented => {\n'
                '        /* audited branch */\n'
                '        "uncovered arm body"\n'
                '    }\n'
                '};\n'
                'consume(message);\n',
                encoding="utf-8",
            )
            report = root / "coverage.lcov"
            report.write_text(
                f"SF:{source}\nDA:4,0\nDA:7,1\nend_of_record\n",
                encoding="utf-8",
            )
            self.assertEqual(
                coverage_contract.load_lcov_line_totals(report, repository_root=root),
                {"lines": {"count": 2, "covered": 1}},
            )


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
