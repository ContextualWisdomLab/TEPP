"""Bootstrap an ephemeral contextual-orchestrator gateway for CI agents.

Provider credentials arrive only at bootstrap, are registered in the
orchestrator KV, and are removed from this process environment before the
gateway begins serving requests. Model discovery therefore exercises every
configured provider while OpenCode receives only the loopback bearer token.
"""

from __future__ import annotations

import argparse
from dataclasses import replace
import json
import os
from pathlib import Path
import re
from typing import Any

PROVIDER_CREDENTIAL_NAMES = (
    "BYTEZ_API_KEY",
    "NVIDIA_NIM_API_KEY",
    "NVIDIA_NIM_API_KEY_SUB",
    "OPENROUTER_API_KEY",
    "OPENAI_API_KEY",
)

_MODEL_TOKEN_RE = re.compile(r"[a-z0-9]+")
_NON_CHAT_MODEL_TOKENS = frozenset(
    {
        "bge",
        "clip",
        "dall",
        "e5",
        "embed",
        "embedding",
        "embeddings",
        "image",
        "images",
        "moderation",
        "realtime",
        "rerank",
        "reranker",
        "siglip",
        "sora",
        "speech",
        "transcribe",
        "transcription",
        "tts",
        "whisper",
    }
)
_NON_CHAT_MODEL_PREFIXES = ("embed", "moderat", "rerank", "transcrib")


def _is_general_chat_model(model_id: object) -> bool:
    """Keep endpoint-only and safety-only catalog rows out of chat routing."""
    if not isinstance(model_id, str):
        return False
    tokens = tuple(_MODEL_TOKEN_RE.findall(model_id.casefold()))
    if not tokens or any(
        token in _NON_CHAT_MODEL_TOKENS
        or token.startswith(_NON_CHAT_MODEL_PREFIXES)
        for token in tokens
    ):
        return False
    return not any(
        token == "safety"
        or token == "guard"
        or token == "shieldgemma"
        or token.startswith("nemoguard")
        for token in tokens
    )


def _is_explicitly_free_model(model: object) -> bool:
    """Admit only discovered routes whose two token prices are explicitly zero."""

    sentinel = object()
    prompt_price = getattr(model, "prompt_price_per_1k", sentinel)
    completion_price = getattr(model, "completion_price_per_1k", sentinel)
    return prompt_price == 0.0 and completion_price == 0.0


def _register_credential(name: str, value: str) -> None:
    """Register one bootstrap secret through contextual-orchestrator's KV seam."""

    from contextual_orchestrator import register_credential

    register_credential(name, value)


def _register_bootstrap_credentials() -> None:
    """Move all provider keys from bootstrap environment into the KV registry."""

    missing: list[str] = []
    for name in PROVIDER_CREDENTIAL_NAMES:
        value = os.environ.pop(name, "")
        if value:
            _register_credential(name, value)
        else:
            missing.append(name)
    if missing:
        raise RuntimeError(f"missing provider credentials: {', '.join(missing)}")


def _selected_agents() -> tuple[list[Any], dict[str, object]]:
    """Discover providers and enable at most three explicitly zero-cost routes."""

    from contextual_orchestrator import InMemoryConfigStore, PriceBook
    from contextual_orchestrator.model_discovery import (
        agent_from_discovered,
        discover_all_models,
        refresh_price_book,
        select_top_n_cheapest_discovered_agents,
    )

    discovered, errors = discover_all_models()
    if not discovered:
        providers = ", ".join(sorted(error.provider_name for error in errors)) or "none"
        raise RuntimeError(f"model discovery produced no candidates; providers_with_errors={providers}")
    price_book = PriceBook(InMemoryConfigStore())
    chat_discovered = [
        model for model in discovered if _is_general_chat_model(model.model_id)
    ]
    if not chat_discovered:
        raise RuntimeError("model discovery produced no general chat candidates")
    priced_count = refresh_price_book(chat_discovered, price_book)
    free_discovered = [model for model in chat_discovered if _is_explicitly_free_model(model)]
    if not free_discovered:
        raise RuntimeError(
            "model discovery produced no explicitly zero-cost general chat candidates"
        )
    selected = select_top_n_cheapest_discovered_agents(free_discovered, price_book, 3)
    if not selected:
        raise RuntimeError("model discovery selected no explicitly zero-cost general chat candidates")
    agents = [
        replace(agent_from_discovered(model, priority=3 - index), disabled=False)
        for index, model in enumerate(selected)
    ]
    report = {
        "discovered_count": len(discovered),
        "chat_candidate_count": len(chat_discovered),
        "explicit_free_candidate_count": len(free_discovered),
        "excluded_non_free_count": len(chat_discovered) - len(free_discovered),
        "excluded_non_chat_count": len(discovered) - len(chat_discovered),
        "priced_count": priced_count,
        "providers_discovered": sorted({model.provider_name for model in discovered}),
        "providers_with_errors": sorted({error.provider_name for error in errors}),
        "selected_models": [agent.model for agent in agents],
    }
    return agents, report


def _write_report(path: Path, report: dict[str, object]) -> None:
    """Write secret-free discovery evidence with restrictive permissions."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, ensure_ascii=True, sort_keys=True) + "\n", encoding="utf-8")
    path.chmod(0o600)


def _start_gateway(agents: list[Any], gateway_token: str, host: str, port: int) -> None:
    """Construct and serve the contextual-orchestrator HTTP gateway."""

    from contextual_orchestrator import TaskOrchestrator
    from contextual_orchestrator.server import SecurityConfig, serve

    serve(
        TaskOrchestrator(agents),
        host=host,
        port=port,
        security=SecurityConfig(auth_token=gateway_token),
    )


def main() -> None:
    """Start the authenticated gateway after provider discovery succeeds."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=18000)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()

    _register_bootstrap_credentials()
    gateway_token = os.environ.pop("CONTEXTUAL_ORCHESTRATOR_INFERENCE_TOKEN", "")
    if not gateway_token:
        raise RuntimeError("CONTEXTUAL_ORCHESTRATOR_INFERENCE_TOKEN is required")
    agents, report = _selected_agents()
    _write_report(args.report, report)
    _start_gateway(agents, gateway_token, args.host, args.port)


if __name__ == "__main__":  # pragma: no cover - exercised by the workflow process
    main()
