"""Restore and verify the exact loopback I/O deadline assertion for PR 159."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "crates/tepp_api/src/analysis_run_live.rs"


def _run(*args: str) -> None:
    """Run one repository command and surface captured output on failure."""

    completed = subprocess.run(
        args,
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    if completed.stdout:
        print(completed.stdout, end="")
    if completed.stderr:
        print(completed.stderr, end="", file=sys.stderr)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def _replace_once(text: str, old: str, new: str, *, label: str) -> str:
    """Replace one reviewed fragment or fail closed when the branch moved."""

    if new in text:
        return text
    if text.count(old) != 1:
        raise SystemExit(f"refusing unknown {label} shape")
    return text.replace(old, new, 1)


def main() -> None:
    """Restore the deadline observation and prove the exact contract test."""

    text = TARGET.read_text(encoding="utf-8")
    text = _replace_once(
        text,
        "    use std::time::Duration;\n",
        "    use std::time::{Duration, Instant};\n",
        label="test time import",
    )
    text = _replace_once(
        text,
        "        NARUON_LIVE_HEADER_COUNT_LIMIT,\n",
        "        NARUON_LIVE_HEADER_COUNT_LIMIT, NARUON_LIVE_IO_TIMEOUT,\n",
        label="timeout constant import",
    )
    text = _replace_once(
        text,
        '''        let stream = TcpStream::connect(timeout_addr).expect("timeout connect");
        let timeout_response = timeout_worker
''',
        '''        let stream = TcpStream::connect(timeout_addr).expect("timeout connect");
        let started = Instant::now();
        let timeout_response = timeout_worker
''',
        label="timeout start observation",
    )
    text = _replace_once(
        text,
        '''        drop(stream);
        assert_eq!(timeout_response.status_code, 413);
''',
        '''        drop(stream);
        assert!(started.elapsed() >= NARUON_LIVE_IO_TIMEOUT);
        assert_eq!(timeout_response.status_code, 413);
''',
        label="timeout deadline assertion",
    )
    TARGET.write_text(text, encoding="utf-8")
    _run("cargo", "fmt", "--check")
    _run(
        "cargo",
        "test",
        "-p",
        "tepp_api",
        "serve_one_covers_loopback_success_disconnect_and_timeout",
        "--",
        "--exact",
    )


if __name__ == "__main__":
    main()
