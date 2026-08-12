"""Safely parse untrusted OpenCode pull-request metadata for a trusted publisher."""

from __future__ import annotations

import argparse
import os
import stat
import sys
import unicodedata
from collections.abc import Sequence
from pathlib import Path

_BIDIRECTIONAL_CONTROLS = frozenset(
    {
        "\u061c",
        "\u200e",
        "\u200f",
        "\u202a",
        "\u202b",
        "\u202c",
        "\u202d",
        "\u202e",
        "\u2066",
        "\u2067",
        "\u2068",
        "\u2069",
    }
)


def _positive_limit(value: int, field_name: str) -> int:
    """Return a positive integer limit or raise a stable validation error."""
    if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
        raise ValueError(f"{field_name} must be a positive integer")
    return value


def _read_regular_file(path: Path, maximum_bytes: int) -> bytes:
    """Read a stable regular file without following a symbolic link."""
    try:
        before = path.lstat()
    except OSError as exc:
        raise ValueError("PR message must be a readable regular file") from exc
    if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode):
        raise ValueError("PR message must be a regular file")

    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
    except OSError as exc:
        raise ValueError("PR message must be a readable regular file") from exc

    try:
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode):
            raise ValueError("PR message must be a regular file")
        if (before.st_dev, before.st_ino) != (opened.st_dev, opened.st_ino):
            raise ValueError("PR message file identity changed while opening")
        with os.fdopen(descriptor, "rb", closefd=True) as handle:
            descriptor = -1
            payload = handle.read(maximum_bytes + 1)
    finally:
        if descriptor >= 0:
            os.close(descriptor)

    if len(payload) > maximum_bytes:
        raise ValueError("PR message exceeds the combined byte budget")
    return payload


def _reject_unsafe_text(value: str) -> None:
    """Reject control and bidirectional characters that can spoof metadata."""
    for character in value:
        if character in _BIDIRECTIONAL_CONTROLS:
            raise ValueError("PR message contains a bidirectional control character")
        if unicodedata.category(character) == "Cc" and character not in {"\n", "\t"}:
            raise ValueError("PR message contains an unsupported control character")


def _write_private_text(path: Path, value: str) -> None:
    """Replace one UTF-8 output file with owner-only permissions."""
    path.parent.mkdir(parents=True, exist_ok=True)
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC
    descriptor = os.open(path, flags, 0o600)
    try:
        os.fchmod(descriptor, 0o600)
        with os.fdopen(
            descriptor,
            "w",
            encoding="utf-8",
            newline="\n",
            closefd=True,
        ) as handle:
            descriptor = -1
            handle.write(value)
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def parse_pr_message(
    source_path: Path,
    title_path: Path,
    body_path: Path,
    *,
    max_title_bytes: int,
    max_body_bytes: int,
) -> tuple[str, str]:
    """Validate one model-authored PR message and write bounded trusted outputs.

    The first line is the title.  The remaining lines are the body; one optional
    blank separator line is removed.  The parser validates strict UTF-8, byte
    budgets, stable regular-file identity, control characters, and bidirectional
    spoofing before any credential-bearing publication step consumes the text.
    """
    title_limit = _positive_limit(max_title_bytes, "max_title_bytes")
    body_limit = _positive_limit(max_body_bytes, "max_body_bytes")
    maximum_source_bytes = title_limit + body_limit + 4
    payload = _read_regular_file(Path(source_path), maximum_source_bytes)

    try:
        text = payload.decode("utf-8", errors="strict")
    except UnicodeDecodeError as exc:
        raise ValueError("PR message must be valid UTF-8") from exc
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    _reject_unsafe_text(text)

    lines = text.split("\n")
    title = lines[0].strip()
    body_lines = lines[1:]
    if body_lines and not body_lines[0].strip():
        body_lines = body_lines[1:]
    body = "\n".join(body_lines).strip()

    if not title:
        raise ValueError("PR message title must not be empty")
    if not body:
        raise ValueError("PR message body must not be empty")
    if len(title.encode("utf-8")) > title_limit:
        raise ValueError("PR message title exceeds the UTF-8 byte budget")
    if len(body.encode("utf-8")) > body_limit:
        raise ValueError("PR message body exceeds the UTF-8 byte budget")

    _write_private_text(Path(title_path), title)
    _write_private_text(Path(body_path), body)
    return title, body


def _parser() -> argparse.ArgumentParser:
    """Build the command-line parser for the trusted publication boundary."""
    parser = argparse.ArgumentParser(
        description="Validate an untrusted PR_MESSAGE.md and write trusted outputs."
    )
    parser.add_argument("source_path", type=Path)
    parser.add_argument("title_path", type=Path)
    parser.add_argument("body_path", type=Path)
    parser.add_argument(
        "--max-title-bytes",
        type=int,
        default=int(os.environ.get("MAX_PR_TITLE_BYTES", "120")),
    )
    parser.add_argument(
        "--max-body-bytes",
        type=int,
        default=int(os.environ.get("MAX_PR_BODY_BYTES", "20000")),
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    """Parse CLI arguments, emit trusted files, and return a process status."""
    arguments = _parser().parse_args(argv)
    try:
        parse_pr_message(
            arguments.source_path,
            arguments.title_path,
            arguments.body_path,
            max_title_bytes=arguments.max_title_bytes,
            max_body_bytes=arguments.max_body_bytes,
        )
    except ValueError as exc:
        print(f"metadata validation failed: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
