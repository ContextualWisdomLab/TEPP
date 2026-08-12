import tempfile
import unittest
from pathlib import Path

from scripts.check_coverage import load_lcov_line_totals


class LcovRecordContractTest(unittest.TestCase):
    """Exercise fail-closed LCOV source-record framing."""

    def test_lcov_requires_end_of_record(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "coverage.lcov"
            report.write_text("SF:src/lib.rs\nDA:1,1\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "end_of_record"):
                load_lcov_line_totals(report)


if __name__ == "__main__":
    unittest.main()
