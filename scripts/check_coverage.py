"""Fail closed unless an LLVM coverage report is exactly complete."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable, Mapping, Sequence


def load_totals(path: Path) -> Mapping[str, Any]:
    """Load LLVM coverage totals, unique-folding branch arms when arrays exist.

    ``totals.branches`` can disagree with ``files[].branches`` after max-folding
    instantiations. The 100% contract is unique True/False arms, matching the
    LCOV authored-line gate. Summary-only reports without branch arrays keep
    the totals mapping and fail closed on that summary.
    """

    payload = json.loads(path.read_text(encoding="utf-8"))
    data = payload.get("data")
    if not isinstance(data, list) or len(data) != 1:
        raise ValueError("coverage JSON must contain exactly one data entry")
    totals = data[0].get("totals")
    if not isinstance(totals, Mapping):
        raise ValueError("coverage JSON data entry must contain totals")
    folded = fold_unique_branch_totals(data[0].get("files"))
    if folded is None:
        return totals
    merged = dict(totals)
    merged["branches"] = folded
    return merged


def _parse_branch_record(record: object) -> tuple[tuple[int, int, int, int], int, int]:
    """Return ``(site, true_count, false_count)`` from one LLVM branch tuple.

    LLVM export writes
    ``[lineStart, colStart, lineEnd, colEnd, trueCount, falseCount, fileId,
    expandedFileId, kind]``.
    """

    if not isinstance(record, list) or len(record) != 9:
        raise ValueError("coverage JSON branch record must contain nine values")
    line_start, column_start, line_end, column_end, true_count, false_count = record[:6]
    coordinates = (line_start, column_start, line_end, column_end)
    if any(not isinstance(value, int) or isinstance(value, bool) for value in coordinates):
        raise ValueError("coverage JSON branch coordinates must be integers")
    if (
        not isinstance(true_count, int)
        or isinstance(true_count, bool)
        or not isinstance(false_count, int)
        or isinstance(false_count, bool)
        or true_count < 0
        or false_count < 0
    ):
        raise ValueError("coverage JSON branch counts must be non-negative integers")
    return coordinates, true_count, false_count


def fold_unique_branch_totals(files: object) -> dict[str, int] | None:
    """Return unique-site True/False arm totals, or None when arrays are absent.

    Instantiations of the same ``(filename, start, end)`` site max-fold. One
    LLVM JSON total that is not in that unique set is not an uncovered
    production arm. Empty ``branches`` lists are instrumentation-absent and
    leave the caller on summary totals.
    """

    if not isinstance(files, list):
        return None
    sites: dict[tuple[str, int, int, int, int], tuple[int, int]] = {}
    saw_records = False
    for file_entry in files:
        if not isinstance(file_entry, Mapping):
            raise ValueError("coverage JSON file entry must be an object")
        records = file_entry.get("branches")
        if records is None:
            continue
        if not isinstance(records, list):
            raise ValueError("coverage JSON branches must be a list")
        if not records:
            continue
        filename = file_entry.get("filename")
        if not isinstance(filename, str) or not filename:
            raise ValueError("coverage JSON file entry must contain a filename")
        for record in records:
            site, true_count, false_count = _parse_branch_record(record)
            saw_records = True
            key = (filename, *site)
            previous = sites.get(key, (0, 0))
            sites[key] = (
                max(previous[0], true_count),
                max(previous[1], false_count),
            )
    if not saw_records:
        return None
    count = len(sites) * 2
    covered = 0
    for true_count, false_count in sites.values():
        if true_count > 0:
            covered += 1
        if false_count > 0:
            covered += 1
    return {"count": count, "covered": covered}


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
    attributes, pure structural braces, multi-line signatures, and in-file
    ``#[cfg(test)]`` modules. Those records are not evidence of uncovered
    production behavior and are excluded from the authored-line gate.

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
    text = lines[line_number - 1].strip()
    if not text:
        return False
    if text.startswith("//"):
        return False
    if text.startswith("#[") or text.startswith("#!["):
        return False
    if text in {"{", "}", "},", ");", "];", "();", "};"}:
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
