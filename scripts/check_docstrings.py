"""Enforce beginner-readable Rust documentation on TEPP public APIs."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Iterable, Sequence

PUBLIC_ITEM_PATTERN = re.compile(
    r"^\s*pub\s+"
    r"(?:async\s+|const\s+|unsafe\s+|extern\s+)*"
    r"(?:fn|struct|enum|trait|mod|type|const|static|use)\b"
)


def rust_sources(root: Path) -> list[Path]:
    """Return production Rust source files in deterministic order."""

    return sorted(root.glob("crates/*/src/**/*.rs"))


def validate_source(path: Path) -> list[str]:
    """Return documentation violations in one Rust source file."""

    lines = path.read_text(encoding="utf-8").splitlines()
    errors: list[str] = []
    if not any(line.lstrip().startswith("//!") for line in lines):
        errors.append(f"{path}: missing crate/module-level //! rustdoc")

    documented = False
    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("///") or stripped.startswith("#[doc"):
            documented = True
            continue
        if stripped.startswith("#[") or not stripped:
            continue
        if PUBLIC_ITEM_PATTERN.match(line):
            if not documented:
                errors.append(f"{path}:{line_number}: public item lacks /// rustdoc")
            documented = False
            continue
        documented = False
    return errors


def validate_repository(root: Path) -> list[str]:
    """Return all Rust documentation violations under *root*."""

    sources = rust_sources(root)
    if not sources:
        return ["no production Rust source files were found"]
    errors: list[str] = []
    for source_path in sources:
        errors.extend(validate_source(source_path))
    return errors


def print_errors(errors: Sequence[str]) -> int:
    """Print *errors* and return a conventional process exit code."""

    if not errors:
        print("Rust documentation contract: PASS")
        return 0
    print("Rust documentation contract: FAIL", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    return 1


def main(arguments: Iterable[str] | None = None) -> int:
    """Validate the repository root supplied as the first argument."""

    supplied = list(arguments if arguments is not None else sys.argv[1:])
    root = Path(supplied[0] if supplied else ".").resolve()
    return print_errors(validate_repository(root))


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
