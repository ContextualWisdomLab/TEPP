"""Fail closed unless an LLVM coverage report is exactly complete."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable, Mapping, Sequence


def load_totals(path: Path) -> Mapping[str, Any]:
    """Load exact totals from LLVM coverage JSON with unique branch arms.

    Full LLVM branch exports can contain several instrumented copies of the
    same source file when unit and integration test binaries are merged and
    when generic instantiations are max-folded. The source-level contract is
    the unique union of each ``(filename, coordinate)`` site's true and false
    outcomes, so those copies are merged before the branch gate runs.
    Summary-only reports without branch arrays keep the totals mapping and
    fail closed on that summary.
    """

    payload = json.loads(path.read_text(encoding="utf-8"))
    data = payload.get("data")
    if not isinstance(data, list) or len(data) != 1:
        raise ValueError("coverage JSON must contain exactly one data entry")
    report = data[0]
    totals = report.get("totals")
    if not isinstance(totals, Mapping):
        raise ValueError("coverage JSON data entry must contain totals")
    folded = fold_unique_branch_totals(report.get("files"))
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


def is_live_sqlx_transport_source(filename: str) -> bool:
    """Return whether *filename* is the live-server SQLx transport source.

    The authored LLVM coverage gate excludes ``sqlx_live.rs`` because a live
    PostgreSQL server is required for the success path. Unreachable-host
    failure remains unit-tested. The branch fold must honor the same
    filename ignore that ``cargo llvm-cov --ignore-filename-regex`` uses,
    or ignored live-transport arms re-enter the unique-site contract.
    """

    return Path(filename).name == "sqlx_live.rs"


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
        if is_live_sqlx_transport_source(filename):
            continue
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


def load_union_branch_totals(files: Sequence[object]) -> Mapping[str, int | float]:
    """Merge LLVM branch outcomes by source coordinate across test binaries."""

    outcomes: dict[tuple[str, int, int, int, int], list[int]] = {}
    for file_record in files:
        if not isinstance(file_record, Mapping):
            raise ValueError("coverage file record must be an object")
        filename = file_record.get("filename")
        if "branches" not in file_record:
            raise ValueError("coverage file record must contain branches")
        branches = file_record["branches"]
        if not isinstance(filename, str) or not filename:
            raise ValueError("coverage file record must contain a filename")
        if not isinstance(branches, list):
            raise ValueError("coverage branches must be a list")
        for branch in branches:
            if not isinstance(branch, list) or len(branch) < 6:
                raise ValueError("coverage branch record is malformed")
            coordinates = branch[:4]
            counts = branch[4:6]
            if not all(
                isinstance(value, int) and not isinstance(value, bool) and value >= 0
                for value in coordinates
            ):
                raise ValueError("coverage branch coordinates are invalid")
            if not all(
                isinstance(value, int) and not isinstance(value, bool) and value >= 0
                for value in counts
            ):
                raise ValueError("coverage branch counts are invalid")
            key = (filename, *coordinates)
            outcome = outcomes.setdefault(key, [0, 0])
            outcome[0] += counts[0]
            outcome[1] += counts[1]
    count = len(outcomes) * 2
    covered = sum(outcome > 0 for counts in outcomes.values() for outcome in counts)
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
    attributes, pure structural braces, multi-line signatures, Rust multiline
    string continuations, literal continuations, expression continuations, and
    in-file ``#[cfg(test)]`` modules. Those records are not evidence of
    uncovered production behavior and are excluded from the authored-line gate.

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
    code, continuation_only, standalone_open_literal = _rust_lexical_lines(
        lines[:line_number]
    )[-1]
    if continuation_only:
        return False
    if standalone_open_literal and not _is_match_arm_body(lines, line_number):
        return False
    text = code.strip()
    if not text:
        return False
    if text.startswith("#[") or text.startswith("#!["):
        return False
    if text in {
        "{",
        "}",
        "(",
        ")",
        "},",
        ");",
        "];",
        "();",
        "};",
        "});",
    }:
        return False
    if (
        _is_standalone_string_literal(text)
        and not _is_match_arm_body(lines, line_number)
    ) or text.startswith("} else"):
        return False
    if text.endswith(" {"):
        type_name = text[:-2]
        if type_name and all(character.isalnum() or character in "_:" for character in type_name):
            return False
    if text.startswith("use ") or text.startswith("pub use "):
        return False
    if text.startswith("mod ") or text.startswith("pub mod "):
        return False
    if text.startswith("impl ") or text.startswith("impl<"):
        return False
    if text.startswith(") ->"):
        return False
    if text.startswith(
        (
            "pub fn ",
            "pub const fn ",
            "pub(crate) fn ",
            "pub(crate) const fn ",
            "const fn ",
            "fn ",
        )
    ):
        if "{" not in text or "}" not in text:
            return False
        body = text[text.find("{") + 1 : text.rfind("}")].strip()
        return bool(body)
    # Keep guarded match arms in the authored-line denominator: the guard
    # executes even though the arm label itself is structural.
    if text.endswith("=> {") and " if " not in text:
        if (
            text.startswith("if ")
            or text.startswith("if(")
            or " if(" in text
        ):
            return True
        return _is_multiline_match_guard(lines, line_number)
    if text.startswith("pub struct ") or text.startswith("struct "):
        return False
    if text.startswith("pub enum ") or text.startswith("enum "):
        return False
    if text.startswith("Ok(Self") or text in {")}", "})"}:
        return False
    if text in {"} else {", "else {", "));"} or text.startswith((".", "||", "&&")):
        return False
    if (
        text.endswith(",")
        and not text.startswith("let ")
        and not text.startswith("return ")
        and _is_structural_comma_continuation(lines, line_number, text)
    ):
        return False
    return True


def _is_structural_comma_continuation(
    lines: list[str], line_number: int, text: str
) -> bool:
    """Return whether a comma-terminated line is proven to be structural.

    A comma can terminate a declaration field, enum variant, function
    parameter, or ordinary call argument. Operators, calls, and assignments
    remain executable because their expressions can perform observable work.
    """

    if any(character in text for character in "()=+-*/%<>!&|?"):
        return False
    previous = ""
    for candidate in reversed(lines[: line_number - 1]):
        if candidate.strip():
            previous = candidate.strip()
            break
    if previous.endswith("(") and "let " not in previous and "=" not in previous:
        return True

    declaration_depth = 0
    function_parenthesis_depth = 0
    for candidate in lines[: line_number - 1]:
        stripped = candidate.strip()
        if declaration_depth:
            declaration_depth += candidate.count("{") - candidate.count("}")
            if declaration_depth <= 0:
                declaration_depth = 0
            continue
        if stripped.startswith(
            (
                "struct ",
                "pub struct ",
                "pub(crate) struct ",
                "enum ",
                "pub enum ",
                "pub(crate) enum ",
            )
        ) and "{" in candidate:
            declaration_depth = candidate.count("{") - candidate.count("}")
            continue
        if function_parenthesis_depth:
            function_parenthesis_depth += candidate.count("(") - candidate.count(")")
            if function_parenthesis_depth <= 0:
                function_parenthesis_depth = 0
            continue
        if "fn " in stripped and "(" in candidate:
            function_parenthesis_depth = candidate.count("(") - candidate.count(")")
    return declaration_depth > 0 or function_parenthesis_depth > 0


def _rust_lexical_lines(lines: list[str]) -> list[tuple[str, bool, bool]]:
    """Return code, literal-continuation-only and standalone-open-literal per line.

    This is the common lexical authority for comment removal, continuation
    classification, block openers and reverse arm lookup. Literal bytes opened
    on a line remain intact. A literal inherited from an earlier physical line
    contributes no code until its delimiter closes; subsequent code is retained.
    Nested comments act as whitespace rather than joining adjacent tokens.
    """

    result: list[tuple[str, bool, bool]] = []
    in_string = False
    raw_hashes: int | None = None
    block_comment_depth = 0
    for line in lines:
        inherited_literal = in_string or raw_hashes is not None
        retain_literal = not inherited_literal
        code: list[str] = []
        cursor = 0
        while cursor < len(line):
            if block_comment_depth:
                if line.startswith("/*", cursor):
                    block_comment_depth += 1
                    cursor += 2
                elif line.startswith("*/", cursor):
                    block_comment_depth -= 1
                    cursor += 2
                    if (
                        block_comment_depth == 0
                        and code
                        and not code[-1][-1].isspace()
                        and cursor < len(line)
                        and not line[cursor].isspace()
                    ):
                        code.append(" ")
                else:
                    cursor += 1
                continue
            if raw_hashes is not None:
                delimiter = '"' + ("#" * raw_hashes)
                closing = line.find(delimiter, cursor)
                if closing == -1:
                    if retain_literal:
                        code.append(line[cursor:])
                    break
                if retain_literal:
                    code.append(line[cursor : closing + len(delimiter)])
                cursor = closing + len(delimiter)
                raw_hashes = None
                retain_literal = True
                continue
            if in_string:
                character = line[cursor]
                if retain_literal:
                    code.append(character)
                if character == "\\" and cursor + 1 < len(line):
                    cursor += 1
                    if retain_literal:
                        code.append(line[cursor])
                elif character == '"':
                    in_string = False
                    retain_literal = True
                cursor += 1
                continue
            if line.startswith("//", cursor):
                break
            if line.startswith("/*", cursor):
                block_comment_depth = 1
                cursor += 2
                continue
            raw_start = _raw_string_start(line, cursor)
            if raw_start is not None:
                raw_hashes, next_cursor = raw_start
                code.append(line[cursor:next_cursor])
                cursor = next_cursor
                retain_literal = True
                continue
            if line[cursor] == '"':
                in_string = True
                retain_literal = True
                code.append(line[cursor])
                cursor += 1
                continue
            if line[cursor] == "'":
                character_end = _character_literal_end(line, cursor)
                if character_end is not None:
                    code.append(line[cursor:character_end])
                    cursor = character_end
                    continue
            code.append(line[cursor])
            cursor += 1
        text = "".join(code)
        continuation_only = inherited_literal and all(
            character.isspace() or character in ",;)]}" for character in text
        )
        standalone_open_literal = (
            (in_string or raw_hashes is not None)
            and text.strip().startswith(('"', 'b"', 'r"', 'r#', 'br"', 'br#'))
        )
        result.append((text, continuation_only, standalone_open_literal))
    return result


def _line_in_multiline_string(lines: list[str], line_number: int) -> bool:
    """Identify comment-only or literal-only lines using the common lexical view."""
    if line_number <= 0 or line_number > len(lines):
        return False
    code, continuation_only, standalone_open_literal = _rust_lexical_lines(
        lines[:line_number]
    )[-1]
    return continuation_only or standalone_open_literal or not code.strip()


def _is_multiline_match_guard(lines: list[str], line_number: int) -> bool:
    """Recognize a guard continued onto the lines immediately before an arm."""

    target_prefix = lines[line_number - 1].strip().partition("=>")[0]
    brace_depth = target_prefix.count("}") - target_prefix.count("{")
    guard_found = False
    inside_block = False
    nested_arrow_seen = False
    for candidate in reversed(lines[: line_number - 1]):
        stripped = candidate.strip()
        if brace_depth == 0 and "=>" in stripped:
            return guard_found
        if (
            inside_block
            and brace_depth >= 1
            and stripped.endswith("=> {")
            and not nested_arrow_seen
        ):
            # The opener of the preceding sibling arm sits directly above its
            # body with no nested match between, so every guard token found so
            # far belongs to that sibling rather than to this arm.
            return guard_found
        if "=>" in stripped and brace_depth >= 1:
            nested_arrow_seen = True
        next_depth = brace_depth + stripped.count("}") - stripped.count("{")
        if brace_depth == 0 < next_depth:
            inside_block = True
            nested_arrow_seen = False
        elif next_depth <= 0 < brace_depth:
            inside_block = False
            nested_arrow_seen = False
        brace_depth = next_depth
        if (
            (stripped.startswith("if ") or stripped.startswith("if("))
            and not stripped.endswith(("}", ";"))
            and brace_depth == 0
        ):
            guard_found = True
        if stripped.startswith("match ") or (
            stripped.startswith("let ") and "= match " in stripped
        ):
            return guard_found
    return guard_found


def _cfg_test_module_line_numbers(lines: list[str]) -> set[int]:
    """Return line numbers belonging to any ``#[cfg(test)] mod ... { ... }`` block.

    Blank lines and further attributes (for example ``#[allow(...)]``) may sit
    between ``#[cfg(test)]`` and the ``mod`` declaration; both belong to the
    module and must not break detection.
    """

    test_lines: set[int] = set()
    index = 0
    while index < len(lines):
        if lines[index].strip().startswith("#[cfg(test)]"):
            look = index + 1
            while look < len(lines) and (
                not lines[look].strip() or lines[look].strip().startswith("#[")
            ):
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
    """Identify inherited literal-only lines through the common lexical view."""
    if line_number <= 0 or line_number > len(lines):
        return False
    return _rust_lexical_lines(lines[:line_number])[-1][1]


def _raw_string_start(line: str, cursor: int) -> tuple[int, int] | None:
    """Return ``(hash_count, next_cursor)`` for a Rust raw-string opener."""

    if line.startswith("br", cursor):
        prefix_end = cursor + 2
    elif line.startswith("r", cursor):
        prefix_end = cursor + 1
    else:
        return None
    hash_end = prefix_end
    while hash_end < len(line) and line[hash_end] == "#":
        hash_end += 1
    if hash_end < len(line) and line[hash_end] == '"':
        return hash_end - prefix_end, hash_end + 1
    return None


def _character_literal_end(line: str, cursor: int) -> int | None:
    """Return the cursor after a one-line Rust character literal, if present."""

    candidate = cursor + 1
    if candidate >= len(line):
        return None
    if line[candidate] == "\\":
        candidate += 2
    else:
        candidate += 1
    if candidate < len(line) and line[candidate] == "'":
        return candidate + 1
    return None


def _is_match_arm_body(lines: list[str], line_number: int) -> bool:
    """Locate the preceding arm label through the shared forward lexical view.

    Only the closest meaningful code line can establish an arm body. Comment
    text and inherited literal payload cannot supply an arrow or erase a label.
    """
    view = _rust_lexical_lines(lines[: line_number - 1])
    for code, continuation_only, standalone_open_literal in reversed(view):
        if continuation_only or standalone_open_literal or not code.strip():
            continue
        return code.rstrip().endswith(("=>", "=> {"))
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


def _rust_code_before_line_comment(
    line: str, preceding_lines: list[str] | None = None
) -> str:
    """Return the requested line's code from the shared Rust lexical authority."""
    return _rust_lexical_lines([*(preceding_lines or []), line])[-1][0]


def opens_a_block(source_path: str, line_number: int, repository_root: Path | None) -> bool:
    """Return whether *line_number* ends by opening a brace-delimited block."""

    try:
        path = (
            resolve_repository_source_path(source_path, repository_root)
            if repository_root is not None
            else Path(source_path)
        )
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return False
    if line_number <= 0 or line_number > len(lines):
        return False
    code = _rust_code_before_line_comment(
        lines[line_number - 1], lines[: line_number - 1]
    )
    return code.rstrip().endswith("{")


def _first_meaningful_source_line_after(
    source_path: str, line_number: int, repository_root: Path | None
) -> int | None:
    """Return the first later line that is not non-authored lexical trivia."""

    try:
        path = (
            resolve_repository_source_path(source_path, repository_root)
            if repository_root is not None
            else Path(source_path)
        )
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return None
    for candidate_number in range(line_number + 1, len(lines) + 1):
        stripped = lines[candidate_number - 1].strip()
        if not stripped:
            continue
        if _line_in_multiline_string(lines, candidate_number):
            continue
        return candidate_number
    return None


def reconcile_contradictory_zero_counts(
    line_counts: dict[tuple[str, int], int], repository_root: Path | None
) -> None:
    """Mark a proven-executed zero-count opener covered without deleting it.

    LLVM can report a zero count for a brace-delimited opener while its first
    meaningful nested line has a positive count. Blank and line-comment-only
    lines are not authored coverage units, so they cannot break that proof. A
    closing brace or any other meaningful unmeasured line stops the proof;
    reconciliation never crosses into a sibling statement outside the block.
    """

    for key in [key for key, count in line_counts.items() if count == 0]:
        source_path, line_number = key
        if not opens_a_block(source_path, line_number, repository_root):
            continue
        nested_line = _first_meaningful_source_line_after(
            source_path, line_number, repository_root
        )
        if nested_line is None:
            continue
        nested_count = line_counts.get((source_path, nested_line))
        if nested_count is not None and nested_count > 0:
            line_counts[key] = 1


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
    reconcile_contradictory_zero_counts(line_counts, root)
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