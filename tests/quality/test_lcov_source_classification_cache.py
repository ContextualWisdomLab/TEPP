"""Regression contract for source-local LCOV classification work."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import check_coverage as coverage_contract


class LcovSourceClassificationCacheTests(unittest.TestCase):
    """Keep authored-line classification linear in source records, not DA rows."""

    def test_each_lcov_source_is_read_once_per_report(self) -> None:
        """Repeated DA rows for one source must reuse one source snapshot."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "sample.rs"
            source.write_text(
                "fn sample() {\n"
                "    first();\n"
                "    second();\n"
                "    third();\n"
                "}\n",
                encoding="utf-8",
            )
            report = root / "coverage.lcov"
            report.write_text(
                f"SF:{source}\n"
                "DA:2,1\n"
                "DA:3,1\n"
                "DA:4,1\n"
                "end_of_record\n",
                encoding="utf-8",
            )

            original_read_text = Path.read_text
            source_reads = 0

            def counted_read_text(path: Path, *args: object, **kwargs: object) -> str:
                nonlocal source_reads
                if path == source:
                    source_reads += 1
                return original_read_text(path, *args, **kwargs)

            with mock.patch.object(Path, "read_text", counted_read_text):
                self.assertEqual(
                    coverage_contract.load_lcov_line_totals(report, repository_root=root),
                    {"lines": {"count": 3, "covered": 3}},
                )

            self.assertEqual(
                source_reads,
                1,
                "source classification must not reread the same file for every DA row",
            )


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
