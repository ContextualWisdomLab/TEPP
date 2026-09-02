"""Regression tests for the hourly contextual-orchestrator free-route gate."""

from __future__ import annotations

from dataclasses import dataclass
import sys
import unittest
from types import ModuleType
from unittest.mock import patch

import scripts.run_contextual_orchestrator as bootstrap


@dataclass(frozen=True)
class FakeModel:
    """Discovery row with the pricing evidence used by free-route admission."""

    model_id: str
    provider_name: str
    prompt_price_per_1k: float | None
    completion_price_per_1k: float | None


@dataclass(frozen=True)
class FakeAgent:
    """Small agent shape accepted by ``dataclasses.replace`` in the bootstrap."""

    model: str
    priority: int = 0
    disabled: bool = True


class FakePriceBook:
    """Constructor seam; admission is decided from discovery pricing evidence."""

    def __init__(self, _store: object) -> None:
        pass


class HourlyOrchestratorFreeAdmissionTests(unittest.TestCase):
    """Keep paid and unpriced discovery rows out of the CI agent pool."""

    def _modules(self, discovered: list[FakeModel], selected_inputs: list[list[FakeModel]]) -> dict[str, ModuleType]:
        package = ModuleType("contextual_orchestrator")
        package.InMemoryConfigStore = object
        package.PriceBook = FakePriceBook

        discovery = ModuleType("contextual_orchestrator.model_discovery")
        discovery.discover_all_models = lambda: (discovered, [])
        discovery.refresh_price_book = lambda models, _book: sum(
            model.prompt_price_per_1k is not None or model.completion_price_per_1k is not None
            for model in models
        )

        def select(models: list[FakeModel], _book: object, limit: int) -> list[FakeModel]:
            selected_inputs.append(list(models))
            return list(models[:limit])

        discovery.select_top_n_cheapest_discovered_agents = select
        discovery.agent_from_discovered = lambda model, priority=0: FakeAgent(
            model=model.model_id,
            priority=priority,
        )
        return {
            "contextual_orchestrator": package,
            "contextual_orchestrator.model_discovery": discovery,
        }

    def test_paid_and_unpriced_routes_never_reach_cheapest_selector(self) -> None:
        """Filter before ranking so a cheap paid or unknown-price route cannot win."""

        free = FakeModel("free-chat", "openrouter", 0.0, 0.0)
        paid = FakeModel("paid-chat", "openrouter", 0.0001, 0.0002)
        unpriced = FakeModel("unknown-chat", "bytez", None, None)
        embedding = FakeModel("text-embedding-3-small", "openai", 0.0, 0.0)
        selected_inputs: list[list[FakeModel]] = []

        with patch.dict(
            sys.modules,
            self._modules([paid, unpriced, embedding, free], selected_inputs),
        ):
            agents, report = bootstrap._selected_agents()

        self.assertEqual([model.model_id for model in selected_inputs[0]], ["free-chat"])
        self.assertEqual([agent.model for agent in agents], ["free-chat"])
        self.assertEqual(report["chat_candidate_count"], 3)
        self.assertEqual(report["explicit_free_candidate_count"], 1)
        self.assertEqual(report["excluded_non_free_count"], 2)
        self.assertEqual(report["excluded_non_chat_count"], 1)

    def test_no_explicitly_free_chat_route_fails_closed(self) -> None:
        """Refuse the hourly LLM run instead of silently spending on a paid route."""

        selected_inputs: list[list[FakeModel]] = []
        discovered = [
            FakeModel("paid-chat", "openrouter", 0.0001, 0.0002),
            FakeModel("unknown-chat", "bytez", None, None),
        ]
        with patch.dict(sys.modules, self._modules(discovered, selected_inputs)):
            with self.assertRaisesRegex(RuntimeError, "no explicitly zero-cost"):
                bootstrap._selected_agents()

        self.assertEqual(selected_inputs, [])

    def test_explicit_zero_requires_both_price_components(self) -> None:
        """Treat partially priced and fully unpriced rows as non-free evidence."""

        self.assertTrue(bootstrap._is_explicitly_free_model(FakeModel("free", "p", 0.0, 0.0)))
        self.assertFalse(bootstrap._is_explicitly_free_model(FakeModel("prompt-paid", "p", 0.1, 0.0)))
        self.assertFalse(bootstrap._is_explicitly_free_model(FakeModel("completion-paid", "p", 0.0, 0.1)))
        self.assertFalse(bootstrap._is_explicitly_free_model(FakeModel("partial", "p", 0.0, None)))
        self.assertFalse(bootstrap._is_explicitly_free_model(FakeModel("unknown", "p", None, None)))


if __name__ == "__main__":
    unittest.main()
