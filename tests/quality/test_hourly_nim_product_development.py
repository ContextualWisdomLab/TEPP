"""Lock the hourly released-orchestrator OpenCode security contract."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

WORKFLOW = Path(".github/workflows/hourly-nim-product-development.yml")
PARSER = Path("scripts/prepare_agent_pr_message.py")
RUNBOOK = Path("docs/operations/HOURLY_NIM_PRODUCT_DEVELOPMENT.md")
DOCTORING = Path("docs/doctoring/hourly-nim-opencode-development.md")
PRD_AMENDMENT = Path("docs/product/prd-v0.4.1-amendment-llm-routing.md")


def _text(path: Path) -> str:
    """Return one required UTF-8 contract file."""

    assert path.is_file(), f"required contract file is missing: {path}"
    return path.read_text(encoding="utf-8")


class HourlyOrchestratorProductDevelopmentContractTests(unittest.TestCase):
    """Structural tests for released-contract, credential-separated automation."""

    def test_workflow_uses_released_orchestrator_free_gateway_only(self) -> None:
        """Reject provider credentials, mutable source pins, and consumer-side routing."""

        text = _text(WORKFLOW)
        for token in (
            'cron: "47 * * * *"',
            "workflow_dispatch:",
            "dry_run:",
            "hourly-nim-product-development-${{ github.repository }}",
            "cancel-in-progress: false",
            "CONTEXTUAL_ORCHESTRATOR_RELEASE",
            "CONTEXTUAL_ORCHESTRATOR_BASE_URL",
            "secrets.CONTEXTUAL_ORCHESTRATOR_GATEWAY_TOKEN",
            "orchestrator/free",
            "repos/ContextualWisdomLab/contextual-orchestrator/releases/tags/",
            "contextual_orchestrator_release_unavailable",
            "contextual_orchestrator_gateway_unavailable",
            "pull_request_inventory_unavailable",
            "open_pull_request",
            "issue_inventory_unavailable",
            "open_issue",
            "maintainer_app_unavailable",
            "base_branch_advanced",
            "open_pull_request_after_generation",
            "open_issue_after_generation",
        ):
            self.assertIn(token, text)

        for forbidden in (
            "BYTEZ_API_KEY",
            "NVIDIA_NIM_API_KEY",
            "NVIDIA_NIM_API_KEY_SUB",
            "OPENROUTER_API_KEY",
            "OPENAI_API_KEY",
            "CONTEXTUAL_ORCHESTRATOR_COMMIT",
            "CONTEXTUAL_ORCHESTRATOR_SHA256",
            "run_contextual_orchestrator.py",
            "select_top_n_cheapest_discovered_agents",
            "OPENCODE_RUN_TIMEOUT_SECONDS",
            "timeout --kill-after",
            "COPILOT_GITHUB_TOKEN",
        ):
            self.assertNotIn(forbidden, text)

        self.assertIn(".immutable == true", text)
        self.assertEqual(text.count("gh pr create"), 1)
        self.assertNotIn("gh pr merge", text)
        self.assertNotIn("gh release create", text)

    def test_workflow_separates_proposal_verification_and_publication_authority(self) -> None:
        """Only the publisher receives repository mutation authority."""

        text = _text(WORKFLOW)
        proposer = text.split("propose_product_increment:", 1)[1].split(
            "package_product_increment:", 1
        )[0]
        verifier = text.split("package_product_increment:", 1)[1].split(
            "publish_product_increment:", 1
        )[0]
        publisher = text.split("publish_product_increment:", 1)[1]

        self.assertIn("CONTEXTUAL_ORCHESTRATOR_GATEWAY_TOKEN", proposer)
        self.assertIn("orchestrator/free", proposer)
        self.assertNotIn("create-github-app-token", proposer)
        self.assertNotIn("gh pr create", proposer)
        self.assertNotIn("CONTEXTUAL_ORCHESTRATOR_GATEWAY_TOKEN", verifier)
        self.assertNotIn("create-github-app-token", verifier)
        self.assertIn("Run every release-quality gate", verifier)
        self.assertNotIn("CONTEXTUAL_ORCHESTRATOR_GATEWAY_TOKEN", publisher)
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

    def test_workflow_binds_artifacts_and_strips_runtime_mutation_channels(self) -> None:
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

    def test_prompt_and_verifier_keep_scientific_and_commercial_gates(self) -> None:
        """Require one buyer gap, released orchestration, and full verification."""

        text = _text(WORKFLOW)
        normalized = " ".join(text.casefold().split())
        for token in (
            "buyer-visible",
            "product-technical-gap-baseline.md",
            "gap id",
            "never invent weights",
            "no heuristics",
            "primary source",
            "exactly one bounded pull request",
            "standalone",
            "modular msa",
            "contextual-orchestrator",
            "released",
            "orchestrator/free",
            "fugu",
            "conductor",
            "trinity",
            "reasoning effort",
            "ablation",
            "100% production statement and branch coverage",
            "100% public docstring coverage",
            "two-word-or-longer snake_case",
            "apa 7",
            "do not merge",
            "do not release",
            "do not deploy",
            "rust",
        ):
            self.assertIn(token, normalized)

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

    def test_runbook_and_prd_amendment_match_released_owner_boundary(self) -> None:
        """Keep operations and product authority free of provider-key bootstrap."""

        runbook = _text(RUNBOOK)
        amendment = _text(PRD_AMENDMENT)
        doctoring = _text(DOCTORING)
        combined = runbook + amendment
        for token in (
            "released",
            "contextual-orchestrator",
            "orchestrator/free",
            "gateway credential",
            "fail closed",
        ):
            self.assertIn(token, combined.casefold())
        for forbidden in (
            "NVIDIA_NIM_API_KEY",
            "OPENROUTER_API_KEY",
            "OPENAI_API_KEY",
            "BYTEZ_API_KEY",
        ):
            self.assertNotIn(forbidden, runbook)
        self.assertIn("APA", doctoring)

    def test_parser_accepts_unicode_and_owner_only_outputs(self) -> None:
        """Parse realistic Korean metadata and protect trusted output files."""

        import scripts.prepare_agent_pr_message as parser

        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            source = tmp_path / "PR_MESSAGE.md"
            title_path = tmp_path / "title.txt"
            body_path = tmp_path / "body.md"
            source.write_text(
                "feat: 시간별 오케스트레이터 제품 개발 루프 추가\r\n\r\n"
                "구매자가 체감하는 제품 Gap 하나를 안전하게 닫습니다.\r\n",
                encoding="utf-8",
            )
            parser.main([str(source), str(title_path), str(body_path)])
            self.assertEqual(
                title_path.read_text(encoding="utf-8"),
                "feat: 시간별 오케스트레이터 제품 개발 루프 추가",
            )
            self.assertIn("구매자가 체감하는 제품 Gap", body_path.read_text(encoding="utf-8"))
            for path in (title_path, body_path):
                self.assertEqual(path.stat().st_mode & 0o777, 0o600)


if __name__ == "__main__":
    unittest.main()
