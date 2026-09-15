"""Regression tests for Rust match-arm line-comment classification."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import check_coverage as coverage_contract


class MatchArmLineCommentTests(unittest.TestCase):
    """Distinguish Rust line comments from ``//`` bytes inside literals."""

    def test_string_internal_slashes_do_not_make_next_literal_a_match_body(self) -> None:
        """An unrelated string containing ``=>//`` cannot promote the next literal."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "independent.rs"
            source.write_text(
                "fn independent() {\n"
                "    let marker = \"=>//\";\n"
                "    \"orphan\";\n"
                "}\n",
                encoding="utf-8",
            )
            self.assertFalse(
                coverage_contract.is_executable_source_line(str(source), 3, root)
            )

    def test_actual_line_comment_after_match_label_keeps_literal_body_executable(self) -> None:
        """A real comment after ``=>`` must not hide the following arm body."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "match_arm.rs"
            source.write_text(
                "fn label(value: i32) -> &'static str {\n"
                "    match value {\n"
                "        _ => // audited arm\n"
                "            \"fallback\",\n"
                "    }\n"
                "}\n",
                encoding="utf-8",
            )
            self.assertTrue(
                coverage_contract.is_executable_source_line(str(source), 4, root)
            )


if __name__ == "__main__":
    unittest.main()
