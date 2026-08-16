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

    def test_stale_pr97_pointers_fail_closed(self) -> None:
        """PR #97 still named #94 as authority and is not the landable pointer."""

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

    def test_backtick_less_landable_gate_phrase_fails_closed(self) -> None:
        """A prose landable-gate sentence is rejected even without crate backticks."""

        self.assertEqual(
            documentation.promotion_authority_failures(
                "The landable coverage gate is PR #94.",
                "- **active-PR:** `prediction_contradiction` Allen coverage gate",
            ),
            [
                "DOCUMENTATION.md names a superseded draft as the coverage-gate authority"
            ],
        )

    def test_crate_named_authority_and_draft_lineage_pass(self) -> None:
        """Naming the crate, and mentioning drafts as non-landable, is allowed."""

        current_documentation = (
            "The active-PR coverage gate in `prediction_contradiction` requires "
            "observed Allen coverage before unmatched predicted mass can be "
            "promoted. Drafts #93, #94, and #97 are not landable while they "
            "still point at PR #94 as the authority."
        )
        current_assessment = (
            "- **active-PR:** `prediction_contradiction` Allen coverage gate; "
            "refuse_promotion requires observed coverage."
        )
        self.assertEqual(
            documentation.promotion_authority_failures(
                current_documentation, current_assessment
            ),
            [],
        )

    def test_live_repository_does_not_name_superseded_drafts(self) -> None:
        """Current canonical files pass the coverage-authority pointer contract."""

        documentation.validate_promotion_authority_pointers()

    def test_validate_promotion_authority_pointers_raises_on_stale_files(
        self,
    ) -> None:
        """File-backed validation fail-closes when either canonical file is stale."""

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "DOCUMENTATION.md").write_text(
                "The landable coverage gate is PR #97\n",
                encoding="utf-8",
            )
            assessment = root / "docs" / "DOCUMENTATION_ASSESSMENT.md"
            assessment.parent.mkdir(parents=True)
            assessment.write_text(
                "- **active-PR:** PR #94 `prediction_contradiction`\n",
                encoding="utf-8",
            )
            with mock.patch.object(documentation, "ROOT", root):
                with self.assertRaises(AssertionError) as raised:
                    documentation.validate_promotion_authority_pointers()
        self.assertIn("superseded draft", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
