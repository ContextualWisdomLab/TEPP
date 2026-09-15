"""Keep the README crate count bound to the workspace (GAP-014)."""

from __future__ import annotations

import re
import unittest
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
CRATE_COUNT_CLAIM = re.compile(r"(\d+) independently documented (?:Rust )?crates")


class ReadmeCrateCountTests(unittest.TestCase):
    """Every counted claim in the README must match `crates/`."""

    def test_readme_counts_match_the_workspace(self) -> None:
        crates = sorted(
            path.name
            for path in (REPOSITORY_ROOT / "crates").iterdir()
            if (path / "Cargo.toml").is_file()
        )
        readme = (REPOSITORY_ROOT / "README.md").read_text(encoding="utf-8")
        claims = [int(match) for match in CRATE_COUNT_CLAIM.findall(readme)]
        self.assertGreaterEqual(len(claims), 2, "the README must state the crate count")
        for claim in claims:
            with self.subTest(claim=claim):
                self.assertEqual(claim, len(crates))


if __name__ == "__main__":
    unittest.main()
