"""Apply and verify the bounded PR 155 review-finding repair."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def _run(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    """Run one repository command and surface its complete captured output."""

    completed = subprocess.run(
        args,
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    if completed.stdout:
        print(completed.stdout, end="")
    if completed.stderr:
        print(completed.stderr, end="", file=sys.stderr)
    if check and completed.returncode != 0:
        raise SystemExit(completed.returncode)
    return completed


def _replace_once(text: str, old: str, new: str, *, label: str) -> str:
    """Replace one known fragment or fail closed when the branch moved."""

    if new in text:
        return text
    if text.count(old) != 1:
        raise SystemExit(f"refusing unknown {label} shape")
    return text.replace(old, new, 1)


def _add_regressions() -> None:
    """Add malformed LLVM coverage records before changing the parser."""

    path = ROOT / "tests/quality/test_check_coverage.py"
    text = path.read_text(encoding="utf-8")
    old = '''            ([{"filename": "", "branches": []}], "must contain a filename"),
            ([{"filename": "src.rs", "branches": {}}], "branches must be a list"),'''
    new = '''            ([{"filename": "", "branches": []}], "must contain a filename"),
            ([{"filename": "src.rs"}], "must contain branches"),
            ([{"filename": "src.rs", "branches": {}}], "branches must be a list"),
            (
                [{"filename": "src.rs", "branches": [[True, 2, 3, 4, 1, 0]]}],
                "coordinates are invalid",
            ),
            (
                [{"filename": "src.rs", "branches": [[1, 2, 3, 4, 0.5, 0]]}],
                "counts are invalid",
            ),'''
    text = _replace_once(text, old, new, label="malformed coverage fixture")
    path.write_text(text, encoding="utf-8")


def _apply_repair() -> None:
    """Apply the strict parser, public constant, and documentation corrections."""

    coverage_path = ROOT / "scripts/check_coverage.py"
    coverage = coverage_path.read_text(encoding="utf-8")
    coverage = coverage.replace(
        'outcomes: dict[tuple[str, int, int, int, int], list[int | float]] = {}',
        'outcomes: dict[tuple[str, int, int, int, int], list[int]] = {}',
        1,
    )
    old_record = '''        filename = file_record.get("filename")
        branches = file_record.get("branches", [])
        if not isinstance(filename, str) or not filename:
            raise ValueError("coverage file record must contain a filename")
        if not isinstance(branches, list):'''
    new_record = '''        filename = file_record.get("filename")
        if "branches" not in file_record:
            raise ValueError("coverage file record must contain branches")
        branches = file_record["branches"]
        if not isinstance(filename, str) or not filename:
            raise ValueError("coverage file record must contain a filename")
        if not isinstance(branches, list):'''
    coverage = _replace_once(coverage, old_record, new_record, label="coverage file record")
    old_scalars = '''            if not all(isinstance(value, int) and value >= 0 for value in coordinates):
                raise ValueError("coverage branch coordinates are invalid")
            if not all(
                isinstance(value, (int, float))
                and not isinstance(value, bool)
                and value >= 0
                for value in counts
            ):'''
    new_scalars = '''            if not all(
                isinstance(value, int) and not isinstance(value, bool) and value >= 0
                for value in coordinates
            ):
                raise ValueError("coverage branch coordinates are invalid")
            if not all(
                isinstance(value, int) and not isinstance(value, bool) and value >= 0
                for value in counts
            ):'''
    coverage = _replace_once(coverage, old_scalars, new_scalars, label="coverage scalar validation")
    coverage_path.write_text(coverage, encoding="utf-8")

    contract_path = ROOT / "crates/tepp_api/tests/lineageweave_http_contract.rs"
    contract = contract_path.read_text(encoding="utf-8")
    old_import = '''    ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH, NARUON_LIVE_HEADER_BYTE_LIMIT,
    lineageweave_analysis_run_exchange,'''
    new_import = '''    ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE,
    NARUON_LIVE_HEADER_BYTE_LIMIT, lineageweave_analysis_run_exchange,'''
    contract = _replace_once(contract, old_import, new_import, label="Naruon consumer import")
    contract = _replace_once(
        contract,
        'let naruon = service.handle_http_request(&http_request("naruon", &run));',
        'let naruon = service.handle_http_request(&http_request(NARUON_CONSUMER_CODE, &run));',
        label="Naruon consumer use",
    )
    contract_path.write_text(contract, encoding="utf-8")

    adr_path = ROOT / "docs/adr/0017-consumer-scoped-analysis-run-ingress.md"
    adr = adr_path.read_text(encoding="utf-8")
    old_claim = (
        "An HTTP `202 Accepted` response means only that TEPP accepted a durable "
        "analysis-run identity for later execution. It is not a completed temporal "
        "model, calibrated score, theta estimate, uncertainty statement, or scientific claim."
    )
    new_claim = (
        "An HTTP `202 Accepted` response means only that TEPP accepted an analysis-run "
        "identity for later execution. In the current loopback proof the accepted-run "
        "registry is in-memory and is not durable across restarts; persistence remains "
        "separate work. The response is not a completed temporal model, calibrated score, "
        "theta estimate, uncertainty statement, or scientific claim."
    )
    adr = _replace_once(adr, old_claim, new_claim, label="ADR durability claim")
    adr_path.write_text(adr, encoding="utf-8")

    validator_path = ROOT / "scripts/validate_documentation.py"
    validator = validator_path.read_text(encoding="utf-8")
    anchor = '    "docs/adr/0016-tdt-chronos-event-intelligence-boundary.md",\n'
    replacement = anchor + '    "docs/adr/0017-consumer-scoped-analysis-run-ingress.md",\n'
    validator = _replace_once(validator, anchor, replacement, label="documentation ADR inventory")
    validator_path.write_text(validator, encoding="utf-8")

    changelog_path = ROOT / "CHANGELOG.md"
    changelog = changelog_path.read_text(encoding="utf-8")
    added = (
        "- `tepp_api` LineageWeave consumer-scoped analysis-run ingress: versioned, "
        "credential-free requests use a published consumer identity and isolate idempotency "
        "by consumer, tenant workspace, and opaque caller key; the one-shot restack workflow "
        "is removed after the protected-main merge is verified.\n"
    )
    adr_entry = (
        "- ADR 0017 records the consumer-scoped analysis-run ingress, its in-memory "
        "loopback maturity, and the persistence boundary required before production use.\n"
    )
    if adr_entry not in changelog:
        if added not in changelog:
            raise SystemExit("refusing unknown changelog consumer-ingress entry")
        changelog = changelog.replace(added, added + adr_entry, 1)
    if "ADR 0001–0016" in changelog:
        changelog = changelog.replace("ADR 0001–0016", "ADR 0001–0017")
    elif "ADR 0001-0016" in changelog:
        changelog = changelog.replace("ADR 0001-0016", "ADR 0001-0017")
    elif "ADR 0001–0017" not in changelog and "ADR 0001-0017" not in changelog:
        raise SystemExit("refusing unknown changelog ADR range")
    changelog_path.write_text(changelog, encoding="utf-8")


def main() -> None:
    """Prove RED, apply the repair, and prove focused GREEN."""

    _add_regressions()
    red = _run(
        sys.executable,
        "-m",
        "unittest",
        "tests.quality.test_check_coverage.CoverageContractTests.test_full_branch_reports_fail_closed_on_malformed_records",
        check=False,
    )
    if red.returncode == 0:
        raise SystemExit("coverage regressions unexpectedly passed before the parser repair")
    _apply_repair()
    _run(sys.executable, "-m", "unittest", "tests.quality.test_check_coverage")
    _run(sys.executable, "scripts/validate_documentation.py")
    _run("cargo", "fmt", "--check")
    _run("cargo", "test", "-p", "tepp_api", "--test", "lineageweave_http_contract")


if __name__ == "__main__":
    main()
