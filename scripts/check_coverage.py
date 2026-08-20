"""Fail closed unless an LLVM coverage report is exactly complete."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable, Mapping, Sequence


def load_totals(path: Path) -> Mapping[str, Any]:
    """Load the single-report totals mapping from LLVM coverage JSON."""

    payload = json.loads(path.read_text(encoding="utf-8"))
    data = payload.get("data")
    if not isinstance(data, list) or len(data) != 1:
        raise ValueError("coverage JSON must contain exactly one data entry")
    totals = data[0].get("totals")
    if not isinstance(totals, Mapping):
        raise ValueError("coverage JSON data entry must contain totals")
    return totals


def resolve_repository_source_path(source_path: str, repository_root: Path) -> Path:
    """Resolve *source_path* and require it stay under *repository_root*.

    Absolute or ``..`` paths that escape the repository root fail closed so an
    untrusted LCOV ``SF:`` record cannot force arbitrary file reads.
    """

    root = repository_root.resolve()
    candidate = Path(source_path)
    resolved = (
        candidate.resolve() if candidate.is_absolute() else (root / candidate).resolve()
    )
    try:
        resolved.relative_to(root)
    except ValueError as error:
        raise ValueError(
            f"LCOV source path escapes repository root: {source_path}"
        ) from error
    return resolved


def is_executable_source_line(
    source_path: str,
    line_number: int,
    repository_root: Path | None = None,
) -> bool:
    """Return whether *line_number* in *source_path* is an executable source line.

    LLVM LCOV sometimes emits zero-count DA records for documentation comments,
    attributes, pure structural braces, multi-line signatures, Rust multiline
    string continuations, and in-file ``#[cfg(test)]`` modules. Those records
    are not evidence of uncovered production behavior and are excluded from the
    authored-line gate.

    When *repository_root* is provided, *source_path* must resolve under that
    root (same fail-closed rule as LCOV ``SF:`` loading).
    """

    try:
        path = (
            resolve_repository_source_path(source_path, repository_root)
            if repository_root is not None
            else Path(source_path)
        )
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return True
    # Stale LCOV rows past EOF are instrumentation noise, not production gaps.
    if line_number <= 0 or line_number > len(lines):
        return False
    if line_number in _cfg_test_module_line_numbers(lines):
        return False
    if _line_in_cfg_not_feature_block(lines, line_number):
        return False
    if _line_in_multiline_string_literal(lines, line_number):
        return False
    text = lines[line_number - 1].strip()
    if not text:
        return False
    if text.startswith("//"):
        return False
    if text.startswith("#[") or text.startswith("#!["):
        return False
    if text in {"{", "}", "},", ")", ");", "];", "();", "};"}:
        return False
    if _is_standalone_string_literal(text) or text.startswith("} else"):
        return False
    if text.startswith("use ") or text.startswith("pub use "):
        return False
    if text.startswith("mod ") or text.startswith("pub mod "):
        return False
    if text.startswith("impl ") or text.startswith("impl<"):
        return False
    if text.startswith(") ->"):
        return False
    if text.startswith("pub fn ") or text.startswith("fn "):
        return False
    if text.startswith("pub struct ") or text.startswith("struct "):
        return False
    if text.startswith("pub enum ") or text.startswith("enum "):
        return False
    if text.startswith("Ok(Self") or text in {")}", "})", "})"}:
        return False
    if text.endswith(",") and not text.startswith("let ") and not text.startswith("return "):
        return False
    return True


def _cfg_test_module_line_numbers(lines: list[str]) -> set[int]:
    """Return line numbers belonging to any ``#[cfg(test)] mod ... { ... }`` block."""

    test_lines: set[int] = set()
    index = 0
    while index < len(lines):
        if lines[index].strip().startswith("#[cfg(test)]"):
            look = index + 1
            while look < len(lines) and not lines[look].strip():
                look += 1
            if look < len(lines) and lines[look].strip().startswith("mod "):
                depth = 0
                started = False
                cursor = look
                while cursor < len(lines):
                    raw = lines[cursor]
                    depth += raw.count("{") - raw.count("}")
                    if "{" in raw:
                        started = True
                    test_lines.add(cursor + 1)
                    if started and depth <= 0:
                        break
                    cursor += 1
                index = cursor + 1
                continue
        index += 1
    return test_lines


def _line_in_cfg_not_feature_block(lines: list[str], line_number: int) -> bool:
    """Return True when *line_number* is inside ``#[cfg(not(feature = ...))]`` code.

    Workspace CI builds with ``--all-features``, so these inactive alternatives
    must not fail the authored-line gate when LLVM still emits zero DA rows.
    """

    index = 0
    while index < len(lines):
        stripped = lines[index].strip()
        if stripped.startswith("#[cfg(not(feature"):
            depth = 0
            started = False
            cursor = index + 1
            while cursor < len(lines):
                raw = lines[cursor]
                depth += raw.count("{") - raw.count("}")
                if "{" in raw:
                    started = True
                if cursor + 1 == line_number:
                    return True
                if started and depth <= 0:
                    break
                cursor += 1
            index = cursor + 1
            continue
        index += 1
    return False


def _line_in_multiline_string_literal(lines: list[str], line_number: int) -> bool:
    """Return whether a line is inside a Rust normal-string continuation."""
    in_string = False
    in_block_comment = False
    for index, raw in enumerate(lines, start=1):
        if in_string and index == line_number:
            return True
        escaped = False
        cursor = 0
        while cursor < len(raw):
            if in_block_comment:
                if raw.startswith("*/", cursor):
                    in_block_comment = False
                    cursor += 2
                else:
                    cursor += 1
                continue
            if in_string:
                character = raw[cursor]
                if character == '"' and not escaped:
                    in_string = False
                if character == "\\":
                    escaped = not escaped
                else:
                    escaped = False
                cursor += 1
                continue
            if raw.startswith("//", cursor):
                break
            if raw.startswith("/*", cursor):
                in_block_comment = True
                cursor += 2
                continue
            if raw[cursor] == '"':
                in_string = True
            cursor += 1
    return False


def _is_standalone_string_literal(text: str) -> bool:
    """Return whether *text* is only a normal string literal and punctuation."""
    if not text.startswith('"'):
        return False
    escaped = False
    for index, character in enumerate(text[1:], start=1):
        if character == '"' and not escaped:
            return text[index + 1 :].strip() in {"", ",", ";"}
        if character == "\\":
            escaped = not escaped
        else:
            escaped = False
    return False


def load_lcov_line_totals(
    path: Path, repository_root: Path | None = None
) -> Mapping[str, Any]:
    """Load authored source-line totals from a fully framed LLVM LCOV report."""

    root = (repository_root or Path.cwd()).resolve()
    source_path: str | None = None
    line_counts: dict[tuple[str, int], int] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        if raw_line.startswith("SF:"):
            if source_path is not None:
                raise ValueError("LCOV source record must end with end_of_record")
            source_path = raw_line[3:]
            if not source_path:
                raise ValueError("LCOV source path must not be empty")
            # Fail closed on traversal before reading any source file content.
            resolve_repository_source_path(source_path, root)
        elif raw_line.startswith("DA:"):
            if source_path is None:
                raise ValueError("LCOV line record must follow a source record")
            fields = raw_line[3:].split(",")
            if len(fields) < 2:
                raise ValueError("LCOV line record must contain line and count")
            try:
                line_number = int(fields[0])
                execution_count = int(fields[1])
            except ValueError as error:
                raise ValueError("LCOV line and count values must be integers") from error
            if line_number <= 0 or execution_count < 0:
                raise ValueError("LCOV line records contain invalid values")
            if not is_executable_source_line(source_path, line_number, root):
                continue
            key = (source_path, line_number)
            if key in line_counts:
                raise ValueError("LCOV report contains a duplicate source line")
            line_counts[key] = execution_count
        elif raw_line == "end_of_record":
            if source_path is None:
                raise ValueError("LCOV end_of_record must close a source record")
            source_path = None

    if source_path is not None:
        raise ValueError("LCOV source record must end with end_of_record")
    if not line_counts:
        raise ValueError("LCOV report contains no authored source lines")
    covered = sum(execution_count > 0 for execution_count in line_counts.values())
    return {"lines": {"count": len(line_counts), "covered": covered}}


def validate_kind(totals: Mapping[str, Any], kind: str) -> str:
    """Return a stable success message or raise for incomplete *kind* coverage."""

    summary = totals.get(kind)
    if not isinstance(summary, Mapping):
        raise ValueError(f"coverage totals do not contain {kind}")
    count = summary.get("count")
    covered = summary.get("covered")
    if not isinstance(count, int) or not isinstance(covered, int):
        raise ValueError(f"{kind} count and covered values must be integers")
    if count < 0 or covered < 0 or covered > count:
        raise ValueError(f"{kind} coverage counts are invalid")
    if covered != count:
        raise ValueError(f"{kind} coverage is incomplete: {covered}/{count}")
    if count == 0:
        return f"{kind} coverage: PASS (0 executable units in this foundation slice)"
    return f"{kind} coverage: PASS ({covered}/{count}, 100%)"


def validate_report(path: Path, kinds: Sequence[str], report_format: str = "json") -> list[str]:
    """Validate all requested coverage *kinds* in *path*."""

    if report_format == "json":
        totals = load_totals(path)
    elif report_format == "lcov":
        if list(kinds) != ["lines"]:
            raise ValueError("LCOV validation supports exactly the lines kind")
        totals = load_lcov_line_totals(path)
    else:
        raise ValueError(f"unsupported coverage report format: {report_format}")
    return [validate_kind(totals, kind) for kind in kinds]


def build_parser() -> argparse.ArgumentParser:
    """Create the command-line argument parser."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument(
        "--kind",
        action="append",
        choices=("lines", "branches"),
        required=True,
        dest="kinds",
    )
    parser.add_argument(
        "--format",
        choices=("json", "lcov"),
        default="json",
        dest="report_format",
    )
    return parser


def main(arguments: Iterable[str] | None = None) -> int:
    """Validate one LLVM coverage report."""

    parser = build_parser()
    namespace = parser.parse_args(list(arguments) if arguments is not None else None)
    try:
        messages = validate_report(
            namespace.report,
            namespace.kinds,
            namespace.report_format,
        )
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(f"Coverage contract: FAIL: {error}", file=sys.stderr)
        return 1
    for message in messages:
        print(message)
    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
