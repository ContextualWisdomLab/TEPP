"""Contract for reconciling impossible LCOV zero counts without shrinking denominator."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import check_coverage as coverage_contract


def lcov(source: Path, records: list[tuple[int, int]]) -> str:
    """Return a framed LCOV report for one source file."""

    body = "".join(f"DA:{line},{count}\n" for line, count in records)
    return f"SF:{source}\n{body}end_of_record\n"


class ContradictoryZeroCountTests(unittest.TestCase):
    """A block body cannot run while the line opening it never does."""

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.addCleanup(self._tmp.cleanup)
        self.source = self.root / "guard.rs"
        self.source.write_text(
            "fn validate(value: &str) -> Result<(), Error> {\n"
            "    let lower = value.to_ascii_lowercase();\n"
            "    if !lower.contains(\"marker\") {\n"
            "        return Err(Error::Missing);\n"
            "    }\n"
            "    Ok(())\n"
            "}\n",
            encoding="utf-8",
        )

    def totals(self, records: list[tuple[int, int]]) -> tuple[int, int]:
        report = self.root / "report.lcov"
        report.write_text(lcov(self.source, records), encoding="utf-8")
        loaded = coverage_contract.load_lcov_line_totals(report, self.root)["lines"]
        return loaded["count"], loaded["covered"]

    def test_impossible_zero_is_reconciled_without_shrinking_denominator(self) -> None:
        """A positive nested body proves its zero-count opener executed."""

        count, covered = self.totals([(2, 9), (3, 0), (4, 2), (6, 7)])
        self.assertEqual((count, covered), (4, 4))

    def test_inline_comment_after_opener_preserves_the_reconciliation_proof(self) -> None:
        """Trailing comments cannot hide an opener whose body demonstrably ran."""

        source = self.root / "commented_guard.rs"
        source.write_text(
            "fn validate(value: &str) {\n"
            "    if value.contains(\"//\") { // audited guard\n"
            "        println!(\"accepted\");\n"
            "    }\n"
            "}\n",
            encoding="utf-8",
        )
        report = self.root / "commented_guard.lcov"
        report.write_text(lcov(source, [(2, 0), (3, 4)]), encoding="utf-8")
        loaded = coverage_contract.load_lcov_line_totals(report, self.root)["lines"]
        self.assertEqual((loaded["count"], loaded["covered"]), (2, 2))

    def test_blank_and_line_comment_gap_preserves_nested_execution_proof(self) -> None:
        """Non-authored trivia cannot make a proven-executed opener look uncovered."""

        source = self.root / "gapped_guard.rs"
        source.write_text(
            "fn validate(ready: bool) {\n"
            "    if ready {\n"
            "\n"
            "        // audit note\n"
            "        println!(\"accepted\");\n"
            "    }\n"
            "}\n",
            encoding="utf-8",
        )
        report = self.root / "gapped_guard.lcov"
        report.write_text(lcov(source, [(2, 0), (5, 4)]), encoding="utf-8")
        loaded = coverage_contract.load_lcov_line_totals(report, self.root)["lines"]
        self.assertEqual((loaded["count"], loaded["covered"]), (2, 2))

    def test_block_comment_only_gap_preserves_nested_execution_proof(self) -> None:
        """Block-comment-only trivia cannot break proof from the first authored body."""

        source = self.root / "block_comment_gap.rs"
        source.write_text(
            "fn validate(ready: bool) {\n"
            "    if ready {\n"
            "        /* audit note\n"
            "           continued */\n"
            "        println!(\"accepted\");\n"
            "    }\n"
            "}\n",
            encoding="utf-8",
        )
        report = self.root / "block_comment_gap.lcov"
        report.write_text(lcov(source, [(2, 0), (5, 4)]), encoding="utf-8")
        loaded = coverage_contract.load_lcov_line_totals(report, self.root)["lines"]
        self.assertEqual((loaded["count"], loaded["covered"]), (2, 2))

    def test_reconciliation_does_not_cross_a_closed_block(self) -> None:
        """A later sibling statement cannot prove an empty opener executed."""

        source = self.root / "closed_guard.rs"
        source.write_text(
            "fn validate(ready: bool) {\n"
            "    if ready {\n"
            "    }\n"
            "    println!(\"outside\");\n"
            "}\n",
            encoding="utf-8",
        )
        report = self.root / "closed_guard.lcov"
        report.write_text(lcov(source, [(2, 0), (4, 4)]), encoding="utf-8")
        loaded = coverage_contract.load_lcov_line_totals(report, self.root)["lines"]
        self.assertEqual((loaded["count"], loaded["covered"]), (2, 1))

    def test_multiline_block_comment_closer_with_code_stays_authored(self) -> None:
        """Code after a block-comment closer is executable, not comment continuation."""

        source = self.root / "comment_closer_code.rs"
        source.write_text(
            "fn validate() {\n"
            "    /* audit note\n"
            "    */ do_work();\n"
            "}\n",
            encoding="utf-8",
        )
        self.assertTrue(
            coverage_contract.is_executable_source_line(str(source), 3, self.root)
        )

        report = self.root / "comment_closer_code.lcov"
        report.write_text(lcov(source, [(3, 0)]), encoding="utf-8")
        loaded = coverage_contract.load_lcov_line_totals(report, self.root)["lines"]
        self.assertEqual((loaded["count"], loaded["covered"]), (1, 0))

    def test_multiline_comment_closer_before_structural_brace_is_not_authored(self) -> None:
        """A comment closer must not turn a structural closing brace into authored code."""

        source = self.root / "comment_closer_brace.rs"
        source.write_text(
            "fn validate(ready: bool) {\n"
            "    if ready {\n"
            "        /* audit note\n"
            "        */ }\n"
            "    println!(\"outside\");\n"
            "}\n",
            encoding="utf-8",
        )
        self.assertFalse(
            coverage_contract.is_executable_source_line(str(source), 4, self.root)
        )
        report = self.root / "comment_closer_brace.lcov"
        report.write_text(lcov(source, [(4, 0), (5, 3)]), encoding="utf-8")
        loaded = coverage_contract.load_lcov_line_totals(report, self.root)["lines"]
        self.assertEqual((loaded["count"], loaded["covered"]), (1, 1))

    def test_complete_block_comment_before_structural_brace_is_not_authored(self) -> None:
        """Same-line block-comment trivia cannot disguise a structural brace."""

        source = self.root / "inline_comment_brace.rs"
        source.write_text(
            "fn validate(ready: bool) {\n"
            "    if ready {\n"
            "        /* audit note */ }\n"
            "    println!(\"outside\");\n"
            "}\n",
            encoding="utf-8",
        )
        self.assertFalse(
            coverage_contract.is_executable_source_line(str(source), 3, self.root)
        )
        report = self.root / "inline_comment_brace.lcov"
        report.write_text(lcov(source, [(3, 0), (4, 3)]), encoding="utf-8")
        loaded = coverage_contract.load_lcov_line_totals(report, self.root)["lines"]
        self.assertEqual((loaded["count"], loaded["covered"]), (1, 1))

    def test_rust_line_comment_scanner_preserves_literal_comment_markers(self) -> None:
        """Only a lexical Rust line comment is removed from an opener probe."""

        cases = (
            ("if ready { // audited", "if ready { "),
            ('if value == "//" { // audited', 'if value == "//" { '),
            ('if value == "escaped \\\"// marker" { // audited', 'if value == "escaped \\\"// marker" { '),
            ('if value == r#"//"# { // audited', 'if value == r#"//"# { '),
            ("if value == '/' { // audited", "if value == '/' { "),
            ("if value == '\\'' { // audited", "if value == '\\'' { "),
            ("if ready /* outer /* nested */ still outer */ { // audited", "if ready  { "),
            ("if ready /* open block comment", "if ready "),
            ('let value = r#"raw string continues // {', 'let value = r#"raw string continues // {'),
            ("let lifetime: &'a str = value;", "let lifetime: &'a str = value;"),
        )
        for source, expected in cases:
            with self.subTest(source=source):
                self.assertEqual(
                    coverage_contract._rust_code_before_line_comment(source), expected
                )

    def test_genuine_zero_on_a_block_opener_is_kept(self) -> None:
        """When the body never ran either, the zero is a real gap."""

        count, covered = self.totals([(2, 9), (3, 0), (4, 0), (6, 7)])
        self.assertEqual((count, covered), (4, 2))

    def test_zero_on_a_line_that_opens_no_block_is_kept(self) -> None:
        """Only a block opener can be contradicted by the line beneath it."""

        count, covered = self.totals([(2, 0), (3, 4), (4, 2), (6, 7)])
        self.assertEqual((count, covered), (4, 3))

    def test_zero_without_a_following_record_is_kept(self) -> None:
        """An opener with no measured body has nothing to contradict it."""

        count, covered = self.totals([(2, 9), (3, 0), (6, 7)])
        self.assertEqual((count, covered), (3, 2))


class NestedLineAbsenceTests(unittest.TestCase):
    """An opener with nothing meaningful beneath it cannot be contradicted."""

    def test_opener_with_no_following_source_line_keeps_its_zero(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "trailing_opener.rs"
            source.write_text(
                "fn run() -> Result<(), Error> {\n"
                "    let value = compute();\n"
                "    if value.is_empty() {\n"
                "        // only a comment lives here\n",
                encoding="utf-8",
            )
            report = root / "report.lcov"
            report.write_text(
                f"SF:{source}\nDA:2,9\nDA:3,0\nend_of_record\n", encoding="utf-8"
            )
            loaded = coverage_contract.load_lcov_line_totals(report, root)["lines"]
            self.assertEqual((loaded["count"], loaded["covered"]), (2, 1))


class BlockOpenerProbeTests(unittest.TestCase):
    """Coverage source probes fail closed at filesystem and trivia boundaries."""

    def test_unreadable_source_opens_no_block(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            missing = Path(temporary) / "absent.rs"
            self.assertFalse(
                coverage_contract.opens_a_block(str(missing), 1, Path(temporary))
            )
            self.assertIsNone(
                coverage_contract._first_meaningful_source_line_after(
                    str(missing), 1, Path(temporary)
                )
            )

    def test_line_outside_the_file_opens_no_block(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "short.rs"
            source.write_text("fn run() {\n", encoding="utf-8")
            root = Path(temporary)
            self.assertTrue(coverage_contract.opens_a_block(str(source), 1, root))
            self.assertFalse(coverage_contract.opens_a_block(str(source), 0, root))
            self.assertFalse(coverage_contract.opens_a_block(str(source), 9, root))

    def test_absent_repository_root_reads_the_path_directly(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "direct.rs"
            source.write_text("fn run() {\n    work();\n", encoding="utf-8")
            self.assertTrue(coverage_contract.opens_a_block(str(source), 1, None))
            self.assertEqual(
                coverage_contract._first_meaningful_source_line_after(str(source), 1, None),
                2,
            )

    def test_only_blank_and_line_comments_produce_no_nested_evidence_line(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "trivia.rs"
            source.write_text("if ready {\n\n    // note\n", encoding="utf-8")
            self.assertIsNone(
                coverage_contract._first_meaningful_source_line_after(
                    str(source), 1, Path(temporary)
                )
            )


if __name__ == "__main__":
    unittest.main()