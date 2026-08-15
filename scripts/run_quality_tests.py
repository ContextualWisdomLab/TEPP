"""Execute repository quality tests for CI coverage reporting."""

from __future__ import annotations

import sys
from pathlib import Path
import unittest


def run() -> int:
    """Run quality tests and return an integer CLI exit code."""

    repository_root = Path(__file__).resolve().parents[1]
    if str(repository_root) not in sys.path:
        sys.path.insert(0, str(repository_root))

    loader = unittest.TestLoader()
    suite = loader.discover("tests/quality", pattern="test_*.py")
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    return 0 if result.wasSuccessful() else 1


def main(argv: list[str] | None = None) -> int:
    """Program entrypoint."""

    del argv  # kept for CLI compatibility with existing quality scripts.
    return run()


if __name__ == "__main__":
    raise SystemExit(main(None))
