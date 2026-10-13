"""Contract tests for the pinned TEPP reusable-workflow callers."""

from pathlib import Path
import re
import unittest

from scripts.effective_ci_source import effective_ci_source


ROOT = Path(__file__).parents[2]
EXPECTED_SHA = "af81ef49683809ce8b302e078455e3960cf0acd1"
EXPECTED_CALLERS = {
    ".github/workflows/ci.yml": {
        "job": "foundation",
        "supplier": "tepp-rust-foundation-ci.yml",
        "name": "TEPP Rust Foundation supplier",
        "gate_ids": (
            "repository-contracts",
            "rust-quality",
            "production-coverage",
            "live-postgres",
        ),
        "concurrency": (
            "tepp-rust-foundation-${{ github.repository }}-"
            "${{ github.event.pull_request.number || github.run_id }}"
        ),
    },
    ".github/workflows/docs-quality.yml": {
        "job": "documentation",
        "supplier": "tepp-documentation-quality.yml",
        "name": "TEPP Documentation supplier",
        "gate_ids": ("validate",),
        "concurrency": (
            "docs-quality-${{ github.repository }}-"
            "${{ github.event.pull_request.number || github.run_id }}"
        ),
    },
}


def _job_block(source: str, job_id: str) -> str:
    """Return one top-level job block from the deliberately small caller YAML."""
    match = re.search(
        rf"(?ms)^  {re.escape(job_id)}:\n(?P<body>.*?)(?=^  [A-Za-z0-9_-]+:|\Z)",
        source,
    )
    assert match, f"missing caller job {job_id}"
    return match.group("body")


def test_callers_pin_one_supplier_without_nested_execution() -> None:
    """Each consumer job delegates once and cannot duplicate supplier steps."""
    seen_suppliers = []
    for relative, expected in EXPECTED_CALLERS.items():
        source = (ROOT / relative).read_text(encoding="utf-8")
        block = _job_block(source, expected["job"])
        expected_use = (
            "    uses: ContextualWisdomLab/.github/.github/workflows/"
            f"{expected['supplier']}@{EXPECTED_SHA}"
        )
        assert block.count(expected_use) == 1
        assert "runs-on:" not in block
        assert "steps:" not in block
        assert "secrets:" not in block
        assert "with:" not in block
        assert block.count("uses:") == 1
        assert f"name: {expected['name']}" in block
        seen_suppliers.append(expected["supplier"])

    assert len(seen_suppliers) == 2
    assert len(set(seen_suppliers)) == 2
    assert sum(len(item["gate_ids"]) for item in EXPECTED_CALLERS.values()) == 5


def test_five_supplier_gates_are_coordinated_by_two_callers() -> None:
    """The foundation and documentation suppliers account for five unique gates."""
    gate_ids = []
    for expected in EXPECTED_CALLERS.values():
        verified = effective_ci_source(ROOT, expected["supplier"])
        supplier_jobs = verified.rsplit("\njobs:\n", 1)[1]
        actual_ids = re.findall(r"^  ([A-Za-z0-9_-]+):$", supplier_jobs, re.MULTILINE)
        assert set(actual_ids) == set(expected["gate_ids"])
        assert len(actual_ids) == len(expected["gate_ids"])
        gate_ids.extend(actual_ids)
    assert len(gate_ids) == 5
    assert len(set(gate_ids)) == 5
    assert set(gate_ids) == {
        "repository-contracts",
        "rust-quality",
        "production-coverage",
        "live-postgres",
        "validate",
    }


def test_original_trigger_concurrency_and_permissions_remain() -> None:
    """Thin callers retain the original event, coalescing, and read-only contract."""
    for relative, expected in EXPECTED_CALLERS.items():
        source = (ROOT / relative).read_text(encoding="utf-8")
        assert "on:\n  pull_request:" in source
        assert "  push:\n    branches:\n      - main" in source
        assert "  workflow_dispatch:\n" in source
        assert "permissions:\n  contents: read" in source
        assert f"  group: {expected['concurrency']}" in source
        assert "  cancel-in-progress: ${{ github.event_name == 'pull_request' }}" in source
        assert "secrets: inherit" not in source


def test_all_caller_references_are_immutable_and_unique() -> None:
    """No floating ref or duplicate central invocation is admitted."""
    references = []
    for relative in EXPECTED_CALLERS:
        source = (ROOT / relative).read_text(encoding="utf-8")
        references.extend(
            re.findall(
                r"uses: ContextualWisdomLab/.github/.github/workflows/[^\s]+",
                source,
            )
        )
    assert len(references) == 2
    assert all(f"@{EXPECTED_SHA}" in reference for reference in references)
    assert all("@main" not in reference for reference in references)
    assert len(set(references)) == 2


class ReusableSupplierCallerTests(unittest.TestCase):
    """Run the caller contracts under the supplier's unittest gate."""

    def test_callers_pin_one_supplier_without_nested_execution(self) -> None:
        test_callers_pin_one_supplier_without_nested_execution()

    def test_five_supplier_gates_are_coordinated_by_two_callers(self) -> None:
        test_five_supplier_gates_are_coordinated_by_two_callers()

    def test_original_trigger_concurrency_and_permissions_remain(self) -> None:
        test_original_trigger_concurrency_and_permissions_remain()

    def test_all_caller_references_are_immutable_and_unique(self) -> None:
        test_all_caller_references_are_immutable_and_unique()
