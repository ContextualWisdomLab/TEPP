"""Tests for the TEPP workspace contract checker."""

from __future__ import annotations

import contextlib
import io
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import check_workspace_contract as contract


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


class WorkspaceContractTests(unittest.TestCase):
    """Exercise successful and fail-closed workspace validation."""

    def copy_repository(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        """Copy the repository to a disposable test directory."""

        temporary = tempfile.TemporaryDirectory()
        destination = Path(temporary.name) / "repository"
        shutil.copytree(
            REPOSITORY_ROOT,
            destination,
            ignore=shutil.ignore_patterns(
                "target", ".git", ".coverage", "__pycache__", "*.pyc"
            ),
        )
        return temporary, destination

    def test_live_repository_satisfies_contract(self) -> None:
        """The committed workspace satisfies every repository contract."""

        self.assertEqual(contract.validate_workspace(REPOSITORY_ROOT), [])

    def test_standards_register_cites_rfc_5646_once(self) -> None:
        """The APA register must not duplicate Phillips & Davis RFC 5646."""

        text = (
            REPOSITORY_ROOT / "docs" / "research" / "standards-and-literature.md"
        ).read_text(encoding="utf-8")
        self.assertEqual(text.count("RFC 5646"), 1)

    def test_liu_2023_register_cites_graham_neubig(self) -> None:
        """The Liu et al. (2023) survey must keep Graham Neubig's initial."""

        text = (
            REPOSITORY_ROOT / "docs" / "research" / "standards-and-literature.md"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "Liu, P., Yuan, W., Fu, J., Jiang, Z., Hayashi, H., & Neubig, G. (2023).",
            text,
        )
        self.assertNotIn("Neubig, P.", text)

    def test_member_paths_match_expected_crates(self) -> None:
        """Workspace members resolve to the approved crate roots by name."""

        self.assertEqual(
            contract.expected_member_paths(),
            [f"crates/{name}" for name in contract.EXPECTED_CRATES],
        )

    def test_placeholder_api_detection_boundaries(self) -> None:
        """Real APIs pass and placeholder or todo bodies are refused."""

        self.assertFalse(contract._contains_placeholder_api("//! documented\n"))
        self.assertFalse(
            contract._contains_placeholder_api("/// Real API.\npub struct EvidenceId;\n")
        )
        self.assertTrue(
            contract._contains_placeholder_api("pub struct Placeholder;\n")
        )
        self.assertTrue(
            contract._contains_placeholder_api("pub fn run() { todo!() }\n")
        )
        self.assertTrue(
            contract._contains_placeholder_api("fn private() { unimplemented!() }\n")
        )

    def test_mapping_normalizes_only_toml_tables(self) -> None:
        """TOML tables map to dictionaries; other shapes fail to empty maps."""

        self.assertEqual(contract._mapping({"key": "value"}), {"key": "value"})
        self.assertEqual(contract._mapping("not-a-table"), {})

    def test_missing_root_manifest_fails_closed(self) -> None:
        """A repository without a root manifest cannot pass."""

        with tempfile.TemporaryDirectory() as temporary:
            self.assertEqual(
                contract.validate_workspace(Path(temporary)),
                ["Cargo.toml is missing"],
            )

    def test_invalid_root_and_crate_contracts_are_reported(self) -> None:
        """Root, package, source, and test drift is reported in one pass."""

        temporary, repository = self.copy_repository()
        self.addCleanup(temporary.cleanup)

        root_manifest = (repository / "Cargo.toml").read_text(encoding="utf-8")
        replacements = {
            'resolver = "2"': 'resolver = "1"',
            '"crates/tepp_api"': '"crates/unapproved_api"',
            'edition = "2024"': 'edition = "2021"',
            'rust-version = "1.98.0"': 'rust-version = "1.96.0"',
            'license = "Apache-2.0"': 'license = "MIT"',
            'unsafe_code = "deny"': 'unsafe_code = "allow"',
            'missing_docs = "deny"': 'missing_docs = "warn"',
            'warnings = "deny"': 'warnings = "warn"',
        }
        for before, after in replacements.items():
            root_manifest = root_manifest.replace(before, after)
        (repository / "Cargo.toml").write_text(root_manifest, encoding="utf-8")

        crate_root = repository / "crates" / "evidence_core"
        manifest = (crate_root / "Cargo.toml").read_text(encoding="utf-8")
        manifest = manifest.replace(
            'name = "evidence_core"', 'name = "wrong_package"'
        )
        manifest = manifest.replace("publish = false", "publish = true")
        manifest = manifest.replace("workspace = true", "workspace = false")
        for inherited in (
            "version.workspace = true\n",
            "edition.workspace = true\n",
            "rust-version.workspace = true\n",
            "license.workspace = true\n",
            "authors.workspace = true\n",
            "repository.workspace = true\n",
            "homepage.workspace = true\n",
            "readme.workspace = true\n",
            "keywords.workspace = true\n",
            "categories.workspace = true\n",
        ):
            manifest = manifest.replace(inherited, "")
        (crate_root / "Cargo.toml").write_text(manifest, encoding="utf-8")
        (crate_root / "src" / "lib.rs").write_text(
            "pub struct Placeholder;\n", encoding="utf-8"
        )
        (crate_root / "tests" / "crate_contract.rs").unlink()
        shutil.rmtree(repository / "crates" / "temporal_core")

        errors = contract.validate_workspace(repository)
        expected_fragments = (
            "workspace resolver",
            "workspace members",
            "workspace default-members",
            "workspace edition",
            "workspace rust-version",
            "workspace license",
            "deny unsafe_code",
            "deny missing_docs",
            "deny warnings",
            "package.name",
            "publish must be false",
            "lints.workspace",
            "must inherit from workspace",
            "crate-level rustdoc",
            "unsafe_code is not explicitly forbidden",
            "missing_docs is not explicitly denied",
            "placeholder production APIs",
            "package identity contract test",
            "crates/temporal_core/Cargo.toml is missing",
        )
        for fragment in expected_fragments:
            self.assertTrue(
                any(fragment in error for error in errors),
                f"missing diagnostic containing {fragment!r}: {errors}",
            )

    def test_missing_library_and_ci_assets_are_reported(self) -> None:
        """Missing source, CI, toolchain, and policy files are rejected."""

        temporary, repository = self.copy_repository()
        self.addCleanup(temporary.cleanup)
        (repository / "crates" / "event_core" / "src" / "lib.rs").unlink()
        (repository / ".github" / "workflows" / "ci.yml").unlink()

        errors = contract.validate_workspace(repository)
        self.assertIn("event_core: src/lib.rs is missing", errors)
        self.assertIn(".github/workflows/ci.yml is missing", errors)

        ci_path = repository / ".github" / "workflows" / "ci.yml"
        ci_path.write_text(
            "uses: actions/checkout@v4\n"
            "env:\n"
            "  COPILOT_GITHUB_TOKEN: forbidden\n"
            "  NVIDIA_NIM_API_KEY: forbidden\n"
            "  CACHE_PATH: ~/.cargo/registry\n",
            encoding="utf-8",
        )
        (repository / "rust-toolchain.toml").unlink()
        (repository / "deny.toml").unlink()
        errors = contract.validate_workspace(repository)
        self.assertTrue(any("required command" in error for error in errors))
        self.assertTrue(
            any("must not reference COPILOT_GITHUB_TOKEN" in error for error in errors)
        )
        self.assertTrue(
            any("must not receive an LLM credential" in error for error in errors)
        )
        self.assertTrue(any("must not cache mutable Cargo" in error for error in errors))
        self.assertIn("rust-toolchain.toml is missing", errors)
        self.assertIn("deny.toml is missing", errors)
        self.assertTrue(any("full commit SHA" in error for error in errors))
        self.assertTrue(any("COPILOT_GITHUB_TOKEN is prohibited" in error for error in errors))

    def test_action_pin_validator_handles_absent_directory(self) -> None:
        """The action-pin validator fails closed when workflows are absent."""

        with tempfile.TemporaryDirectory() as temporary:
            errors = contract._validate_action_pins(Path(temporary) / "workflows")
        self.assertEqual(errors, [".github/workflows directory is missing"])

    def test_print_and_main_exit_codes(self) -> None:
        """Human-readable reporting uses conventional exit codes."""

        standard_output = io.StringIO()
        with contextlib.redirect_stdout(standard_output):
            self.assertEqual(contract.print_errors([]), 0)
        self.assertIn("PASS", standard_output.getvalue())

        standard_error = io.StringIO()
        with contextlib.redirect_stderr(standard_error):
            self.assertEqual(contract.print_errors(["problem"]), 1)
        self.assertIn("problem", standard_error.getvalue())

        self.assertEqual(contract.main([str(REPOSITORY_ROOT)]), 0)
        with mock.patch.object(sys, "argv", ["checker", str(REPOSITORY_ROOT)]):
            self.assertEqual(contract.main(None), 0)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
