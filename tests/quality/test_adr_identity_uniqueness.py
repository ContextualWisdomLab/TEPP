"""Regression tests for repository-wide ADR identity uniqueness."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import validate_documentation as docs


ADR_BODY = """# ADR {number} — Test decision

**Decision status:** Accepted
**Implementation maturity:** partial
**Supersession:** None.

## Context

Context.

## Decision

Decision.

## Alternatives considered

Alternative.

## Consequences

Consequence.

## Verification

Verification.

## Rollback

Rollback.
"""


class AdrIdentityUniquenessTests(unittest.TestCase):
    """Reject branch-local reuse or misdirection of an ADR identifier."""

    def _root(self, index_rows: str, files: dict[str, str]) -> Path:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        adr_root = root / "docs" / "adr"
        adr_root.mkdir(parents=True)
        (adr_root / "README.md").write_text(
            "| ADR | Decision | Decision status | Implementation maturity | Clarification |\n"
            "|---|---|---|---|---|\n"
            + index_rows,
            encoding="utf-8",
        )
        for name, number in files.items():
            (adr_root / name).write_text(ADR_BODY.format(number=number), encoding="utf-8")
        return root

    def test_duplicate_index_rows_fail(self) -> None:
        """Two index rows may not claim the same ADR number."""

        root = self._root(
            "| [0001](0001-one.md) | One | Accepted | partial | first |\n"
            "| [0001](0001-one.md) | One again | Accepted | partial | duplicate |\n",
            {"0001-one.md": "0001"},
        )
        with mock.patch.object(docs, "ROOT", root):
            with self.assertRaisesRegex(AssertionError, "duplicate ADR index identity"):
                docs.validate_adr_graph()

    def test_duplicate_numbered_files_fail(self) -> None:
        """Two numbered root ADR files may not share one repository-wide identity."""

        root = self._root(
            "| [0001](0001-one.md) | One | Accepted | partial | canonical |\n",
            {"0001-one.md": "0001", "0001-two.md": "0001"},
        )
        with mock.patch.object(docs, "ROOT", root):
            with self.assertRaisesRegex(AssertionError, "duplicate ADR file identity"):
                docs.validate_adr_graph()

    def test_mismatched_index_link_identity_fails(self) -> None:
        """Displayed ADR identity must match the linked numbered filename."""

        root = self._root(
            "| [0001](0002-two.md) | One mislabeled | Accepted | partial | bad link |\n"
            "| [0002](0001-one.md) | Two mislabeled | Accepted | partial | bad link |\n",
            {"0001-one.md": "0001", "0002-two.md": "0002"},
        )
        with mock.patch.object(docs, "ROOT", root):
            with self.assertRaisesRegex(AssertionError, "ADR index target identity mismatch"):
                docs.validate_adr_graph()

    def test_duplicate_index_targets_fail(self) -> None:
        """Two index identities may not resolve to the same canonical ADR target."""

        root = self._root(
            "| [0001](0001-one.md) | One | Accepted | partial | canonical |\n"
            "| [0002](0001-one.md) | Two | Accepted | partial | duplicate target |\n",
            {"0001-one.md": "0001", "0002-two.md": "0002"},
        )
        with mock.patch.object(docs, "ROOT", root):
            with self.assertRaisesRegex(AssertionError, "duplicate ADR index target"):
                docs.validate_adr_graph()

    def test_titled_duplicate_index_rows_fail(self) -> None:
        """Markdown link titles may not hide a duplicate ADR identity."""

        root = self._root(
            '| [0001](0001-one.md "canonical") | One | Accepted | partial | first |\n'
            '| [0001](0001-one.md "duplicate") | One again | Accepted | partial | duplicate |\n',
            {"0001-one.md": "0001"},
        )
        with mock.patch.object(docs, "ROOT", root):
            with self.assertRaisesRegex(AssertionError, "duplicate ADR index identity"):
                docs.validate_adr_graph()

    def test_canonical_index_targets_pass(self) -> None:
        """Each displayed identity may link to its unique root ADR file."""

        root = self._root(
            "| [0001](0001-one.md) | One | Accepted | partial | canonical |\n"
            "| [0002](0002-two.md) | Two | Accepted | partial | canonical |\n",
            {"0001-one.md": "0001", "0002-two.md": "0002"},
        )
        with mock.patch.object(docs, "ROOT", root):
            docs.validate_adr_graph()


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
