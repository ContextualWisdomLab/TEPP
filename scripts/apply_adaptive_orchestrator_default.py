#!/usr/bin/env python3
"""Migrate TEPP production contextual-orchestrator calls to explicit auto."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE_SUFFIXES = {".py", ".js", ".mjs", ".cjs", ".ts", ".tsx", ".rs", ".go"}
EXCLUDED = {
    ".git", ".github", "docs", "examples", "fixtures", "migrations",
    "node_modules", "scripts", "target", "test", "tests", "vendor",
}
REPLACEMENTS = (
    ('"mode": "route"', '"mode": "auto"'),
    ("'mode': 'route'", "'mode': 'auto'"),
    ('"orchestration_mode": "route"', '"orchestration_mode": "auto"'),
    ("'orchestration_mode': 'route'", "'orchestration_mode': 'auto'"),
    ('mode="route"', 'mode="auto"'),
    ("mode='route'", "mode='auto'"),
    ('mode: str = "route"', 'mode: str = "auto"'),
    ("mode: str = 'route'", "mode: str = 'auto'"),
)
AUTO_RE = re.compile(
    r"(?:orchestration_mode|mode)(?:\s*:\s*str)?\s*[:=]\s*[\"']auto[\"']",
    re.IGNORECASE,
)

integrations: list[Path] = []
for path in sorted(ROOT.rglob("*")):
    if not path.is_file() or path.suffix.lower() not in SOURCE_SUFFIXES:
        continue
    relative = path.relative_to(ROOT)
    if {part.lower() for part in relative.parts} & EXCLUDED:
        continue
    source = path.read_text(encoding="utf-8")
    lowered = source.lower()
    if "contextual-orchestrator" not in lowered and "contextual_orchestrator" not in lowered:
        continue
    integrations.append(path)
    updated = source
    for old, new in REPLACEMENTS:
        updated = updated.replace(old, new)
    if (
        "chat/completions" in updated.lower()
        and "contextual-orchestrator" in updated.lower()
        and not AUTO_RE.search(updated)
    ):
        candidates = [
            (
                r'(?P<indent>[ \t]*)(?P<q>["\'])model(?P=q)\s*:\s*(?P<v>[^,\n}]+),\s*\n(?P=indent)(?P<mq>["\'])messages(?P=mq)\s*:',
                lambda m: (
                    f"{m.group('indent')}{m.group('q')}model{m.group('q')}:{m.group('v')},\n"
                    f"{m.group('indent')}{m.group('q')}orchestration_mode{m.group('q')}: {m.group('q')}auto{m.group('q')},\n"
                    f"{m.group('indent')}{m.group('mq')}messages{m.group('mq')}:"
                ),
            ),
            (
                r'(?P<p>["\']model["\']\s*:\s*[^,}]+,\s*)(?P<m>["\']messages["\']\s*:)',
                lambda m: f"{m.group('p')}\"orchestration_mode\": \"auto\", {m.group('m')}",
            ),
            (
                r'(?P<i>[ \t]*)model=(?P<v>[^,\n)]+),\s*\n(?P=i)messages=',
                lambda m: (
                    f"{m.group('i')}model={m.group('v')},\n"
                    f"{m.group('i')}extra_body={{\"orchestration_mode\": \"auto\"}},\n"
                    f"{m.group('i')}messages="
                ),
            ),
        ]
        for pattern, replacement in candidates:
            updated, count = re.subn(pattern, replacement, updated, count=1)
            if count == 1:
                break
        else:
            raise RuntimeError(f"could not locate request payload in {relative}")
    if updated != source:
        path.write_text(updated, encoding="utf-8")

if not integrations:
    raise RuntimeError("no production contextual-orchestrator integration was found")

adr = ROOT / "docs" / "adr" / "0025-adaptive-contextual-orchestrator-default.md"
adr.parent.mkdir(parents=True, exist_ok=True)
if not adr.exists():
    adr.write_text(
        '''# ADR-0025: TEPP interpretation delegates execution to contextual-orchestrator auto

- Status: Accepted
- Date: 2026-08-16

## Context

TEPP's multilingual, temporal, multilevel topic interpretation requires different
amounts of test-time computation across extraction, labeling, comparison, and
high-uncertainty synthesis. A consumer-owned `route` or implicit request mode forces
one worker and duplicates provider/workflow policy inside the psychometric product.

## Decision

Every production contextual-orchestrator request explicitly selects `auto`.
Contextual-orchestrator owns route/verify/conduct selection, provider/model choice,
failover, and known-cost tie-breaks. Quality and safety requirements precede cost;
unknown price metadata is not treated as zero.

TEPP retains semantic-unit multilingual input construction, temporal and
multilevel/multiple-membership provenance, strict output validation, statistical
model ownership, and Rust/GPU numeric computation. Fixed orchestration modes remain
controlled evaluation fixtures only.

## References

Omidvar, H., & Akhlaghi, V. (2026). *A communication-theoretic framework for LLM agents: Cost-aware adaptive reliability* [Preprint]. arXiv. https://doi.org/10.48550/arXiv.2605.09121

Tang, Y., Cetin, E., Xu, J., Sun, Q., Nielsen, S., Richard, V., Goda, H., Tymchenko, I., Nguyen, N., Lee, H., Ashiga, M., Kotyan, S., Kuroki, S., & Clanuwat, T. (2026). *Sakana Fugu technical report* [Technical report]. arXiv. https://doi.org/10.48550/arXiv.2606.21228
''',
        encoding="utf-8",
    )

changelog_path = ROOT / "CHANGELOG.md"
if changelog_path.exists():
    text = changelog_path.read_text(encoding="utf-8")
    entry = (
        "- Production interpretation requests now explicitly use contextual-orchestrator "
        "`auto` instead of a single-model route default.\n"
    )
    if entry not in text:
        marker = "## Unreleased\n"
        text = (
            text.replace(marker, marker + "\n### Changed\n\n" + entry, 1)
            if marker in text
            else "## Unreleased\n\n### Changed\n\n" + entry + "\n" + text
        )
        changelog_path.write_text(text, encoding="utf-8")
