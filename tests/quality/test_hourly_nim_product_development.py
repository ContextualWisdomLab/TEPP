"""Lock the hourly NVIDIA NIM OpenCode development security contract."""

from __future__ import annotations

import unittest
from pathlib import Path
from types import ModuleType

WORKFLOW = Path(".github/workflows/hourly-nim-product-development.yml")
PARSER = Path("scripts/prepare_agent_pr_message.py")
RUNBOOK = Path("docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md")
DOCTORING = Path("docs/doctoring/hourly-nim-opencode-development.md")


def _text(path: Path) -> str:
    """Return one required UTF-8 contract file."""

    assert path.is_file(), f"required contract file is missing: {path}"
    return path.read_text(encoding="utf-8")


def _parser_module() -> ModuleType:
    """Load the trusted pull-request metadata parser as a covered module."""

    assert PARSER.is_file()
    import scripts.prepare_agent_pr_message as module

    return module


class HourlyNimProductDevelopmentContractTests(unittest.TestCase):
    """Structural tests for the credential-separated product-development loop."""

    def test_hourly_workflow_schedule_credentials_and_queue_gate(self) -> None:
        """Run at minute 47 with NIM only and fail closed around PR inventory."""

        text = _text(WORKFLOW)
        for token in (
            'cron: "47 * * * *"',
            "workflow_dispatch:",
            "dry_run:",
            "hourly-nim-product-development-${{ github.repository }}",
            "cancel-in-progress: false",
            "secrets.NVIDIA_NIM_API_KEY",
            "{env:NVIDIA_NIM_API_KEY}",
            "OPENCODE_VERSION",
            "OPENCODE_SHA256",
            "sha256sum -c",
            "pull_request_inventory_unavailable",
            "open_pull_request",
            "nim_api_key_unavailable",
            "maintainer_app_unavailable",
            "base_branch_advanced",
            "open_pull_request_after_generation",
            "ContextualWisdomLab/TEPP",
        ):
            self.assertIn(token, text)
        self.assertNotIn("COPILOT_GITHUB_TOKEN", text)
        self.assertNotIn("CONTEXTUAL_ORCHESTRATOR_TOKEN", text)
        self.assertEqual(text.count("gh pr create"), 1)
        self.assertNotIn("gh pr merge", text)
        self.assertNotIn("gh release create", text)

    def test_hourly_workflow_separates_three_runner_trust_boundaries(self) -> None:
        """Separate model execution, verification, and late publication authority."""

        text = _text(WORKFLOW)
        proposer = text.split("propose_product_increment:", 1)[1].split(
            "package_product_increment:", 1
        )[0]
        verifier = text.split("package_product_increment:", 1)[1].split(
            "publish_product_increment:", 1
        )[0]
        publisher = text.split("publish_product_increment:", 1)[1]

        self.assertIn("NVIDIA_NIM_API_KEY", proposer)
        self.assertNotIn("create-github-app-token", proposer)
        self.assertNotIn("gh pr create", proposer)
        self.assertNotIn("NVIDIA_NIM_API_KEY", verifier)
        self.assertNotIn("create-github-app-token", verifier)
        self.assertIn("Run every release-quality gate", verifier)
        self.assertNotIn("NVIDIA_NIM_API_KEY", publisher)
        self.assertIn("create-github-app-token", publisher)
        self.assertIn("gh pr create", publisher)
        self.assertNotIn("cargo test", publisher)
        self.assertNotIn("pytest ", publisher)
        self.assertLess(
            text.index("Preserve trusted metadata parser"),
            text.index("Verify and apply immutable proposal without executing it"),
        )
        self.assertLess(
            text.index("Parse bounded untrusted pull-request metadata"),
            text.index("Mint dedicated maintainer App token only for publication"),
        )

    def test_hourly_workflow_binds_artifacts_and_strips_runtime_channels(self) -> None:
        """Bind the patch exactly and remove untrusted GitHub mutation channels."""

        text = _text(WORKFLOW)
        for token in (
            "artifact-id",
            "artifact-digest",
            "patch_sha256",
            "changed_files",
            "diff_bytes",
            "MAX_CHANGED_FILES",
            "MAX_DIFF_BYTES",
            "120000",
            "160000",
            "git diff --cached --check",
            "git apply --check --binary",
            "retention-days: 1",
            "overwrite: false",
            "-u GH_TOKEN",
            "-u GITHUB_TOKEN",
            "-u ACTIONS_ID_TOKEN_REQUEST_TOKEN",
            "-u ACTIONS_RUNTIME_TOKEN",
            "-u ACTIONS_RESULTS_URL",
            "-u ACTIONS_CACHE_URL",
            "-u GITHUB_ENV",
            "-u GITHUB_OUTPUT",
            "-u GITHUB_PATH",
            "-u GITHUB_STATE",
            "-u GITHUB_STEP_SUMMARY",
            "timeout --kill-after",
            '"webfetch": "deny"',
            '"websearch": "deny"',
            '"external_directory": "deny"',
            '"task": "deny"',
            '"git push *": "deny"',
            '"git tag *": "deny"',
            '"gh *": "deny"',
            "TEPP_MAINTAINER_APP_CLIENT_ID",
            "TEPP_MAINTAINER_APP_PRIVATE_KEY",
        ):
            self.assertIn(token, text)
        self.assertEqual(text.count("artifact-ids:"), 2)

    def test_hourly_prompt_and_verifier_keep_commercial_quality_gates(self) -> None:
        """Require one buyer gap, research-grounded orchestration, and full checks."""

        text = _text(WORKFLOW)
        normalized = " ".join(text.casefold().split())
        for token in (
            "buyer-visible",
            "exactly one bounded pull request",
            "standalone",
            "modular MSA",
            "ContextualWisdomLab/.github",
            "naruon",
            "contextual-orchestrator",
            "Fugu",
            "Conductor",
            "TRINITY",
            "single-model",
            "deep multi-agent",
            "reasoning effort",
            "access lists",
            "recursive depth",
            "ablation",
            "Speed is not a priority",
            "100% production statement and branch coverage",
            "100% public docstring coverage",
            "two-word-or-longer snake_case",
            "APA 7",
            "CHANGELOG.md",
            "Do not merge",
            "Do not release",
            "Do not deploy",
            "Rust",
        ):
            self.assertIn(token.casefold(), normalized)

        verifier = text.split("package_product_increment:", 1)[1].split(
            "publish_product_increment:", 1
        )[0]
        for command in (
            "python3 scripts/check_workspace_contract.py",
            "python3 scripts/check_docstrings.py",
            "cargo clippy --workspace --all-targets --all-features -- -D warnings",
            "cargo nextest run --workspace --all-features",
            "cargo deny check",
            'line_coverage="$RUNNER_TEMP/coverage.lcov"',
            'branch_coverage="$RUNNER_TEMP/coverage-branches.json"',
            'python3 scripts/check_coverage.py "$line_coverage" --kind lines --format lcov',
            'python3 scripts/check_coverage.py "$branch_coverage" --kind branches',
        ):
            self.assertIn(command, verifier)

    def test_parser_accepts_unicode_and_owner_only_outputs(self) -> None:
        """Parse realistic Korean metadata and protect trusted output files."""

        import tempfile

        parser = _parser_module()
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            source = tmp_path / "PR_MESSAGE.md"
            title_path = tmp_path / "title.txt"
            body_path = tmp_path / "body.md"
            source.write_text(
                "feat: 시간별 NIM 제품 개발 루프 추가\r\n\r\n"
                "구매자가 체감하는 제품 Gap 하나를 안전하게 닫습니다.\r\n",
                encoding="utf-8",
            )
            parser.main(
                [
                    str(source),
                    str(title_path),
                    str(body_path),
                ]
            )
            self.assertEqual(
                title_path.read_text(encoding="utf-8"),
                "feat: 시간별 NIM 제품 개발 루프 추가",
            )
            self.assertIn("구매자가 체감하는 제품 Gap", body_path.read_text(encoding="utf-8"))
            for path in (title_path, body_path):
                self.assertEqual(path.stat().st_mode & 0o777, 0o600)

    def test_supporting_runbook_and_doctoring_exist(self) -> None:
        """Keep operations and research doctoring discoverable."""

        runbook = _text(RUNBOOK)
        doctoring = _text(DOCTORING)
        for token in ("NVIDIA_NIM_API_KEY", "proposal", "verification", "publication"):
            self.assertIn(token, runbook)
        self.assertIn("APA", doctoring)
        self.assertIn("Do not configure `COPILOT_GITHUB_TOKEN`", runbook)

    def test_hourly_queue_keeps_weaker_coverage_locks_unmerged(self) -> None:
        """A runner must not treat #104, #108, #109, #111, or #112 as the landable gate."""

        runbook = _text(RUNBOOK)
        unmerged_sentences = [
            sentence
            for sentence in runbook.replace("\n", " ").split(".")
            if "unmerged" in sentence.casefold()
        ]
        joined = " ".join(unmerged_sentences)
        for pull_request in (93, 94, 97, 101, 102, 104, 108, 109, 111, 112):
            with self.subTest(pull_request=pull_request):
                self.assertIn(f"PR #{pull_request}", joined)
        self.assertIn("PR #107", runbook)
        self.assertIn("PR #105", joined)
        self.assertIn("PR #87", joined)
        self.assertNotIn("coverage-authority landing PR #", runbook.casefold())
        self.assertIn("`prediction_contradiction`", runbook)


if __name__ == "__main__":
    unittest.main()
