#!/usr/bin/env python3
"""Validate TEPP's repository-level documentation and workflow contracts."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PRODUCT_TECHNICAL_GAP_BASELINE = "docs/product-technical-gap-baseline.md"
DOMAIN_CONTEXT_MAP = "docs/architecture/domain-context-map.md"
TEMPORAL_DEPENDENCE_COMPOSITION = "docs/architecture/temporal-dependence-composition.md"
TEMPORAL_DEPENDENCE_RESEARCH = "docs/research/temporal-dependence-models.md"

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
    "docs/adr/0017-hourly-contextual-orchestrator-gateway.md",
    "docs/adr/0018-consumer-scoped-analysis-run-ingress.md",
    "docs/adr/0019-project-history-wire-size-symmetry.md",
    "docs/adr/0020-span-grounded-semantic-units.md",
    "docs/adr/0021-lineageweave-project-history-boundary.md",
    "docs/adr/0022-deterministic-analysis-run-execution.md",
    "docs/product/prd-v0.4-approved.md",
    PRODUCT_TECHNICAL_GAP_BASELINE,
    DOMAIN_CONTEXT_MAP,
    TEMPORAL_DEPENDENCE_COMPOSITION,
    "docs/roadmaps/2026-08-05-tepp-delivery-roadmap.md",
    "docs/superpowers/plans/2026-08-05-temporal-event-foundation.md",
    "docs/research/standards-and-literature.md",
    TEMPORAL_DEPENDENCE_RESEARCH,
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
PROTECTED_MAIN_SHA = re.compile(
    r"\*\*Protected-main evidence:\*\*\s*`(?P<sha>[0-9a-f]{40})`"
)
SNAPSHOT_STAMP = re.compile(
    r"\*\*Snapshot:\*\*\s*(?P<stamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z)"
)
INVENTORY_ROW = re.compile(
    r"^\|\s*#(?P<number>\d+)\s*\|\s*`(?P<sha>[0-9a-f]{40})`\s*\|\s*"
    r"(?P<draft>true|false)\s*\|",
    re.MULTILINE,
)
PRIORITY_INVENTORY_HEADING = "## Current priority open pull-request evidence"
LEVEL_TWO_HEADING = re.compile(r"^##\s+", re.MULTILINE)
OPEN_PR_COUNT = re.compile(
    r"\|\s*Open pull requests\s*\|\s*\*\*(?P<count>\d+)\*\*"
)
QUEUED_CHECKS_AS_SHIPPED = re.compile(
    r"queued Checks.{0,80}implemented-main",
    re.IGNORECASE | re.DOTALL,
)
QUEUED_CHECKS_NEGATED_VERB = re.compile(
    r"\b(?P<cue>never|not|cannot|must\s+not|do\s+not|does\s+not)\b"
    r"[a-z\s]{0,12}\b"
    r"(?P<verb>promot\w+|treat\w*|make\w*|mean\w*|constitut\w+|represent\w*)",
    re.IGNORECASE,
)
QUEUED_CHECKS_ADVERSATIVE = re.compile(
    r"\b(?:but|however|yet|although|though)\b", re.IGNORECASE
)
QUEUED_CHECKS_SENTENCE_BREAK = re.compile(r"[.;!?\n]")
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
    r"(?:"
    r"landable coverage gate is PR #\d+"
    r"|PR #\d+ is the landable coverage gate"
    r"|landable gate is PR #\d+"
    r"|coverage-authority landing PR is PR #\d+"
    r"|coverage-authority landing PR #\d+"
    r"|merge PR #\d+ as the coverage-authority"
    r")",
    re.IGNORECASE,
)
STALE_REFUSE_PROMOTION_DRAFT_AUTHORITY = re.compile(
    r"refuse_promotion`? in PR #\d+ is the coverage authority"
)
STALE_MERGE_WEAK_DRAFTS = re.compile(r"merging the existing drafts")
UNMERGED_QUEUE_SENTENCE = re.compile(r"[^.]*unmerged[^.]*", re.IGNORECASE)
NEGATED_KEEP_UNMERGED = re.compile(r"do not keep\b[^.]*\bunmerged", re.IGNORECASE)
NARUON_LIVE_HTTP_SUBJECT = re.compile(
    r"naruon live HTTP(?: loopback)?\s*(?:\(|is\s+)PR #(\d+)",
    re.IGNORECASE,
)
REQUIRED_UNMERGED_COVERAGE_DRAFTS = (93, 94, 97, 101, 102, 104, 108, 109, 111, 112)
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
    PRODUCT_TECHNICAL_GAP_BASELINE,
    "docs/DOCUMENTATION_ASSESSMENT.md",
    "docs/TRD.md",
    "ARCHITECTURE.md",
    DOMAIN_CONTEXT_MAP,
    TEMPORAL_DEPENDENCE_COMPOSITION,
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
    TEMPORAL_DEPENDENCE_RESEARCH,
    "GOVERNANCE.md",
    "AGENTS.md",
    "CLAUDE.md",
    "CHANGELOG.md",
)


def markdown_files() -> list[Path]:
    """Return all version-controlled Markdown candidates under the repository."""

    return sorted(path for path in ROOT.rglob("*.md") if ".git" not in path.parts)


def validate_required_files(root: Path = ROOT) -> None:
    """Require the approved governance, product, and technical documentation baseline."""

    missing = [path for path in REQUIRED_FILES if not (root / path).is_file()]
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


def naruon_live_http_subject(hourly: str) -> str | None:
    """Return the PR number named as the naruon live HTTP subject, if any.

    A keep-unmerged mention of PR #107 is not the subject. The subject is the
    number immediately after ``naruon live HTTP`` or
    ``naruon live HTTP loopback``.
    """

    match = NARUON_LIVE_HTTP_SUBJECT.search(hourly)
    if match is None:
        return None
    return match.group(1)


def _hourly_queue_lock_failures(hourly: str) -> list[str]:
    """Return queue-lock failures when hourly names a coverage or naruon pointer.

    Inverted and paraphrased landable-gate sentences are rejected by
    ``STALE_LANDABLE_COVERAGE_GATE``. This queue lock refuses an unmerged list
    that omits later coverage drafts, refuses a negated Keep-unmerged sentence,
    and refuses a naruon pointer whose subject is not PR #107 with #87 and
    #105 kept unmerged.
    """

    if not hourly:
        return []
    failures: list[str] = []
    if NEGATED_KEEP_UNMERGED.search(hourly):
        failures.append(
            "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md negates the "
            "Keep-unmerged coverage-authority lock"
        )
    looks_like_queue = "unmerged" in hourly.casefold() or "naruon" in hourly.casefold()
    if not looks_like_queue:
        return failures
    joined = _hourly_unmerged_text(hourly)
    if any(
        f"PR #{number}" not in joined for number in REQUIRED_UNMERGED_COVERAGE_DRAFTS
    ):
        failures.append(
            "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md omits later "
            "coverage-authority drafts from the unmerged set"
        )
    if (
        naruon_live_http_subject(hourly) != "107"
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
    a draft such as #93, #94, #97, #101, #102, #104, #108, #109, #111, or
    #112.
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


def validate_documentation_map(root: Path = ROOT) -> None:
    """Require cross-cutting canonical documents to be discoverable from the root map."""

    documentation = (root / "DOCUMENTATION.md").read_text(encoding="utf-8")
    link_targets = {
        match.group("target") for match in MARKDOWN_LINK.finditer(documentation)
    }
    missing_links = [path for path in CANONICAL_LINKS if path not in link_targets]
    if missing_links:
        raise AssertionError(
            f"canonical documentation map is missing links: {missing_links}"
        )


def _promotion_is_denied(text: str, claim: re.Match[str]) -> bool:
    """Return whether the sentence around ``claim`` negates its promotion.

    A sentence denies the claim only when a negation cue directly governs a
    promotion verb within that same sentence and no adversative conjunction
    separates that pair from the ``implemented-main`` assertion. This accepts
    honest wordings such as "does not treat queued Checks as implemented-main"
    while refusing sentences where an unrelated negation coexists with an
    affirmative maturity claim after "but".
    """

    sentence_start = 0
    for boundary in QUEUED_CHECKS_SENTENCE_BREAK.finditer(text, 0, claim.start()):
        sentence_start = boundary.end()
    window_end = claim.end()
    for cue in QUEUED_CHECKS_NEGATED_VERB.finditer(text, sentence_start, window_end):
        if QUEUED_CHECKS_ADVERSATIVE.search(text, cue.end(), window_end) is None:
            return True
    return False


def _priority_inventory_section(text: str) -> str:
    """Return only the canonical priority-inventory section from the gap register."""

    heading = re.search(
        rf"^{re.escape(PRIORITY_INVENTORY_HEADING)}[ \t]*$",
        text,
        re.MULTILINE,
    )
    if heading is None:
        return ""
    section_start = heading.end()
    next_heading = LEVEL_TWO_HEADING.search(text, section_start)
    section_end = next_heading.start() if next_heading is not None else len(text)
    return text[section_start:section_end]


def validate_product_technical_gap_baseline(root: Path = ROOT) -> None:
    """Require a dated live gap register with an honest priority PR inventory."""

    path = root / PRODUCT_TECHNICAL_GAP_BASELINE
    if not path.is_file():
        raise AssertionError(
            f"missing required documentation: ['{PRODUCT_TECHNICAL_GAP_BASELINE}']"
        )
    text = path.read_text(encoding="utf-8")
    failures: list[str] = []
    if SNAPSHOT_STAMP.search(text) is None:
        failures.append("gap baseline lacks a dated UTC snapshot stamp")
    if PROTECTED_MAIN_SHA.search(text) is None:
        failures.append("gap baseline lacks a 40-character protected-main SHA")
    if "Closure evidence" not in text:
        failures.append("gap baseline lacks operator-gap closure evidence")
    if "Exact current head" not in text:
        failures.append("gap baseline lacks an exact-head open-PR inventory")
    if any(
        not _promotion_is_denied(text, match)
        for match in QUEUED_CHECKS_AS_SHIPPED.finditer(text)
    ):
        failures.append("gap baseline treats queued Checks as implemented-main")
    priority_inventory = _priority_inventory_section(text)
    inventory = list(INVENTORY_ROW.finditer(priority_inventory))
    inventory_numbers = [match.group("number") for match in inventory]
    if not inventory:
        failures.append("gap baseline open-PR inventory has no exact-head rows")
    elif len(inventory_numbers) != len(set(inventory_numbers)):
        failures.append("gap baseline priority inventory contains duplicate PR rows")
    count_match = OPEN_PR_COUNT.search(text)
    if count_match is None:
        failures.append("gap baseline lacks an open pull-request count")
    elif int(count_match.group("count")) < len(inventory):
        failures.append(
            "gap baseline open-PR count "
            f"{count_match.group('count')} is smaller than priority inventory "
            f"{len(inventory)}"
        )
    if failures:
        raise AssertionError("\n".join(failures))


def validate_adr_graph() -> None:
    """Require every numbered ADR to have one repository-wide identity."""

    adr_root = ROOT / "docs" / "adr"
    adr_index = (adr_root / "README.md").read_text(encoding="utf-8")
    indexed_number_list = [
        match.group("number") for match in ADR_TABLE_ROW.finditer(adr_index)
    ]
    duplicate_index_numbers = sorted(
        number
        for number in set(indexed_number_list)
        if indexed_number_list.count(number) > 1
    )
    if duplicate_index_numbers:
        raise AssertionError(
            f"duplicate ADR index identity: {duplicate_index_numbers}"
        )
    indexed_numbers = set(indexed_number_list)

    adr_paths_by_number: dict[str, list[Path]] = {}
    for path in sorted(adr_root.glob("[0-9][0-9][0-9][0-9]-*.md")):
        match = ADR_FILE_NAME.fullmatch(path.name)
        if not match:
            raise AssertionError(f"invalid ADR filename: {path.relative_to(ROOT)}")
        adr_paths_by_number.setdefault(match.group("number"), []).append(path)

    duplicate_file_numbers = sorted(
        number for number, paths in adr_paths_by_number.items() if len(paths) > 1
    )
    if duplicate_file_numbers:
        raise AssertionError(
            f"duplicate ADR file identity: {duplicate_file_numbers}"
        )

    adr_files = {number: paths[0] for number, paths in adr_paths_by_number.items()}
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
    validate_product_technical_gap_baseline()
    validate_adr_graph()
    validate_markdown()
    validate_workflows()
    validate_json()
    print("TEPP documentation validation passed")


if __name__ == "__main__":
    main()
