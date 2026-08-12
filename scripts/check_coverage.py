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


def load_lcov_line_totals(path: Path) -> Mapping[str, Any]:
    """Load authored source-line totals from a fully framed LLVM LCOV report."""

    source_path: str | None = None
    line_counts: dict[tuple[str, int], int] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        if raw_line.startswith("SF:"):
            if source_path is not None:
                raise ValueError("LCOV source record must end with end_of_record")
            source_path = raw_line[3:]
            if not source_path:
                raise ValueError("LCOV source path must not be empty")
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
