"""Lock the hourly contextual-orchestrator OpenCode security contract."""

from __future__ import annotations

from dataclasses import dataclass
import os
import sys
import unittest
from pathlib import Path
from types import ModuleType
from unittest.mock import patch

WORKFLOW = Path(".github/workflows/hourly-nim-product-development.yml")
BOOTSTRAP = Path("scripts/run_contextual_orchestrator.py")
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
        """Run at minute 47 with provider discovery and fail closed around inventory."""

        text = _text(WORKFLOW)
        bootstrap = _text(BOOTSTRAP)
        for token in (
            'cron: "47 * * * *"',
            "workflow_dispatch:",
            "dry_run:",
            "hourly-nim-product-development-${{ github.repository }}",
            "cancel-in-progress: false",
            "secrets.BYTEZ_API_KEY",
            "secrets.NVIDIA_NIM_API_KEY",
            "secrets.NVIDIA_NIM_API_KEY_SUB",
            "secrets.OPENROUTER_API_KEY",
            "secrets.OPENAI_API_KEY",
            "CONTEXTUAL_ORCHESTRATOR_COMMIT",
            "CONTEXTUAL_ORCHESTRATOR_SHA256",
            "run_contextual_orchestrator.py",
            "/healthz",
            "/v1/models",
            "{env:OPENCODE_GATEWAY_TOKEN}",
            "OPENCODE_VERSION",
            "OPENCODE_SHA256",
            "sha256sum -c",
            "pull_request_inventory_unavailable",
            "open_pull_request",
            "issue_inventory_unavailable",
            "open_issue",
            "contextual_orchestrator_credentials_unavailable",
            "maintainer_app_unavailable",
            "base_branch_advanced",
            "open_pull_request_after_generation",
            "issue_inventory_unavailable_after_generation",
            "open_issue_after_generation",
            "ContextualWisdomLab/TEPP",
        ):
            self.assertIn(token, text)
        for token in ("discover_all_models", "register_credential", "PROVIDER_CREDENTIAL_NAMES"):
            self.assertIn(token, bootstrap)
        self.assertNotIn("COPILOT_GITHUB_TOKEN", text)
        self.assertNotIn("CONTEXTUAL_ORCHESTRATOR_TOKEN=", text)
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

        self.assertIn("BYTEZ_API_KEY", proposer)
        self.assertIn("OPENAI_API_KEY", proposer)
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
                "feat: 시간별 오케스트레이터 제품 개발 루프 추가\r\n\r\n"
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
                "feat: 시간별 오케스트레이터 제품 개발 루프 추가",
            )
            self.assertIn("구매자가 체감하는 제품 Gap", body_path.read_text(encoding="utf-8"))
            for path in (title_path, body_path):
                self.assertEqual(path.stat().st_mode & 0o777, 0o600)

    def test_supporting_runbook_and_doctoring_exist(self) -> None:
        """Keep operations and research doctoring discoverable."""

        runbook = _text(RUNBOOK)
        doctoring = _text(DOCTORING)
        for token in ("OPENAI_API_KEY", "contextual-orchestrator", "proposal", "verification", "publication"):
            self.assertIn(token, runbook)
        self.assertIn("APA", doctoring)
        self.assertIn("Do not configure `COPILOT_GITHUB_TOKEN`", runbook)

    def test_bootstrap_registers_each_provider_key_and_removes_environment_values(self) -> None:
        """Exercise the real bootstrap loop with a key-counting KV double."""

        import scripts.run_contextual_orchestrator as bootstrap

        calls: list[tuple[str, str]] = []
        fake_package = ModuleType("contextual_orchestrator")
        fake_package.register_credential = lambda name, value: calls.append((name, value))
        values = {name: f"test-value-{name}" for name in bootstrap.PROVIDER_CREDENTIAL_NAMES}
        with patch.dict(sys.modules, {"contextual_orchestrator": fake_package}), patch.dict(
            os.environ, values, clear=False
        ):
            bootstrap._register_bootstrap_credentials()
            self.assertEqual(
                calls,
                [(name, values[name]) for name in bootstrap.PROVIDER_CREDENTIAL_NAMES],
            )
            for name in bootstrap.PROVIDER_CREDENTIAL_NAMES:
                self.assertNotIn(name, os.environ)

    def test_bootstrap_fails_closed_when_one_provider_key_is_missing(self) -> None:
        """Reject incomplete provider bootstrap without silently selecting a subset."""

        import scripts.run_contextual_orchestrator as bootstrap

        fake_package = ModuleType("contextual_orchestrator")
        fake_package.register_credential = lambda _name, _value: None
        values = {
            name: ("present" if index == 0 else "")
            for index, name in enumerate(bootstrap.PROVIDER_CREDENTIAL_NAMES)
        }
        with patch.dict(sys.modules, {"contextual_orchestrator": fake_package}), patch.dict(
            os.environ, values, clear=False
        ):
            with self.assertRaisesRegex(RuntimeError, "missing provider credentials"):
                bootstrap._register_bootstrap_credentials()

    def test_discovery_selection_and_empty_provider_fail_closed_paths(self) -> None:
        """Select discovered candidates and reject a discovery result with no models."""

        import scripts.run_contextual_orchestrator as bootstrap

        @dataclass(frozen=True)
        class FakeAgent:
            """Small dataclass matching the fields changed by dataclasses.replace."""

            model: str
            priority: int = 0
            disabled: bool = True

        class FakePriceBook:
            """Minimal price-book constructor accepted by the selection seam."""

            def __init__(self, _store: object) -> None:
                pass

        sample = type("SampleModel", (), {"model_id": "model_one", "provider_name": "provider_one"})()
        error = type("SampleError", (), {"provider_name": "provider_two"})()
        fake_package = ModuleType("contextual_orchestrator")
        fake_package.InMemoryConfigStore = object
        fake_package.PriceBook = FakePriceBook
        fake_discovery = ModuleType("contextual_orchestrator.model_discovery")
        fake_discovery.agent_from_discovered = lambda model, priority=0: FakeAgent(
            model.model_id, priority=priority
        )
        fake_discovery.discover_all_models = lambda: ([sample], [error])
        fake_discovery.refresh_price_book = lambda _models, _book: 1
        fake_discovery.select_top_n_cheapest_discovered_agents = lambda models, _book, _limit: models
        fake_modules = {
            "contextual_orchestrator": fake_package,
            "contextual_orchestrator.model_discovery": fake_discovery,
        }
        with patch.dict(sys.modules, fake_modules):
            agents, report = bootstrap._selected_agents()
            self.assertEqual([agent.model for agent in agents], ["model_one"])
            self.assertFalse(agents[0].disabled)
            self.assertEqual(report["providers_with_errors"], ["provider_two"])

            fake_discovery.discover_all_models = lambda: ([], [])
            with self.assertRaisesRegex(RuntimeError, "providers_with_errors=none"):
                bootstrap._selected_agents()

    def test_report_gateway_and_main_contract_are_executable_with_seams(self) -> None:
        """Cover report permissions, gateway construction, and CLI orchestration."""

        import tempfile

        import scripts.run_contextual_orchestrator as bootstrap

        class FakeOrchestrator:
            """Capture the selected agent pool passed to the gateway runtime."""

            def __init__(self, agents: list[object]) -> None:
                self.agents = agents

        class FakeSecurity:
            """Capture the loopback bearer token passed to the HTTP server."""

            def __init__(self, auth_token: str) -> None:
                self.auth_token = auth_token

        server_calls: list[tuple[object, str, int, object]] = []
        fake_package = ModuleType("contextual_orchestrator")
        fake_package.TaskOrchestrator = FakeOrchestrator
        fake_server = ModuleType("contextual_orchestrator.server")
        fake_server.SecurityConfig = FakeSecurity
        fake_server.serve = lambda orchestrator, *, host, port, security: server_calls.append(
            (orchestrator, host, port, security)
        )
        with patch.dict(
            sys.modules,
            {"contextual_orchestrator": fake_package, "contextual_orchestrator.server": fake_server},
        ):
            with tempfile.TemporaryDirectory() as tmp:
                report_path = Path(tmp) / "nested" / "discovery.json"
                bootstrap._write_report(report_path, {"discovered_count": 1})
                self.assertEqual(report_path.read_text(encoding="utf-8"), '{"discovered_count": 1}\n')
                self.assertEqual(report_path.stat().st_mode & 0o777, 0o600)

            bootstrap._start_gateway(["agent"], "gateway-token", "127.0.0.1", 18000)
            self.assertEqual(server_calls[0][0].agents, ["agent"])
            self.assertEqual(server_calls[0][1], "127.0.0.1")
            self.assertEqual(server_calls[0][2], 18000)
            self.assertEqual(server_calls[0][3].auth_token, "gateway-token")

        with patch.object(bootstrap, "_register_bootstrap_credentials"), patch.object(
            bootstrap, "_selected_agents", return_value=([], {"discovered_count": 0})
        ) as selected, patch.object(bootstrap, "_write_report") as written, patch.object(
            bootstrap, "_start_gateway"
        ) as started, patch.dict(
            os.environ, {"CONTEXTUAL_ORCHESTRATOR_INFERENCE_TOKEN": "gateway-token"}, clear=False
        ), patch.object(sys, "argv", ["run_contextual_orchestrator.py", "--report", "report.json"]):
            bootstrap.main()
            selected.assert_called_once_with()
            written.assert_called_once_with(Path("report.json"), {"discovered_count": 0})
            started.assert_called_once_with([], "gateway-token", "127.0.0.1", 18000)

        with patch.object(bootstrap, "_register_bootstrap_credentials"), patch.dict(
            os.environ, {}, clear=False
        ), patch.object(sys, "argv", ["run_contextual_orchestrator.py", "--report", "report.json"]):
            with self.assertRaisesRegex(RuntimeError, "INFERENCE_TOKEN is required"):
                bootstrap.main()


if __name__ == "__main__":
    unittest.main()
