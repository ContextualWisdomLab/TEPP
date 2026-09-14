"""Enforce AGENTS.md contract 12 on SQL migration object names.

Contract 12 requires every database object name to contain at least two words
and to use ``snake_case``. The rule was documented without an executable gate,
so this checker extracts declared tables, indexes, triggers, functions, types,
views, sequences, policies, constraints, and columns from ``migrations/*.sql``
and reports every name that does not satisfy the contract.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

IDENTIFIER = r"[A-Za-z_][A-Za-z0-9_$]*"
CONTRACT_NAME = re.compile(r"^[a-z][a-z0-9]*(?:_[a-z0-9]+)+$")
DECLARATION = re.compile(
    r"\bCREATE\s+(?:OR\s+REPLACE\s+)?(?:UNIQUE\s+)?"
    r"(TABLE|INDEX|TRIGGER|FUNCTION|TYPE|VIEW|SEQUENCE|POLICY)\s+"
    rf"(?:IF\s+NOT\s+EXISTS\s+)?({IDENTIFIER})",
    re.IGNORECASE,
)
CONSTRAINT_DECLARATION = re.compile(
    rf"\bCONSTRAINT\s+(?:IF\s+(?:NOT\s+)?EXISTS\s+)?({IDENTIFIER})",
    re.IGNORECASE,
)
TABLE_BODY = re.compile(
    rf"\bCREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?{IDENTIFIER}\s*\(",
    re.IGNORECASE,
)
FIRST_WORD = re.compile(rf"\s*({IDENTIFIER})")
TABLE_CONSTRAINT_KEYWORDS = frozenset(
    {"constraint", "primary", "foreign", "unique", "check", "exclude", "like"}
)


def is_contract_name(name: str) -> bool:
    """Return whether ``name`` is multi-word ``snake_case``."""

    return CONTRACT_NAME.match(name) is not None


def _line_number(text: str, offset: int) -> int:
    """Return the one-based line number of ``offset`` within ``text``."""

    return text.count("\n", 0, offset) + 1


def _closing_parenthesis(text: str, open_index: int) -> int:
    """Return the index closing the parenthesis at ``open_index``."""

    depth = 0
    for index in range(open_index, len(text)):
        if text[index] == "(":
            depth += 1
        elif text[index] == ")":
            depth -= 1
            if depth == 0:
                return index
    return len(text)


def _top_level_segments(body: str) -> list[tuple[int, str]]:
    """Split a table body on commas that are not inside parentheses."""

    segments: list[tuple[int, str]] = []
    depth = 0
    start = 0
    for index, character in enumerate(body):
        if character == "(":
            depth += 1
        elif character == ")":
            depth -= 1
        elif character == "," and depth == 0:
            segments.append((start, body[start:index]))
            start = index + 1
    segments.append((start, body[start:]))
    return segments


def declared_names(text: str) -> list[tuple[int, str, str]]:
    """Return sorted ``(line, kind, name)`` triples declared by ``text``."""

    found: list[tuple[int, str, str]] = []
    for match in DECLARATION.finditer(text):
        found.append((_line_number(text, match.start(2)), match.group(1).lower(), match.group(2)))
    for match in CONSTRAINT_DECLARATION.finditer(text):
        found.append((_line_number(text, match.start(1)), "constraint", match.group(1)))
    for match in TABLE_BODY.finditer(text):
        open_index = match.end() - 1
        body = text[open_index + 1 : _closing_parenthesis(text, open_index)]
        for offset, segment in _top_level_segments(body):
            word = FIRST_WORD.match(segment)
            if word is None:
                continue
            if word.group(1).lower() in TABLE_CONSTRAINT_KEYWORDS:
                continue
            position = open_index + 1 + offset + word.start(1)
            found.append((_line_number(text, position), "column", word.group(1)))
    return sorted(found)


def violations(text: str, display_path: Path) -> list[str]:
    """Return one message per name in ``text`` that breaks the contract."""

    return [
        f"{display_path}:{line}: {kind} name '{name}' is not multi-word snake_case"
        for line, kind, name in declared_names(text)
        if not is_contract_name(name)
    ]


def migration_files(repository_root: Path) -> list[Path]:
    """Return the SQL migration files owned by ``repository_root``."""

    return sorted((repository_root / "migrations").glob("*.sql"))


def check_migrations(repository_root: Path) -> list[str]:
    """Return every contract violation found in the migration directory."""

    messages: list[str] = []
    for path in migration_files(repository_root):
        messages.extend(
            violations(path.read_text(encoding="utf-8"), path.relative_to(repository_root))
        )
    return messages


def main(argv: list[str] | None = None) -> int:
    """Run the naming gate and return a process exit status."""

    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("repository_root", nargs="?", default=Path("."), type=Path)
    arguments = parser.parse_args(argv)
    messages = check_migrations(arguments.repository_root)
    for message in messages:
        print(message)
    if messages:
        return 1
    print("database object naming contract satisfied")
    return 0


if __name__ == "__main__":
    sys.exit(main())
