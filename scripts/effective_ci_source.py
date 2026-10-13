"""Resolve reviewed CI supplier bytes offline with immutable Git-object custody.

Fixtures are non-executable evidence, not alternative GitHub Actions jobs. A pin
or caller change requires a reviewed fixture refresh, never a network fallback.
"""

from __future__ import annotations

import hashlib
from pathlib import Path

SUPPLIER_PIN = "9dab3c2a607ddef69b5ff8cb0f761e90f6bc28de"
CALLER_SHA256 = "4cdbacc6c531bba1fcb0632e9a4ea88ec9814820bb4e8c45447d57bc046201e3"
SUPPLIER_SHA256 = "0baf1384a6f66b357bba6cf74b384ecae447767f512eb2c90bb50849c74a7160"
DOCUMENTATION_CALLER_SHA256 = "99ed24d9ce64a4232dcb29e05ddcdedac659c37f5c8b4230386f57435b3bd1f2"
DOCUMENTATION_SUPPLIER_SHA256 = "76d5f64f05fbf953c8c96b6bf8129880a904498b1bbce355567108975bfb00aa"
FIXTURE_PATH = Path("tests/fixtures/central-ci") / SUPPLIER_PIN


def _verify_object(kind: str, data: bytes, expected: str) -> None:
    """Require the original Git object identity, including type and byte size."""
    header = f"{kind} {len(data)}\0".encode("ascii")
    if hashlib.sha1(header + data).hexdigest() != expected:
        raise ValueError(f"CI supplier {kind} Git object mismatch")


def _tree_child(data: bytes, mode: str, name: str) -> str:
    """Read an exact child from a previously authenticated raw Git tree."""
    marker = f"{mode} {name}\0".encode("ascii")
    start = data.index(marker) + len(marker)
    return data[start:start + 20].hex()


def effective_ci_source(root: Path, supplier_name: str = "tepp-rust-foundation-ci.yml") -> str:
    """Return verified caller plus pinned supplier for static contract checks.

    Exact caller bytes bind dispatch, read-only permissions, concurrency, pin,
    and the absence of competing local jobs. Every raw Git object is checked
    from the caller's reviewed commit through the original tree path to blob;
    SHA-256 additionally binds the workflow bytes. Missing or changed evidence
    fails closed. No Git installation, repository metadata, or network is needed.
    Only the foundation (default) and documentation supplier names are allowed;
    each is bound to its own exact caller and digest at the same reviewed pin.
    """
    contracts = {
        "tepp-rust-foundation-ci.yml": ("ci.yml", CALLER_SHA256, SUPPLIER_SHA256),
        "tepp-documentation-quality.yml": (
            "docs-quality.yml", DOCUMENTATION_CALLER_SHA256, DOCUMENTATION_SUPPLIER_SHA256,
        ),
    }
    if supplier_name not in contracts:
        raise ValueError("CI supplier name is not allowlisted")
    caller_name, caller_sha256, supplier_sha256 = contracts[supplier_name]
    caller = (root / ".github/workflows" / caller_name).read_bytes()
    if hashlib.sha256(caller).hexdigest() != caller_sha256:
        raise ValueError("CI caller differs from reviewed immutable supplier contract")
    fixture = root / FIXTURE_PATH
    commit = (fixture / "commit.object").read_bytes()
    _verify_object("commit", commit, SUPPLIER_PIN)
    expected_tree = commit.split(b"\n", 1)[0].split(b" ", 1)[1].decode("ascii")
    for filename, child in (("root.tree", ".github"), (".github.tree", "workflows")):
        tree = (fixture / filename).read_bytes()
        _verify_object("tree", tree, expected_tree)
        expected_tree = _tree_child(tree, "40000", child)
    tree = (fixture / "workflows.tree").read_bytes()
    _verify_object("tree", tree, expected_tree)
    expected_blob = _tree_child(tree, "100644", supplier_name)
    supplier = (fixture / supplier_name).read_bytes()
    _verify_object("blob", supplier, expected_blob)
    if hashlib.sha256(supplier).hexdigest() != supplier_sha256:
        raise ValueError("CI supplier SHA-256 mismatch")
    return caller.decode("utf-8") + "\n" + supplier.decode("utf-8")
