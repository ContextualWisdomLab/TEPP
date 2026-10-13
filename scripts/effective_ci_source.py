"""Resolve reviewed CI supplier bytes offline with immutable Git-object custody.

Fixtures are non-executable evidence, not alternative GitHub Actions jobs. A pin
or caller change requires a reviewed fixture refresh, never a network fallback.
"""

from __future__ import annotations

import hashlib
from pathlib import Path

SUPPLIER_PIN = "3136c940235a3762b804bd99eb20fbe129e70c06"
CALLER_SHA256 = "4d89992569e0de28dccdcf1912ff9ecb5b74e7c024408c35676ec5c518f71e9d"
SUPPLIER_SHA256 = "c0022bb22b8ebf0f70c65251f3902773f6d323fcdde62d042e17fc5ddccfd01a"
DOCUMENTATION_CALLER_SHA256 = "ba28bb65d86519fce5957cd4cf06cfb276ae637c7087a844e9e7e1c5c935b808"
DOCUMENTATION_SUPPLIER_SHA256 = "d1d62e9ba4ff5ecc0ef499061dae5ea84b05db4a5d94f42b0290d83b8f5885d5"
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
