"""Tests for Rust public-API documentation validation."""

from __future__ import annotations

import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import check_docstrings as docstrings
from scripts import check_workspace_contract as workspace_contract
from scripts.check_workspace_contract import EXPECTED_CRATES
from scripts import check_workspace_contract as contract


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


class DocstringContractTests(unittest.TestCase):
    """Exercise Rust documentation discovery and validation."""

    def test_live_repository_is_documented(self) -> None:
        """Every crate root and production module is discovered and documented."""

        sources = docstrings.rust_sources(REPOSITORY_ROOT)
        crate_roots = sorted(REPOSITORY_ROOT.glob("crates/*/src/lib.rs"))
        self.assertEqual(
            sorted(path.parent.parent.name for path in crate_roots),
            sorted(EXPECTED_CRATES),
        )
        expected_crate_roots = {
            REPOSITORY_ROOT / path / "src" / "lib.rs"
            for path in contract.expected_member_paths()
        }
        self.assertEqual(set(crate_roots), expected_crate_roots)
        self.assertEqual(len(crate_roots), len(workspace_contract.EXPECTED_CRATES))
        self.assertEqual(len(crate_roots), len(contract.EXPECTED_CRATES))
        self.assertTrue(set(crate_roots).issubset(sources))
        self.assertGreaterEqual(len(sources), len(crate_roots))
        self.assertEqual(docstrings.validate_repository(REPOSITORY_ROOT), [])

    def test_missing_sources_fail_closed(self) -> None:
        """A repository with no production Rust source cannot pass."""

        with tempfile.TemporaryDirectory() as temporary:
            self.assertEqual(
                docstrings.validate_repository(Path(temporary)),
                ["no production Rust source files were found"],
            )

    def test_documented_and_undocumented_items(self) -> None:
        """Attributes and whitespace do not detach rustdoc from public items."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "lib.rs"
            source.write_text(
                "//! Module docs.\n"
                "\n"
                "/// A documented structure.\n"
                "#[derive(Debug)]\n"
                "pub struct Documented;\n"
                "\n"
                "pub fn undocumented() {}\n"
                "\n"
                "/// A documented constant.\n"
                "#[doc = \"Additional documentation.\"]\n"
                "pub const VALUE: usize = 1;\n",
                encoding="utf-8",
            )
            errors = docstrings.validate_source(source)
        self.assertEqual(len(errors), 1)
        self.assertIn("public item lacks", errors[0])

    def test_missing_module_docs_are_reported(self) -> None:
        """Crate or module documentation is mandatory."""

        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "lib.rs"
            source.write_text("fn private_item() {}\n", encoding="utf-8")
            errors = docstrings.validate_source(source)
        self.assertEqual(errors, [f"{source}: missing crate/module-level //! rustdoc"])

    def test_print_and_main_exit_codes(self) -> None:
        """Reporting succeeds for clean repositories and fails for empty ones."""

        standard_output = io.StringIO()
        with contextlib.redirect_stdout(standard_output):
            self.assertEqual(docstrings.print_errors([]), 0)
        self.assertIn("PASS", standard_output.getvalue())

        standard_error = io.StringIO()
        with contextlib.redirect_stderr(standard_error):
            self.assertEqual(docstrings.print_errors(["problem"]), 1)
        self.assertIn("problem", standard_error.getvalue())

        self.assertEqual(docstrings.main([str(REPOSITORY_ROOT)]), 0)
        with mock.patch.object(sys, "argv", ["checker", str(REPOSITORY_ROOT)]):
            self.assertEqual(docstrings.main(None), 0)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
