"""Tests for repository documentation contracts, including promotion authority."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import validate_documentation as documentation


class PromotionAuthorityPointerTests(unittest.TestCase):
    """Refuse canonical docs that name a superseded draft as the coverage gate."""

    def test_stale_pr94_pointers_fail_closed(self) -> None:
        """The #94 wording that made refuse_promotion look landable is rejected."""

        stale_documentation = (
            "The active-PR coverage gate in `prediction_contradiction` (PR #94) "
            "requires observed Allen coverage."
        )
        stale_assessment = (
            "- **active-PR:** PR #94 `prediction_contradiction` Allen coverage gate"
        )
        self.assertEqual(
            documentation.promotion_authority_failures(
                stale_documentation, stale_assessment
            ),
            [
                "DOCUMENTATION.md names a superseded draft as the coverage-gate authority",
                "docs/DOCUMENTATION_ASSESSMENT.md names a superseded draft as the "
                "active-PR coverage gate",
            ],
        )

    def test_stale_pr93_pointers_fail_closed(self) -> None:
        """The earlier contradiction-only draft is also not landable authority."""

        stale_documentation = (
            "The active-PR coverage gate in `prediction_contradiction` (PR #93) "
            "requires observed Allen coverage."
        )
        stale_assessment = (
            "- **active-PR:** PR #93 `prediction_contradiction` Allen coverage gate"
        )
        self.assertEqual(
            documentation.promotion_authority_failures(
                stale_documentation, stale_assessment
            ),
            [
                "DOCUMENTATION.md names a superseded draft as the coverage-gate authority",
                "docs/DOCUMENTATION_ASSESSMENT.md names a superseded draft as the "
                "active-PR coverage gate",
            ],
        )

    def test_one_sided_stale_pointers_fail_independently(self) -> None:
        """Each canonical file is checked even when the other is already current."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` (PR #94).",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
            ),
            [
                "DOCUMENTATION.md names a superseded draft as the coverage-gate authority"
            ],
        )
        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` requires coverage.",
                "- **active-PR:** PR #94 `prediction_contradiction` Allen coverage gate",
            ),
            [
                "docs/DOCUMENTATION_ASSESSMENT.md names a superseded draft as the "
                "active-PR coverage gate"
            ],
        )

    def test_stale_pr97_pointers_fail_closed(self) -> None:
        """The #97 pointer-repair draft is also not landable authority."""

        stale_documentation = (
            "The active-PR coverage gate in `prediction_contradiction` (PR #97) "
            "requires observed Allen coverage."
        )
        stale_assessment = (
            "- **active-PR:** PR #97 `prediction_contradiction` Allen coverage gate"
        )
        self.assertEqual(
            documentation.promotion_authority_failures(
                stale_documentation, stale_assessment
            ),
            [
                "DOCUMENTATION.md names a superseded draft as the coverage-gate authority",
                "docs/DOCUMENTATION_ASSESSMENT.md names a superseded draft as the "
                "active-PR coverage gate",
            ],
        )

    def test_parenthetical_without_backtick_fails(self) -> None:
        """A missing markdown fence must not hide a draft authority pointer."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The landable gate is prediction_contradiction (PR #94).",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
            ),
            [
                "DOCUMENTATION.md names a superseded draft as the coverage-gate authority"
            ],
        )

    def test_landable_and_refuse_promotion_authority_sentences_fail(self) -> None:
        """Plain-language authority sentences are rejected even without markdown."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The landable coverage gate is PR #94.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
            ),
            [
                "DOCUMENTATION.md names a superseded draft as the coverage-gate authority"
            ],
        )
        self.assertEqual(
            documentation.promotion_authority_failures(
                "`refuse_promotion` in PR #94 is the coverage authority.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
            ),
            [
                "DOCUMENTATION.md names a superseded draft as the coverage-gate authority"
            ],
        )

    def test_citation_repair_drafts_are_not_landable_authority(self) -> None:
        """#101, #102, #104, and #108 still name a draft as the gate."""

        for pull_request in (101, 102, 104, 108):
            with self.subTest(pull_request=pull_request):
                self.assertEqual(
                    documentation.promotion_authority_failures(
                        f"The landable coverage gate is PR #{pull_request}.",
                        "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                    ),
                    [
                        "DOCUMENTATION.md names a superseded draft as the "
                        "coverage-gate authority"
                    ],
                )
                self.assertEqual(
                    documentation.promotion_authority_failures(
                        (
                            "The landable gate is prediction_contradiction "
                            f"(PR #{pull_request})."
                        ),
                        "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                    ),
                    [
                        "DOCUMENTATION.md names a superseded draft as the "
                        "coverage-gate authority"
                    ],
                )
                self.assertEqual(
                    documentation.promotion_authority_failures(
                        (
                            "`refuse_promotion` in PR "
                            f"#{pull_request} is the coverage authority."
                        ),
                        "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                    ),
                    [
                        "DOCUMENTATION.md names a superseded draft as the "
                        "coverage-gate authority"
                    ],
                )

    def test_changelog_and_architecture_are_scanned(self) -> None:
        """CHANGELOG and ARCHITECTURE cannot rename a draft as the landable gate."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` requires coverage.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                extra_documents={
                    "CHANGELOG.md": "The landable coverage gate is PR #102.",
                    "ARCHITECTURE.md": (
                        "gate in prediction_contradiction (PR #101) requires coverage"
                    ),
                },
            ),
            [
                "CHANGELOG.md names a superseded draft as the coverage-gate authority",
                "ARCHITECTURE.md names a superseded draft as the coverage-gate authority",
            ],
        )

    def test_hourly_merge_existing_drafts_fails(self) -> None:
        """The queue must not treat #93/#94/#97 as mergeable drafts."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` requires coverage.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                hourly=(
                    "Prefer reviewing, repairing, and merging the existing drafts."
                ),
            ),
            [
                "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md tells the "
                "queue to merge superseded coverage drafts"
            ],
        )

    def test_extra_canonical_files_are_scanned(self) -> None:
        """ADR, TRACEABILITY, and UML cannot rename a draft as the gate."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` requires coverage.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                extra_documents={
                    "docs/TRACEABILITY.md": (
                        "The landable coverage gate is PR #94."
                    )
                },
            ),
            [
                "docs/TRACEABILITY.md names a superseded draft as the "
                "coverage-gate authority"
            ],
        )

    def test_hourly_unmerged_set_omitting_later_drafts_fails(self) -> None:
        """#104, #108, #109, and #111 must appear in Keep-unmerged sentences."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` requires coverage.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                hourly=(
                    "Keep PR #93, PR #94, PR #97, PR #101, PR #102, PR #104, and "
                    "PR #108 unmerged. naruon live HTTP loopback (PR #107; "
                    "keep PR #87 and PR #105 unmerged)"
                ),
            ),
            [
                "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md omits later "
                "coverage-authority drafts from the unmerged set"
            ],
        )

    def test_hourly_naruon_pointer_away_from_107_fails(self) -> None:
        """The next buyer slice must stay on the live loopback listener, not #105."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` requires coverage.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                hourly=(
                    "Keep PR #93, PR #94, PR #97, PR #101, PR #102, PR #104, "
                    "PR #108, PR #109, and PR #111 unmerged. naruon live HTTP "
                    "loopback (PR #105; keep PR #87 unmerged)"
                ),
            ),
            [
                "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md points naruon "
                "live HTTP away from PR #107"
            ],
        )

    def test_crate_named_authority_and_draft_lineage_pass(self) -> None:
        """Naming the crate, and mentioning drafts as non-landable, is allowed."""

        current_documentation = (
            "The active-PR coverage gate in `prediction_contradiction` requires "
            "observed Allen coverage. Drafts #93, #94, and #97 are not landable "
            "while they still point at PR #94 as the authority."
        )
        current_assessment = (
            "- **active-PR:** `prediction_contradiction` Allen coverage gate; "
            "refuse_promotion requires observed coverage."
        )
        current_hourly = (
            "Keep PR #93, PR #94, PR #97, PR #101, PR #102, PR #104, "
            "PR #108, PR #109, and PR #111 unmerged. Prefer merging the "
            "coverage-authority landing PR. naruon live HTTP loopback "
            "(PR #107; keep PR #87 and PR #105 unmerged)"
        )
        self.assertEqual(
            documentation.promotion_authority_failures(
                current_documentation,
                current_assessment,
                hourly=current_hourly,
            ),
            [],
        )

    def test_live_repository_does_not_name_superseded_drafts(self) -> None:
        """Current canonical files pass the coverage-authority pointer contract."""

        documentation.validate_promotion_authority_pointers()

    def test_assessment_parenthetical_and_hourly_parenthetical_fail(self) -> None:
        """Assessment and hourly files fail on parenthetical draft pointers too."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` requires coverage.",
                "gate in `prediction_contradiction` (PR #94) requires coverage",
            ),
            [
                "docs/DOCUMENTATION_ASSESSMENT.md names a superseded draft as the "
                "active-PR coverage gate"
            ],
        )
        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` requires coverage.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                hourly="gate in `prediction_contradiction` (PR #93) requires coverage",
            ),
            [
                "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md tells the "
                "queue to merge superseded coverage drafts"
            ],
        )

    def test_extra_file_active_pr_pointer_fails(self) -> None:
        """An extra canonical file with an active-PR draft pointer is rejected."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The active-PR coverage gate in `prediction_contradiction` requires coverage.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
                extra_documents={
                    "docs/UML.md": "- **active-PR:** PR #97 `prediction_contradiction`"
                },
            ),
            [
                "docs/UML.md names a superseded draft as the coverage-gate authority"
            ],
        )

    def test_validate_promotion_authority_pointers_raises_on_missing_files(
        self,
    ) -> None:
        """File-backed validation fail-closes when an authority document is absent."""

        with tempfile.TemporaryDirectory() as temporary:
            with mock.patch.object(documentation, "ROOT", Path(temporary)):
                with self.assertRaises(AssertionError) as raised:
                    documentation.validate_promotion_authority_pointers()
        self.assertIn("missing promotion-authority documents", str(raised.exception))

    def test_validate_promotion_authority_pointers_raises_on_stale_files(
        self,
    ) -> None:
        """File-backed validation fail-closes when either canonical file is stale."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "DOCUMENTATION.md").write_text(
                "gate in `prediction_contradiction` (PR #94) requires coverage\n",
                encoding="utf-8",
            )
            for relative in (
                "docs/DOCUMENTATION_ASSESSMENT.md",
                "docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md",
                "docs/TRACEABILITY.md",
                "docs/UML.md",
                "docs/adr/0016-tdt-chronos-event-intelligence-boundary.md",
                "CHANGELOG.md",
                "ARCHITECTURE.md",
                "README.md",
                "docs/adr/README.md",
            ):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(
                    "- **active-PR:** PR #94 `prediction_contradiction`\n"
                    if relative.endswith("DOCUMENTATION_ASSESSMENT.md")
                    else "crate-named coverage gate\n",
                    encoding="utf-8",
                )
            with mock.patch.object(documentation, "ROOT", root):
                with self.assertRaises(AssertionError) as raised:
                    documentation.validate_promotion_authority_pointers()
        self.assertIn("superseded draft", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
