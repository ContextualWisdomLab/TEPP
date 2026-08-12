from pathlib import Path

import pytest

from scripts.check_coverage import load_lcov_line_totals


def test_lcov_requires_end_of_record(tmp_path: Path) -> None:
    report = tmp_path / "coverage.lcov"
    report.write_text("SF:src/lib.rs\nDA:1,1\n", encoding="utf-8")

    with pytest.raises(ValueError, match="end_of_record"):
        load_lcov_line_totals(report)
