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
STALE_COVERAGE_GATE_PARENTHETICAL = re.compile(
    r"prediction_contradiction`? \(PR #\d+\)"
)
STALE_ACTIVE_PR_COVERAGE_GATE = re.compile(r"\*\*active-PR:\*\*\s*PR #\d+\b")
STALE_LANDABLE_COVERAGE_GATE = re.compile(
    r"landable coverage gate is PR #\d+\b",
    re.IGNORECASE,
)
STALE_REFUSE_PROMOTION_DRAFT_AUTHORITY = re.compile(
    r"refuse_promotion`? in PR #\d+ is the coverage authority"
)
STALE_MERGE_WEAK_DRAFTS = re.compile(r"merging the existing drafts")
UNMERGED_QUEUE_SENTENCE = re.compile(r"[^.]*unmerged[^.]*", re.IGNORECASE)
REQUIRED_UNMERGED_COVERAGE_DRAFTS = (93, 94, 97, 101, 102, 104, 108)
AUTHORITY_POINTER_FILES = (
    "DOCUMENTATION.md",
    "docs/DOCUMENTATION_ASSESSMENT.md",
    "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md",
    "docs/TRACEABILITY.md",
    "docs/UML.md",
    "docs/adr/0016-tdt-chronos-event-intelligence-boundary.md",
    "CHANGELOG.md",
    "ARCHITECTURE.md",
    "README.md",
    "docs/adr/README.md",
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


def _document_has_stale_coverage_authority(text: str) -> bool:
    """Return whether one document names a superseded draft as the coverage gate."""

    return bool(
        STALE_COVERAGE_GATE_PARENTHETICAL.search(text)
        or STALE_LANDABLE_COVERAGE_GATE.search(text)
        or STALE_REFUSE_PROMOTION_DRAFT_AUTHORITY.search(text)
    )


def _hourly_unmerged_text(hourly: str) -> str:
    """Return Keep-unmerged sentences so later drafts cannot hide outside the lock."""

    collapsed = hourly.replace("\n", " ")
    return " ".join(UNMERGED_QUEUE_SENTENCE.findall(collapsed))


def _hourly_queue_lock_failures(hourly: str) -> list[str]:
    """Return queue-lock failures when hourly names a coverage or naruon pointer.

    The phrase lock already rejects `landable coverage gate is PR #N`. This
    queue lock refuses an unmerged list that stops at #101/#102, and refuses a
    naruon pointer that is not PR #107 with #87 and #105 kept unmerged.
    """

    if not hourly:
        return []
    looks_like_queue = "unmerged" in hourly.casefold() or "naruon" in hourly.casefold()
    if not looks_like_queue:
        return []
    failures: list[str] = []
    joined = _hourly_unmerged_text(hourly)
    if any(
        f"PR #{number}" not in joined for number in REQUIRED_UNMERGED_COVERAGE_DRAFTS
    ):
        failures.append(
            "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md omits later "
            "coverage-authority drafts from the unmerged set"
        )
    if (
        "PR #107" not in hourly
        or "PR #105" not in joined
        or "PR #87" not in joined
    ):
        failures.append(
            "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md points naruon "
            "live HTTP away from PR #107"
        )
    return failures


def promotion_authority_failures(
    documentation: str,
    assessment: str,
    hourly: str = "",
    extra_documents: dict[str, str] | None = None,
) -> list[str]:
    """Return stale pointers that name a superseded draft as the coverage gate.

    A pull-request number is not landable coverage authority. Canonical docs
    and the hourly queue must name the `prediction_contradiction` crate, not
    a draft such as #93, #94, #97, #101, #102, #104, or #108.
    """

    failures: list[str] = []
    if _document_has_stale_coverage_authority(documentation):
        failures.append(
            "DOCUMENTATION.md names a superseded draft as the coverage-gate authority"
        )
    if STALE_ACTIVE_PR_COVERAGE_GATE.search(assessment) or (
        _document_has_stale_coverage_authority(assessment)
    ):
        failures.append(
            "docs/DOCUMENTATION_ASSESSMENT.md names a superseded draft as the "
            "active-PR coverage gate"
        )
    if STALE_MERGE_WEAK_DRAFTS.search(hourly) or (
        _document_has_stale_coverage_authority(hourly)
    ):
        failures.append(
            "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md tells the "
            "queue to merge superseded coverage drafts"
        )
    failures.extend(_hourly_queue_lock_failures(hourly))
    for path, text in (extra_documents or {}).items():
        if _document_has_stale_coverage_authority(text) or (
            STALE_ACTIVE_PR_COVERAGE_GATE.search(text)
        ):
            failures.append(
                f"{path} names a superseded draft as the coverage-gate authority"
            )
    return failures


def validate_promotion_authority_pointers() -> None:
    """Refuse canonical docs that still treat a pull request as landable authority."""

    missing = [
        relative
        for relative in AUTHORITY_POINTER_FILES
        if not (ROOT / relative).is_file()
    ]
    if missing:
        raise AssertionError(f"missing promotion-authority documents: {missing}")
    texts = {
        relative: (ROOT / relative).read_text(encoding="utf-8")
        for relative in AUTHORITY_POINTER_FILES
    }
    extra_documents = {
        path: text
        for path, text in texts.items()
        if path
        not in {
            "DOCUMENTATION.md",
            "docs/DOCUMENTATION_ASSESSMENT.md",
            "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md",
        }
    }
    failures = promotion_authority_failures(
        texts["DOCUMENTATION.md"],
        texts["docs/DOCUMENTATION_ASSESSMENT.md"],
        hourly=texts["docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md"],
        extra_documents=extra_documents,
    )
    if failures:
        raise AssertionError("\n".join(failures))


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
    validate_promotion_authority_pointers()
    validate_documentation_map()
    validate_adr_graph()
    validate_markdown()
    validate_workflows()
    validate_json()
    print("TEPP documentation validation passed")


if __name__ == "__main__":
    main()
