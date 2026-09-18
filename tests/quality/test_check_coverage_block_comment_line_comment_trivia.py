"""Regression contracts for mixed Rust block/line-comment trivia."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import check_coverage as coverage_contract


class BlockCommentLineCommentTriviaTests(unittest.TestCase):
    """Keep mixed comment-only rows transparent without hiding authored code."""

    def test_mixed_comment_only_row_does_not_break_opener_reconciliation(self) -> None:
        """`/* ... */ // ...` is trivia between a proven opener and its body."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "mixed_comment_gap.rs"
            source.write_text(
                "fn run(ready: bool) {\n"
                "    if ready {\n"
                "        /* audited */ // explanatory note\n"
                "        do_work();\n"
                "    }\n"
                "}\n",
                encoding="utf-8",
            )
            report = root / "coverage.lcov"
            report.write_text(
                f"SF:{source}\nDA:2,0\nDA:4,3\nend_of_record\n",
                encoding="utf-8",
            )

            self.assertEqual(
                coverage_contract.load_lcov_line_totals(report, repository_root=root),
                {"lines": {"count": 2, "covered": 2}},
            )

    def test_mixed_comment_only_row_does_not_hide_match_arm_body(self) -> None:
        """The arm walk skips a complete block comment followed by a line comment."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "mixed_comment_arm.rs"
            source.write_text(
                "fn label(value: i32) -> &'static str {\n"
                "    match value {\n"
                "        _ => {\n"
                "            /* audited */ // explanatory note\n"
                "            \"fallback\"\n"
                "        }\n"
                "    }\n"
                "}\n",
                encoding="utf-8",
            )

            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 5, root)
            )

    def test_code_after_block_comment_closer_is_not_treated_as_trivia(self) -> None:
        """A real suffix after `*/` remains authored even with later `//` commentary."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "comment_then_code.rs"
            source.write_text(
                "fn run() {\n"
                "    /* audited */ do_work(); // still executable\n"
                "}\n",
                encoding="utf-8",
            )

            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 2, root)
            )


if __name__ == "__main__":
    unittest.main()
