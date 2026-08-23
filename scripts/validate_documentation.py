#!/usr/bin/env python3
"""Validate TEPP's repository-level documentation and workflow contracts."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_FILES = (
    "DOCUMENTATION.md",
    "AGENTS.md",
    "CLAUDE.md",
    "ARCHITECTURE.md",
    "CHANGELOG.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "GOVERNANCE.md",
    "docs/API_CONTRACT.md",
    "docs/COMPLIANCE_READINESS.md",
    "docs/DOCUMENTATION_ASSESSMENT.md",
    "docs/ERD.md",
    "docs/LLM_ORCHESTRATION.md",
    "docs/OPERABILITY.md",
    "docs/PRIVACY_DATA_GOVERNANCE.md",
    "docs/TEST_STRATEGY.md",
    "docs/THREAT_MODEL.md",
    "docs/TRACEABILITY.md",
    "docs/TRD.md",
    "docs/UML.md",
    "docs/adr/README.md",
    "docs/adr/ADR_POLICY.md",
    "docs/adr/0009-purpose-bound-pii-governance.md",
    "docs/adr/0010-adaptive-llm-orchestration.md",
    "docs/adr/0011-standalone-modular-msa-boundary.md",
    "docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md",
    "docs/adr/0013-bitemporal-persistence-reproducibility-and-split-authority.md",
    "docs/adr/0014-scientific-claim-promotion-and-release-evidence.md",
    "docs/adr/0015-autonomous-development-review-and-merge-authority.md",
    "docs/adr/0016-tdt-chronos-event-intelligence-boundary.md",
    "docs/adr/0017-consumer-scoped-analysis-run-ingress.md",
    "docs/product/prd-v0.4-approved.md",
    "docs/roadmaps/2026-08-05-tepp-delivery-roadmap.md",
    "docs/superpowers/plans/2026-08-05-temporal-event-foundation.md",
    "docs/research/standards-and-literature.md",
)

PLACEHOLDER_PATTERNS = (
    re.compile(r"\bTBD\b"),
    re.compile(r"\bTODO\b"),
    re.compile(r"implement later", re.IGNORECASE),
    re.compile(r"fill in", re.IGNORECASE),
)

ACTION_REFERENCE = re.compile(r"uses:\s*[^\s@]+@([^\s#]+)")
FULL_COMMIT_SHA = re.compile(r"^[0-9a-f]{40}$")
MARKDOWN_LINK = re.compile(
    r'(?<!!)\[[^]\n]+\]\((?P<target>[^)\s]+)(?:\s+"[^"]*")?\)'
)
ADR_TABLE_ROW = re.compile(r"^\|\s*\[(?P<number>\d{4})\]", re.MULTILINE)
ADR_FILE_NAME = re.compile(r"^(?P<number>\d{4})-[a-z0-9-]+\.md$")
ADR_DECISION_STATUS = re.compile(
    r"^\*\*Decision status:\*\*\s*(Accepted|Proposed|Superseded|Rejected)\b",
    re.MULTILINE,
)
ADR_IMPLEMENTATION_STATUS = re.compile(
    r"^\*\*Implementation maturity:\*\*\s*"
    r"(implemented-main|active-PR|partial|accepted-target|research-only|out-of-scope)\b",
    re.MULTILINE,
)
ADR_REQUIRED_HEADINGS = (
    "## Context",
    "## Decision",
    "## Alternatives considered",
    "## Consequences",
    "## Verification",
)

CANONICAL_LINKS = (
    "docs/product/prd-v0.4-approved.md",
    "docs/DOCUMENTATION_ASSESSMENT.md",
    "docs/TRD.md",
    "ARCHITECTURE.md",
    "docs/API_CONTRACT.md",
    "docs/UML.md",
    "docs/ERD.md",
    "SECURITY.md",
    "docs/THREAT_MODEL.md",
    "docs/PRIVACY_DATA_GOVERNANCE.md",
    "docs/COMPLIANCE_READINESS.md",
    "docs/LLM_ORCHESTRATION.md",
    "docs/TEST_STRATEGY.md",
    "docs/OPERABILITY.md",
    "docs/TRACEABILITY.md",
    "docs/adr/README.md",
    "docs/adr/ADR_POLICY.md",
    "docs/roadmaps/2026-08-05-tepp-delivery-roadmap.md",
    "docs/superpowers/plans/2026-08-05-temporal-event-foundation.md",
    "docs/research/standards-and-literature.md",
    "GOVERNANCE.md",
    "AGENTS.md",
    "CLAUDE.md",
    "CHANGELOG.md",
)


def markdown_files() -> list[Path]:
    """Return all version-controlled Markdown candidates under the repository."""

    return sorted(path for path in ROOT.rglob("*.md") if ".git" not in path.parts)


def validate_required_files() -> None:
    """Require the approved governance, product, and technical documentation baseline."""

    missing = [path for path in REQUIRED_FILES if not (ROOT / path).is_file()]
    if missing:
        raise AssertionError(f"missing required documentation: {missing}")


def validate_documentation_map() -> None:
    """Require cross-cutting canonical documents to be discoverable from the root map."""

    documentation = (ROOT / "DOCUMENTATION.md").read_text(encoding="utf-8")
    link_targets = {
        match.group("target") for match in MARKDOWN_LINK.finditer(documentation)
    }
    missing_links = [path for path in CANONICAL_LINKS if path not in link_targets]
    if missing_links:
        raise AssertionError(
            f"canonical documentation map is missing links: {missing_links}"
        )


def validate_adr_graph() -> None:
    """Require every numbered ADR to be indexed and carry unambiguous authority metadata."""

    adr_root = ROOT / "docs" / "adr"
    adr_index = (adr_root / "README.md").read_text(encoding="utf-8")
    indexed_numbers = {
        match.group("number") for match in ADR_TABLE_ROW.finditer(adr_index)
    }

    adr_files: dict[str, Path] = {}
    for path in sorted(adr_root.glob("[0-9][0-9][0-9][0-9]-*.md")):
        match = ADR_FILE_NAME.fullmatch(path.name)
        if not match:
            raise AssertionError(f"invalid ADR filename: {path.relative_to(ROOT)}")
        adr_files[match.group("number")] = path

    file_numbers = set(adr_files)
    if indexed_numbers != file_numbers:
        raise AssertionError(
            "ADR index/file mismatch: "
            f"index_only={sorted(indexed_numbers - file_numbers)}, "
            f"file_only={sorted(file_numbers - indexed_numbers)}"
        )

    failures: list[str] = []
    for number, path in adr_files.items():
        text = path.read_text(encoding="utf-8")
        if ADR_DECISION_STATUS.search(text) is None:
            failures.append(f"ADR {number} lacks a valid Decision status")
        if ADR_IMPLEMENTATION_STATUS.search(text) is None:
            failures.append(f"ADR {number} lacks a valid Implementation maturity")
        if "**Supersedes:**" not in text and "**Supersession:**" not in text:
            failures.append(f"ADR {number} lacks explicit supersession metadata")
        for heading in ADR_REQUIRED_HEADINGS:
            if heading not in text:
                failures.append(f"ADR {number} lacks required heading {heading!r}")
        if "## Rollback" not in text:
            failures.append(f"ADR {number} lacks rollback/supersession behavior")

    if failures:
        raise AssertionError("\n".join(failures))


def validate_markdown() -> None:
    """Reject placeholders and unbalanced fenced code blocks."""

    failures: list[str] = []
    for path in markdown_files():
        text = path.read_text(encoding="utf-8")
        if text.count("```") % 2:
            failures.append(f"unbalanced code fence: {path.relative_to(ROOT)}")
        for pattern in PLACEHOLDER_PATTERNS:
            if pattern.search(text):
                failures.append(
                    f"placeholder {pattern.pattern!r}: {path.relative_to(ROOT)}"
                )
    if failures:
        raise AssertionError("\n".join(failures))


def validate_workflows() -> None:
    """Require immutable action pins and the approved NVIDIA secret boundary."""

    workflow_root = ROOT / ".github" / "workflows"
    if not workflow_root.exists():
        raise AssertionError("missing .github/workflows")

    combined = ""
    failures: list[str] = []
    for path in sorted(workflow_root.glob("*.yml")):
        text = path.read_text(encoding="utf-8")
        combined += text
        for reference in ACTION_REFERENCE.findall(text):
            if not FULL_COMMIT_SHA.fullmatch(reference):
                failures.append(
                    f"mutable action reference {reference!r}: {path.relative_to(ROOT)}"
                )

    if "COPILOT_GITHUB_TOKEN" in combined:
        failures.append("COPILOT_GITHUB_TOKEN is prohibited")
    if "hourly-autonomous-development" in " ".join(
        path.stem for path in workflow_root.glob("*.yml")
    ) and "NVIDIA_NIM_API_KEY" not in combined:
        failures.append("autonomous LLM workflow lacks NVIDIA_NIM_API_KEY")
    if failures:
        raise AssertionError("\n".join(failures))


def validate_json() -> None:
    """Parse every repository JSON document."""

    for path in sorted(ROOT.rglob("*.json")):
        if ".git" in path.parts:
            continue
        json.loads(path.read_text(encoding="utf-8"))


def main() -> None:
    """Run all deterministic documentation validation groups."""

    validate_required_files()
    validate_documentation_map()
    validate_adr_graph()
    validate_markdown()
    validate_workflows()
    validate_json()
    print("TEPP documentation validation passed")


if __name__ == "__main__":
    main()
