"""In-process ``__main__`` entrypoint coverage for owned Python tooling (#493).

Every script listed under ``[run] source`` in ``.coveragerc`` is executed
through ``runpy.run_module(..., run_name="__main__")`` so the real
``if __name__ == "__main__"`` boundary is measured instead of being removed
from the coverage denominator with ``# pragma: no cover``.
"""

from __future__ import annotations

import configparser
import contextlib
import io
import runpy
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
COVERAGE_CONFIG = REPOSITORY_ROOT / ".coveragerc"


def covered_production_modules() -> list[str]:
    """Return the dotted module names measured by ``.coveragerc``."""

    parser = configparser.ConfigParser()
    parser.read(COVERAGE_CONFIG, encoding="utf-8")
    return [line for line in parser["run"]["source"].splitlines() if line.strip()]


def run_entrypoint(module_name: str, argv: list[str]) -> int:
    """Execute ``module_name`` as ``__main__`` and return its exit status."""

    loaded_module = sys.modules.pop(module_name, None)
    stdout, stderr = io.StringIO(), io.StringIO()
    try:
        with mock.patch("sys.argv", [module_name.rsplit(".", 1)[-1] + ".py", *argv]):
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                with unittest.TestCase().assertRaises(SystemExit) as raised:
                    runpy.run_module(module_name, run_name="__main__")
    finally:
        if loaded_module is not None:
            sys.modules[module_name] = loaded_module
    return int(raised.exception.code)


class CoveragePolicyTests(unittest.TestCase):
    """Owned production tooling must not rely on pragma-based suppression."""

    def test_coveragerc_has_no_pragma_exclusion(self) -> None:
        parser = configparser.ConfigParser()
        parser.read(COVERAGE_CONFIG, encoding="utf-8")
        self.assertEqual(parser["report"].get("fail_under"), "100")
        self.assertEqual(parser["run"].get("branch"), "True")
        self.assertNotIn("exclude_lines", parser["report"])

    def test_covered_sources_have_no_pragma_no_cover(self) -> None:
        modules = covered_production_modules()
        self.assertGreaterEqual(len(modules), 6)
        for module_name in modules:
            source_path = REPOSITORY_ROOT / (module_name.replace(".", "/") + ".py")
            with self.subTest(module=module_name):
                self.assertFalse(
                    "pragma: no cover" in source_path.read_text(encoding="utf-8"),
                    f"{source_path} relies on pragma-based coverage suppression",
                )


class ModuleEntrypointTests(unittest.TestCase):
    """Each covered script's ``__main__`` guard is exercised in-process."""

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.addCleanup(self._tmp.cleanup)

    def test_check_docstrings_entrypoint(self) -> None:
        self.assertEqual(
            run_entrypoint("scripts.check_docstrings", [str(REPOSITORY_ROOT)]), 0
        )

    def test_check_workspace_contract_entrypoint(self) -> None:
        self.assertEqual(
            run_entrypoint("scripts.check_workspace_contract", [str(REPOSITORY_ROOT)]),
            0,
        )

    def test_check_coverage_entrypoint_reports_missing_report(self) -> None:
        missing = self.root / "missing.lcov"
        self.assertEqual(
            run_entrypoint(
                "scripts.check_coverage",
                [str(missing), "--kind", "lines", "--format", "lcov"],
            ),
            1,
        )

    def test_release_evidence_entrypoint_reports_missing_bundle(self) -> None:
        self.assertEqual(
            run_entrypoint(
                "scripts.release_evidence",
                [
                    "validate",
                    "--evidence-directory",
                    str(self.root / "absent"),
                    "--repository-root",
                    str(REPOSITORY_ROOT),
                ],
            ),
            1,
        )

    def test_actions_workflow_fleet_entrypoint_requires_token(self) -> None:
        with mock.patch.dict("os.environ", {}, clear=True):
            self.assertEqual(
                run_entrypoint(
                    "scripts.actions_workflow_fleet",
                    ["audit", "--owner", "cwl", "--repo", "tepp"],
                ),
                2,
            )


if __name__ == "__main__":
    unittest.main()
