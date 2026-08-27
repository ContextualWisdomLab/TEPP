"""Direct branch-coverage tests for the workspace contract checker.

Each test builds a minimal synthetic repository and asserts the presence of
the exact diagnostic that a specific guard produces, so every branch in
``scripts/check_workspace_contract.py`` is exercised without depending on a
single large broken-manifest fixture. These tests deliberately use tiny trees
and never touch the live repository.
"""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import check_workspace_contract as contract


def write(root: Path, relative: str, text: str) -> None:
    """Write *text* at *relative* beneath *root*, creating parents."""
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def stub_ci(root: Path) -> None:
    """Write a minimal CI file and valid shared tooling files."""
    snippets = "\n".join(f"  {snippet}" for snippet in contract.REQUIRED_CI_SNIPPETS)
    write(root, ".github/workflows/ci.yml", f"env:\n{snippets}\n")
    write(root, "rust-toolchain.toml", "")
    write(root, "deny.toml", "")


class DirectWorkspaceContractTests(unittest.TestCase):
    """Exercise each remaining branch of the workspace contract checker."""

    def test_root_workspace_fields_reject_each_bad_value(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            write(
                root,
                "Cargo.toml",
                '[workspace]\npackage = { edition = "2021" }\n'
                f"members = {[f'crates/{crate}' for crate in contract.EXPECTED_CRATES[:-1]]!r}\n"
                "default-members = []\n"
                'lints.rust = { unsafe_code = "warn", warnings = "warn" }\n',
            )
            stub_ci(root)
            errors = contract.validate_workspace(root)
            for expected in (
                "workspace resolver must be 2",
                "workspace default-members must exactly match workspace members",
                "workspace edition must be 2024",
                "workspace rust-version must be 1.98.0",
                "workspace license must be Apache-2.0",
                "workspace must deny unsafe_code",
                "workspace must deny missing_docs",
                "workspace must deny warnings",
            ):
                self.assertTrue(
                    any(expected in error for error in errors),
                    f"missing diagnostic {expected!r}",
                )

    def test_crate_contract_reports_each_manifest_and_source_violation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            crate = contract.EXPECTED_CRATES[0]
            write(root, f"crates/{crate}/Cargo.toml", "[package]\npublish = true\n")
            write(root, f"crates/{crate}/src/lib.rs", "pub fn run() { todo!() }\n")
            errors = contract._validate_crate(root, crate)
            for expected in (
                "package.name must match its directory",
                "publish must be false",
                "lints.workspace must be true",
                "must inherit from workspace",
                "crate-level rustdoc is missing",
                "unsafe_code is not explicitly forbidden",
                "missing_docs is not explicitly denied",
                "placeholder production APIs are prohibited",
                "package identity contract test is missing",
            ):
                self.assertTrue(
                    any(expected in error for error in errors),
                    f"missing diagnostic {expected!r} in {errors}",
                )

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            errors = contract._validate_crate(root, contract.EXPECTED_CRATES[0])
            self.assertEqual(errors, ["crates/evidence_core/Cargo.toml is missing"])

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            crate = contract.EXPECTED_CRATES[0]
            write(root, f"crates/{crate}/Cargo.toml", "[package]\n")
            errors = contract._validate_crate(root, crate)
            self.assertTrue(
                any("src/lib.rs is missing" in e for e in errors),
                f"missing src/lib.rs diagnostic: {errors}",
            )


    def test_ci_contract_missing_file_diagnostic(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            errors = contract._validate_ci_contract(Path(temporary))
            self.assertEqual(errors, [".github/workflows/ci.yml is missing"])

    def test_ci_contract_and_action_pins_reject_violations(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            write(
                root,
                ".github/workflows/ci.yml",
                "COPILOT_GITHUB_TOKEN: forbidden\nexport NVIDIA_NIM_API_KEY=raw\n"
                "~/.cargo/registry\n",
            )
            errors = contract._validate_ci_contract(root)
            self.assertTrue(
                any("COPILOT_GITHUB_TOKEN" in e for e in errors),
                "missing COPILOT prohibition",
            )
            self.assertTrue(
                any("must not receive an LLM credential" in e for e in errors),
                "missing LLM credential prohibition",
            )
            self.assertTrue(
                any("must not cache mutable Cargo registry" in e for e in errors),
                "missing registry-cache prohibition",
            )
            self.assertTrue(
                any("rust-toolchain.toml is missing" in e for e in errors),
                "missing toolchain diagnostic",
            )
            self.assertTrue(
                any("deny.toml is missing" in e for e in errors),
                "missing deny.toml diagnostic",
            )

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            work = root / ".github" / "workflows"
            write(root, ".github/workflows/bad.yml", "uses: actions/checkout@v4\n")
            write(root, "uses.txt", "uses: actions/checkout@v4\n")
            write(
                root,
                ".github/workflows/leak.yml",
                "COPILOT_GITHUB_TOKEN: nope\n",
            )
            errors = contract._validate_action_pins(work)
            self.assertTrue(
                any("must use a full commit SHA" in e for e in errors),
                "missing action-pin diagnostic",
            )
            self.assertTrue(
                any("COPILOT_GITHUB_TOKEN is prohibited" in e for e in errors),
                "missing workflow COPILOT prohibition",
            )


if __name__ == "__main__":  # pragma: no cover
    unittest.main()