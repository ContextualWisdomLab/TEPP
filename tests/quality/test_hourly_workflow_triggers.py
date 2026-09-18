"""Structural contract for the centrally dispatched hourly workflow trigger."""

from __future__ import annotations

import unittest
from pathlib import Path

WORKFLOW = Path(".github/workflows/hourly-nim-product-development.yml")


def _normalized_yaml_key(raw: str) -> str:
    """Return a simple YAML mapping key with optional quotes removed."""

    key = raw.strip()
    if len(key) >= 2 and key[0] == key[-1] and key[0] in {"'", '"'}:
        return key[1:-1]
    return key


def _workflow_triggers(text: str) -> set[str]:
    """Parse the block-form top-level ``on`` mapping and return its trigger keys.

    The workflow deliberately uses block mapping syntax. Inline/scalar ``on``
    forms fail closed here so adding a trigger cannot bypass the central-dispatch
    contract through a different YAML spelling.
    """

    lines = text.splitlines()
    on_index: int | None = None
    for index, raw in enumerate(lines):
        if not raw or raw.lstrip().startswith("#"):
            continue
        if len(raw) - len(raw.lstrip()) != 0 or ":" not in raw:
            continue
        key, remainder = raw.split(":", 1)
        if _normalized_yaml_key(key) != "on":
            continue
        if remainder.strip():
            raise AssertionError("workflow trigger contract requires block-form on mapping")
        on_index = index
        break
    if on_index is None:
        raise AssertionError("workflow is missing a top-level on mapping")

    triggers: set[str] = set()
    for raw in lines[on_index + 1 :]:
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip())
        if indent == 0:
            break
        if indent != 2 or ":" not in raw:
            continue
        key, _remainder = raw.strip().split(":", 1)
        triggers.add(_normalized_yaml_key(key))
    return triggers


class HourlyWorkflowTriggerTests(unittest.TestCase):
    """Only central ``workflow_dispatch`` may admit the hourly writer."""

    def test_repository_workflow_allows_only_workflow_dispatch(self) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")
        self.assertEqual(_workflow_triggers(text), {"workflow_dispatch"})

    def test_other_block_triggers_are_detected_even_when_quoted(self) -> None:
        for extra in ("push", "repository_dispatch", "schedule", '"schedule"'):
            with self.subTest(extra=extra):
                text = f"on:\n  workflow_dispatch:\n  {extra}:\n"
                self.assertNotEqual(_workflow_triggers(text), {"workflow_dispatch"})

    def test_inline_trigger_spelling_fails_closed(self) -> None:
        with self.assertRaisesRegex(AssertionError, "block-form"):
            _workflow_triggers('"on": [workflow_dispatch, push]\n')


if __name__ == "__main__":
    unittest.main()
