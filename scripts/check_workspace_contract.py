"""Validate the TEPP Rust workspace and repository quality contracts.

The checker deliberately uses only Python's standard library so it can run before
the Rust workspace or third-party quality tools are installed.
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path
from typing import Any, Iterable, Mapping, Sequence

EXPECTED_CRATES: tuple[str, ...] = (
    "evidence_core",
    "temporal_core",
    "event_core",
    "relation_graph",
    "membership_core",
    "persistence_postgres",
    "corpus_split",
    "tepp_simulation",
    "validation_core",
    "tepp_api",
    "inferred_status",
    "support_edge",
    "system_clock",
    "event_clock",
    "assertion_clock",
    "cutoff_clock",
    "available_clock",
    "document_clocks",
    "revision_order",
    "encrypted_mapping",
    "citation_edge",
    "psychometric_fit",
    "subevent_containment",
    "prediction_contradiction",
    "provider_receipt",
    "operational_log",
    "service_tls",
    "derived_sensitivity",
    "longitudinal_core",
    "topic_lineage",
    "network_analysis",
    "interpretation_gateway",
    "model_selection",
    "checkpoint_authority",
)

REQUIRED_CI_SNIPPETS: tuple[str, ...] = (
    "cargo fmt --all -- --check",
    "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    "cargo nextest run --workspace --all-features",
    "cargo test --doc --workspace --all-features",
    "cargo doc --workspace --all-features --no-deps",
    "cargo deny check",
    "cargo llvm-cov --workspace --all-features",
    "python3 scripts/check_docstrings.py",
    "python3 scripts/check_coverage.py",
    "Restore pinned Rust quality tools",
    "Verify pinned Rust quality tool versions",
    "Restore pinned cargo-llvm-cov",
    "Verify pinned cargo-llvm-cov version",
)

ACTION_PATTERN = re.compile(r"^\s*uses:\s*([^\s#]+)@([^\s#]+)", re.MULTILINE)
FULL_SHA_PATTERN = re.compile(r"^[0-9a-fA-F]{40}$")
PLACEHOLDER_PATTERN = re.compile(
    r"(?:\bpub\b[^\n]*(?:Placeholder|placeholder)|\b(?:todo|unimplemented)!\s*\()"
)


def load_toml(path: Path) -> Mapping[str, Any]:
    """Load one UTF-8 TOML document from *path*."""

    with path.open("rb") as stream:
        return tomllib.load(stream)


def expected_member_paths() -> list[str]:
    """Return the canonical ordered workspace member paths."""

    return [f"crates/{crate_name}" for crate_name in EXPECTED_CRATES]


def validate_workspace(root: Path) -> list[str]:
    """Return every workspace-contract violation below *root*.

    The result is deterministic and sorted by validation order so CI diagnostics
    remain stable across operating systems.
    """

    errors: list[str] = []
    root_manifest_path = root / "Cargo.toml"
    if not root_manifest_path.is_file():
        return ["Cargo.toml is missing"]

    root_manifest = load_toml(root_manifest_path)
    workspace = _mapping(root_manifest.get("workspace"))
    package_defaults = _mapping(root_manifest.get("workspace", {}).get("package"))
    rust_lints = _mapping(
        root_manifest.get("workspace", {}).get("lints", {}).get("rust")
    )

    expected_members = expected_member_paths()
    if workspace.get("resolver") != "2":
        errors.append("workspace resolver must be 2")
    if workspace.get("members") != expected_members:
        errors.append("workspace members must exactly match the approved crate list")
    if workspace.get("default-members") != expected_members:
        errors.append("workspace default-members must exactly match workspace members")
    if package_defaults.get("edition") != "2024":
        errors.append("workspace edition must be 2024")
    if package_defaults.get("rust-version") != "1.97.1":
        errors.append("workspace rust-version must be 1.97.1")
    if package_defaults.get("license") != "Apache-2.0":
        errors.append("workspace license must be Apache-2.0")
    if rust_lints.get("unsafe_code") != "forbid":
        errors.append("workspace must forbid unsafe_code")
    if rust_lints.get("missing_docs") != "deny":
        errors.append("workspace must deny missing_docs")
    if rust_lints.get("warnings") != "deny":
        errors.append("workspace must deny warnings")

    for crate_name in EXPECTED_CRATES:
        errors.extend(_validate_crate(root, crate_name))

    errors.extend(_validate_ci_contract(root))
    errors.extend(_validate_action_pins(root / ".github" / "workflows"))
    return errors


def _mapping(value: Any) -> Mapping[str, Any]:
    """Return *value* as a mapping, or an empty mapping for other values."""

    return value if isinstance(value, Mapping) else {}


def _validate_crate(root: Path, crate_name: str) -> list[str]:
    """Return contract violations for one workspace crate."""

    errors: list[str] = []
    crate_root = root / "crates" / crate_name
    manifest_path = crate_root / "Cargo.toml"
    library_path = crate_root / "src" / "lib.rs"
    test_path = crate_root / "tests" / "crate_contract.rs"

    if not manifest_path.is_file():
        return [f"{manifest_path.relative_to(root)} is missing"]

    manifest = load_toml(manifest_path)
    package = _mapping(manifest.get("package"))
    if package.get("name") != crate_name:
        errors.append(f"{crate_name}: package.name must match its directory")
    if package.get("publish") is not False:
        errors.append(f"{crate_name}: publish must be false")
    if _mapping(manifest.get("lints")).get("workspace") is not True:
        errors.append(f"{crate_name}: lints.workspace must be true")
    for inherited_field in (
        "version",
        "edition",
        "rust-version",
        "license",
        "authors",
        "repository",
        "homepage",
        "readme",
        "keywords",
        "categories",
    ):
        if package.get(inherited_field, {}).get("workspace") is not True:
            errors.append(f"{crate_name}: {inherited_field} must inherit from workspace")

    if not library_path.is_file():
        errors.append(f"{crate_name}: src/lib.rs is missing")
    else:
        library_text = library_path.read_text(encoding="utf-8")
        if "//! " not in library_text:
            errors.append(f"{crate_name}: crate-level rustdoc is missing")
        if "#![forbid(unsafe_code)]" not in library_text:
            errors.append(f"{crate_name}: unsafe_code is not explicitly forbidden")
        if "#![deny(missing_docs)]" not in library_text:
            errors.append(f"{crate_name}: missing_docs is not explicitly denied")
        if _contains_placeholder_api(library_text):
            errors.append(f"{crate_name}: placeholder production APIs are prohibited")

    if not test_path.is_file():
        errors.append(f"{crate_name}: package identity contract test is missing")
    return errors


def _contains_placeholder_api(source: str) -> bool:
    """Return whether *source* exposes or executes placeholder behavior."""

    return bool(PLACEHOLDER_PATTERN.search(source))


def _validate_ci_contract(root: Path) -> list[str]:
    """Return violations in the Task 1 CI workflow and toolchain files."""

    errors: list[str] = []
    ci_path = root / ".github" / "workflows" / "ci.yml"
    toolchain_path = root / "rust-toolchain.toml"
    deny_path = root / "deny.toml"

    if not ci_path.is_file():
        return [".github/workflows/ci.yml is missing"]

    ci_text = ci_path.read_text(encoding="utf-8")
    for snippet in REQUIRED_CI_SNIPPETS:
        if snippet not in ci_text:
            errors.append(f"CI workflow is missing required command: {snippet}")
    if "COPILOT_GITHUB_TOKEN" in ci_text:
        errors.append("CI workflow must not reference COPILOT_GITHUB_TOKEN")
    if "NVIDIA_NIM_API_KEY" in ci_text:
        errors.append("Task 1 CI must not receive an LLM credential")
    if "~/.cargo/registry" in ci_text or "~/.cargo/git" in ci_text:
        errors.append("CI must not cache mutable Cargo registry or Git source trees")
    if not toolchain_path.is_file():
        errors.append("rust-toolchain.toml is missing")
    if not deny_path.is_file():
        errors.append("deny.toml is missing")
    return errors


def _validate_action_pins(workflow_root: Path) -> list[str]:
    """Return unpinned GitHub Action and reusable-workflow references."""

    errors: list[str] = []
    if not workflow_root.is_dir():
        return [".github/workflows directory is missing"]
    for workflow_path in sorted(workflow_root.glob("*.y*ml")):
        workflow_text = workflow_path.read_text(encoding="utf-8")
        if "COPILOT_GITHUB_TOKEN" in workflow_text:
            errors.append(
                f"{workflow_path.name}: COPILOT_GITHUB_TOKEN is prohibited"
            )
        for action_name, action_ref in ACTION_PATTERN.findall(workflow_text):
            if not FULL_SHA_PATTERN.fullmatch(action_ref):
                errors.append(
                    f"{workflow_path.name}: {action_name} must use a full commit SHA"
                )
    return errors


def print_errors(errors: Sequence[str]) -> int:
    """Print *errors* and return a conventional process exit code."""

    if not errors:
        print("TEPP workspace contract: PASS")
        return 0
    print("TEPP workspace contract: FAIL", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    return 1


def main(arguments: Iterable[str] | None = None) -> int:
    """Validate a repository root supplied as the first argument."""

    supplied = list(arguments if arguments is not None else sys.argv[1:])
    root = Path(supplied[0] if supplied else ".").resolve()
    return print_errors(validate_workspace(root))


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
