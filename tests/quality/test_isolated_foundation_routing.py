"""Foundation and docs select the dedicated untrusted-work runner boundary."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXPECTED_SELECTOR = (
    "    runs-on:\n"
    "      group: CWL CI isolated\n"
    "      labels: [self-hosted, linux, x64, cwlab-ci-isolated]\n"
)


class IsolatedFoundationRoutingTests(unittest.TestCase):
    """Source routing never borrows the credential-bearing control pool."""

    def test_all_five_validation_jobs_require_group_and_capability(self) -> None:
        """Both group and all four labels are required for every local CI job."""
        for name, expected_count in (("ci.yml", 4), ("docs-quality.yml", 1)):
            with self.subTest(workflow=name):
                workflow = (ROOT / ".github/workflows" / name).read_text(encoding="utf-8")
                self.assertEqual(workflow.count(EXPECTED_SELECTOR), expected_count)
                self.assertEqual(len(re.findall(r"(?m)^    runs-on:", workflow)), expected_count)
                self.assertNotIn("runs-on: ubuntu-latest", workflow)
                self.assertNotIn("cwlab-control", workflow)

    def test_untrusted_jobs_keep_read_only_permissions_and_checkout(self) -> None:
        """Validation workflows retain PR events and credential-free checkout."""
        for name, expected_count in (("ci.yml", 4), ("docs-quality.yml", 1)):
            with self.subTest(workflow=name):
                workflow = (ROOT / ".github/workflows" / name).read_text(encoding="utf-8")
                self.assertIn("  pull_request:\n", workflow)
                self.assertNotIn("pull_request_target", workflow)
                self.assertIn("permissions:\n  contents: read\n", workflow)
                self.assertNotIn("secrets.", workflow)
                self.assertEqual(workflow.count("persist-credentials: false"), expected_count)

    def test_documented_consumer_jobs_match_actual_workflow_ids(self) -> None:
        """The operator scope table names every actual validation job exactly."""
        document = (ROOT / "docs/operations/ISOLATED_FOUNDATION_RUNNERS.md").read_text(
            encoding="utf-8"
        )
        for name in ("ci.yml", "docs-quality.yml"):
            with self.subTest(workflow=name):
                workflow = (ROOT / ".github/workflows" / name).read_text(encoding="utf-8")
                job_block = workflow.split("\njobs:\n", 1)[1]
                actual = set(re.findall(r"(?m)^  ([a-z][a-z0-9-]*):$", job_block))
                rows = [
                    line for line in document.splitlines()
                    if line.startswith(f"| `.github/workflows/{name}` |")
                ]
                self.assertEqual(len(rows), 1)
                documented = set(re.findall(r"`([^`]+)`", rows[0].split("|")[2]))
                self.assertEqual(documented, actual)

    def test_hourly_secret_jobs_are_not_moved_into_public_validation_pool(self) -> None:
        """The separate hourly NIM/App-secret boundary is not adopted by this slice."""
        workflow = (ROOT / ".github/workflows/hourly-nim-product-development.yml").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("CWL CI isolated", workflow)
        self.assertNotIn("cwlab-ci-isolated", workflow)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
