"""Prevent multiline-comment trivia from erasing a closed coverage boundary."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import check_coverage as coverage_contract


class StructuralCommentBoundaryTests(unittest.TestCase):
    """Retain the denominator without borrowing execution from sibling code."""

    def totals(self, source_text: str, records: list[tuple[int, int]]) -> tuple[int, int]:
        """Measure synthetic Rust rows through the production LCOV loader."""
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "guard.rs"
            source.write_text(source_text, encoding="utf-8")
            report = root / "report.lcov"
            report.write_text(
                f"SF:{source}\n"
                + "".join(f"DA:{line},{count}\n" for line, count in records)
                + "end_of_record\n",
                encoding="utf-8",
            )
            lines = coverage_contract.load_lcov_line_totals(report, root)["lines"]
            return lines["count"], lines["covered"]

    def test_closer_with_multiline_comment_stops_sibling_execution_proof(self) -> None:
        """A brace followed by an open comment still closes the original block."""
        self.assertEqual(
            self.totals(
                "fn validate(ready: bool) {\n"
                "    if ready {\n"
                "    } /* audit\n"
                "       */\n"
                "    do_work();\n"
                "}\n",
                [(2, 0), (5, 4)],
            ),
            (2, 1),
        )

    def test_line_comment_token_inside_block_does_not_hide_closer(self) -> None:
        """A slash pair inside a block comment cannot hide later structural code."""
        self.assertEqual(
            self.totals(
                "fn validate(ready: bool) {\n"
                "    if ready {\n"
                "        /* audit\n"
                "        // */ }\n"
                "    do_work();\n"
                "}\n",
                [(2, 0), (5, 4)],
            ),
            (2, 1),
        )

    def test_closer_after_block_comment_retains_trailing_line_comment(self) -> None:
        """An actual trailing line comment cannot erase the preceding closer."""
        self.assertEqual(
            self.totals(
                "fn validate(ready: bool) {\n"
                "    if ready {\n"
                "        /* audit\n"
                "        // */ } // closed\n"
                "    do_work();\n"
                "}\n",
                [(2, 0), (5, 4)],
            ),
            (2, 1),
        )

    def test_structural_closer_da_does_not_change_authored_denominator(self) -> None:
        """A zero DA for a brace before an open comment remains structural."""
        self.assertEqual(
            self.totals(
                "fn validate(ready: bool) {\n"
                "    if ready {\n"
                "    } /* audit\n"
                "       */\n"
                "    do_work();\n"
                "}\n",
                [(2, 0), (3, 0), (5, 4)],
            ),
            (2, 1),
        )

    def test_executable_suffix_after_block_internal_slashes_keeps_zero_da(self) -> None:
        """Code after a block-comment terminator remains in the denominator."""
        self.assertEqual(
            self.totals(
                "fn run() {\n    /* audit\n    // */ do_work();\n"
                "    other_work();\n}\n",
                [(3, 0), (4, 4)],
            ),
            (2, 1),
        )

    def test_executable_prefix_before_comment_and_closer_keeps_zero_da(self) -> None:
        """A comment and closer cannot discard an earlier executable call."""
        self.assertEqual(
            self.totals(
                "fn run() {\n    do_work(); /* audit */ }\n"
                "fn exercised() {\n    other_work();\n}\n",
                [(2, 0), (4, 4)],
            ),
            (2, 1),
        )

    def test_opener_after_block_internal_slashes_accepts_nested_execution(self) -> None:
        """An opener after a block comment retains genuine nested execution proof."""
        self.assertEqual(
            self.totals(
                "fn run(ready: bool) {\n    /* audit\n    // */ if ready {\n"
                "        do_work();\n    }\n}\n",
                [(3, 0), (4, 4)],
            ),
            (2, 2),
        )

    def test_comment_execution_matrix_preserves_every_measured_call(self) -> None:
        """Comment variants retain both zero and positive executable DA rows."""
        cases = (
            ("fn run() {\n    /* audit\n    // */ do_work();\n    other_work();\n}\n", 3, 4),
            ("fn run() {\n    do_work(); /* audit */ }\nfn exercised() {\n    other_work();\n}\n", 2, 4),
            ("fn run() {\n    /* outer\n    /* nested */ // */ do_work(); // trailing\n    other_work();\n}\n", 3, 4),
            ('fn run() {\n    let value = r#"/* */ //"#;\n    other_work();\n}\n', 2, 3),
            ("fn run() {\n    let value = '/';\n    other_work();\n}\n", 2, 3),
        )
        for source, focal, other in cases:
            for count in (0, 4):
                with self.subTest(focal=focal, count=count, source=source):
                    self.assertEqual(
                        self.totals(source, [(focal, count), (other, 4)]),
                        (2, 1 if count == 0 else 2),
                    )

    def test_multiline_literal_suffix_remains_executable(self) -> None:
        """Closing a multiline literal cannot hide a following zero-count call."""
        self.assertEqual(
            self.totals(
                'fn run() {\n    let text = "first\nsecond"; do_work();\n'
                '    other_work();\n}\n',
                [(3, 0), (4, 4)],
            ),
            (2, 1),
        )

    def test_literal_match_arm_after_block_internal_slashes_keeps_zero_da(self) -> None:
        """A lexical comment prefix cannot hide an unexecuted literal match arm."""
        self.assertEqual(
            self.totals(
                "fn label(value: bool) -> &'static str {\n    match value {\n"
                '        /* audit\n        // */ true =>\n            "uncovered",\n'
                '        false => "covered",\n    }\n}\n',
                [(5, 0), (6, 4)],
            ),
            (2, 1),
        )

    def test_shared_literal_and_arm_matrix_preserves_authority(self) -> None:
        """Literal endings and comment-prefixed arms retain measured execution."""
        literals = (
            ('"first', 'second"'),
            ('r##"first', 'second"##'),
            ('br#"first', 'second"#'),
            ('b"first', 'second"'),
        )
        for opener, closer in literals:
            for suffix in ("; do_work();", "; /* note */ do_work();", "; // only trivia"):
                source = (
                    f"fn run() {{\n    let text = {opener}\n{closer}{suffix}\n"
                    "    other_work();\n}\n"
                )
                for count in (0, 4):
                    with self.subTest(opener=opener, suffix=suffix, count=count):
                        expected = (1, 1) if "only trivia" in suffix else (2, 1 if count == 0 else 2)
                        self.assertEqual(self.totals(source, [(3, count), (4, 4)]), expected)
        for label in ("true =>", "// */ true =>", "/* inner */ // */ true =>"):
            prefix = "        /* audit\n" if "*/" in label else "\n"
            source = (
                "fn label(value: bool) -> &'static str {\n    match value {\n"
                f'{prefix}        {label}\n            "result",\n'
                '        false => "covered",\n    }\n}\n'
            )
            for count in (0, 4):
                with self.subTest(label=label, count=count):
                    self.assertEqual(self.totals(source, [(5, count), (6, 4)]), (2, 1 if count == 0 else 2))

    def test_multiline_literal_arm_opening_keeps_unexecuted_da(self) -> None:
        """An arm's multiline literal opening remains a measured execution unit."""
        self.assertEqual(
            self.totals(
                "fn label(value: bool) -> &'static str {\n    match value {\n"
                '        true =>\n            "uncovered\n            tail",\n'
                '        false => "covered",\n    }\n}\n',
                [(4, 0), (6, 4)],
            ),
            (2, 1),
        )

    def test_multiline_arm_opening_matrix_preserves_context_and_denominator(self) -> None:
        """Opening DA rows require arm authority while payload DA remains trivia."""
        for opener, closer in (
            ('"uncovered', 'tail"'),
            ('r##"uncovered', 'tail"##'),
            ('br#"uncovered', 'tail"#'),
            ('b"uncovered', 'tail"'),
        ):
            for count in (0, 4):
                for arm in (True, False):
                    for suffix in (",", ", /* note */", "; do_work();"):
                        label = "true =>" if arm else "record("
                        source = (
                            "fn label(value: bool) -> &'static str {\n    match value {\n"
                            f"        {label}\n            {opener}\n            {closer}{suffix}\n"
                            '        false => "covered",\n    }\n}\n'
                        )
                        with self.subTest(opener=opener, count=count, arm=arm, suffix=suffix):
                            total = 1 + int(arm) + int("do_work" in suffix)
                            covered = 1 + int(arm and count > 0) + int("do_work" in suffix)
                            self.assertEqual(
                                self.totals(source, [(4, count), (5, 4), (6, 4)]),
                                (total, covered),
                            )

    def test_plain_closer_still_stops_sibling_execution_proof(self) -> None:
        """The comment repair retains the ordinary structural-boundary guard."""
        self.assertEqual(
            self.totals(
                "fn validate(ready: bool) {\n"
                "    if ready {\n"
                "    }\n"
                "    do_work();\n"
                "}\n",
                [(2, 0), (4, 4)],
            ),
            (2, 1),
        )

    def test_comment_only_gap_retains_real_nested_execution_proof(self) -> None:
        """A comment without structural code still permits valid reconciliation."""
        self.assertEqual(
            self.totals(
                "fn validate(ready: bool) {\n"
                "    if ready {\n"
                "        /* audit\n"
                "           */\n"
                "        do_work();\n"
                "    }\n"
                "}\n",
                [(2, 0), (5, 4)],
            ),
            (2, 2),
        )
