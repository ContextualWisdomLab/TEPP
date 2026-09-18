"""Regression contracts for coverage match-arm bodies around Rust comments."""

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

    def test_match_arm_literal_after_inline_line_comment_is_executable(self) -> None:
        """A trailing line comment on the arm label cannot hide its body."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "inline_line_comment.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Commented => { // why this arm exists\n'
                '        "arm body after inline comment"\n'
                '    }\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 3)
            )

    def test_inline_comment_detection_preserves_double_slash_inside_string_pattern(self) -> None:
        """A URL in a string pattern is not mistaken for the trailing comment."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "string_pattern_inline_comment.rs"
            source.write_text(
                'let message = match uri {\n'
                '    "https://example.test" => { // audited URL arm\n'
                '        "url arm body"\n'
                '    }\n'
                '    _ => "other",\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 3)
            )

    def test_unterminated_block_comment_swallows_the_line_beneath(self) -> None:
        """An opener with no closer makes the text under it comment, not code."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "open_ended_comment.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Commented => {\n'
                '        /* an opener with no closer on this line\n'
                '        "arm body after an unterminated opener"\n'
                '    }\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertFalse(coverage_contract.is_executable_source_line(str(source), 4))

    def test_arm_walk_skips_an_unterminated_block_comment_opener(self) -> None:
        """`_is_match_arm_body` walks past an opener that never closes.

        `is_executable_source_line` cannot reach this branch: a line under an
        unterminated opener is comment text and is filtered earlier. The walk
        still has to handle the shape, so it is exercised directly.
        """

        lines = [
            "let message = match self {",
            "    Self::Commented => {",
            "        /* an opener with no closer on this line",
            '        "arm body after an unterminated opener"',
            "    }",
            "};",
        ]
        self.assertTrue(coverage_contract._is_match_arm_body(lines, 4))

    def test_blank_line_between_arm_label_and_literal_body(self) -> None:
        """A blank line is not a code token and must not end the walk."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "blank_line_arm.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Spaced => {\n'
                '\n'
                '        "arm body after a blank line"\n'
                '    }\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 4))

    def test_block_comment_opening_on_its_own_line_is_fully_skipped(self) -> None:
        """A multi-line comment whose opener starts the line leaves no token."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "opener_own_line.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Commented =>\n'
                '        /* the opener owns this line\n'
                '           and the comment closes here */\n'
                '        "arm body after an owned opener",\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 5))

    def test_multiline_block_comment_opened_after_the_arm_label(self) -> None:
        """The arm label survives when it opens the comment that follows it."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "opener_after_label.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Commented => { /* the label opens this note\n'
                '       which closes on the next line */\n'
                '        "arm body after a label-opened note"\n'
                '    }\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 4))

    def test_trailing_block_comment_on_the_arm_label_is_stripped(self) -> None:
        """`=> { /* note */` still ends the arm pattern once the note is cut."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "trailing_block_comment.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Commented => { /* note */\n'
                '        "arm body after a trailing block comment"\n'
                '    }\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 3))

    def test_leading_block_comment_before_arm_label_is_stripped(self) -> None:
        """A leading one-line block comment cannot hide the arm label after it."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "leading_block_comment.rs"
            source.write_text(
                'let message = match self {\n'
                '    /* audited */ Self::Commented => {\n'
                '        "arm body after a leading block comment"\n'
                '    }\n'
                '};\n',
                encoding="utf-8",
            )
            self.assertTrue(coverage_contract.is_executable_source_line(str(source), 3))

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

    def test_lcov_retains_match_arm_literal_after_inline_line_comment(self) -> None:
        """LCOV denominator retains zero-hit bodies after trailing line comments."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "lcov_inline_comment.rs"
            source.write_text(
                'let message = match self {\n'
                '    Self::Commented => { // audited branch\n'
                '        "uncovered arm body"\n'
                '    }\n'
                '};\n'
                'consume(message);\n',
                encoding="utf-8",
            )
            report = root / "coverage.lcov"
            report.write_text(
                f"SF:{source}\nDA:3,0\nDA:6,1\nend_of_record\n",
                encoding="utf-8",
            )
            self.assertEqual(
                coverage_contract.load_lcov_line_totals(report, repository_root=root),
                {"lines": {"count": 2, "covered": 1}},
            )

    def test_lcov_retains_match_arm_literal_after_leading_block_comment(self) -> None:
        """LCOV denominator keeps zero-hit bodies after leading block comments."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "lcov_leading_block_comment.rs"
            source.write_text(
                'let message = match self {\n'
                '    /* audited */ Self::Commented => {\n'
                '        "uncovered arm body"\n'
                '    }\n'
                '};\n'
                'consume(message);\n',
                encoding="utf-8",
            )
            report = root / "coverage.lcov"
            report.write_text(
                f"SF:{source}\nDA:3,0\nDA:6,1\nend_of_record\n",
                encoding="utf-8",
            )
            self.assertEqual(
                coverage_contract.load_lcov_line_totals(report, repository_root=root),
                {"lines": {"count": 2, "covered": 1}},
            )


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
