#!/usr/bin/env python3
"""Stage TEPP's production adaptive-orchestration contract."""

from pathlib import Path

root = Path(__file__).resolve().parents[1]
test_path = root / "tests" / "test_contextual_orchestrator_default_policy.py"
content = '''"""TEPP production contextual-orchestrator requests explicitly use auto."""

from __future__ import annotations

import re
import unittest
from pathlib import Path

SOURCE_SUFFIXES = {".py", ".js", ".mjs", ".ts", ".tsx", ".rs", ".go"}
EXCLUDED = {
    ".git", ".github", "docs", "examples", "fixtures", "migrations",
    "node_modules", "scripts", "target", "test", "tests", "vendor",
}
FORCED_ROUTE = re.compile(
    r"(?:orchestration_mode|mode)(?:\\s*:\\s*str)?\\s*[:=]\\s*[\\\"']route[\\\"']",
    re.IGNORECASE,
)
AUTO = re.compile(
    r"(?:orchestration_mode|mode)(?:\\s*:\\s*str)?\\s*[:=]\\s*[\\\"']auto[\\\"']",
    re.IGNORECASE,
)


class AdaptiveOrchestratorDefaultTest(unittest.TestCase):
    def test_production_integrations_are_explicitly_adaptive(self) -> None:
        root = Path(__file__).resolve().parents[1]
        integration_files: list[str] = []
        violations: list[str] = []
        for path in sorted(root.rglob("*")):
            if not path.is_file() or path.suffix.lower() not in SOURCE_SUFFIXES:
                continue
            relative = path.relative_to(root)
            if {part.lower() for part in relative.parts} & EXCLUDED:
                continue
            text = path.read_text(encoding="utf-8")
            lowered = text.lower()
            if "contextual-orchestrator" not in lowered and "contextual_orchestrator" not in lowered:
                continue
            integration_files.append(relative.as_posix())
            if FORCED_ROUTE.search(text):
                violations.append(f"{relative}: forced route")
            if "chat/completions" in lowered and "contextual-orchestrator" in lowered and not AUTO.search(text):
                violations.append(f"{relative}: implicit mode")
        self.assertTrue(integration_files, "no production contextual-orchestrator integration was found")
        self.assertEqual(violations, [])


if __name__ == "__main__":
    unittest.main()
'''
if test_path.exists():
    if test_path.read_text(encoding="utf-8") != content:
        raise SystemExit(f"refusing to replace a different test: {test_path}")
else:
    test_path.parent.mkdir(parents=True, exist_ok=True)
    test_path.write_text(content, encoding="utf-8")
