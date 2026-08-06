# ADR 0006: VRAM-Adaptive GPU Compute and NVIDIA Orchestration

**Status:** Accepted  
**Date:** 2026-08-05

## Decision

The CPU `f64` path is the numerical reference. GPU computation is streamed and backend-neutral, with CUDA as the primary performance path and WGPU/CubeCL or equivalent portable acceleration when validated. Full-corpus document-by-topic responsibilities are never retained in GPU memory.

A VRAM controller measures availability, reserves safety memory, predicts peak usage, autotunes micro-batches, uses stable mixed precision, releases temporary tensors, retries OOM with bounded batch reduction, falls back to CPU, and records allocation, transfer, retry, kernel, and fallback telemetry. Local LLM and topic-model GPU phases do not coexist on small devices.

Approved LLM tests and autonomous development use the GitHub secret `NVIDIA_NIM_API_KEY`. `COPILOT_GITHUB_TOKEN` is prohibited. Existing review-agent credentials remain unchanged. `contextual-orchestrator` must compare direct routing and deeper role-based workflows, including reasoning-effort, decomposition, recursion, workflow-stage, and access-list ablations.

## Consequences

GPU availability improves throughput without changing the estimand or making low-VRAM devices unsupported. OOM is a tested operating condition. Orchestration quality and cost are measured rather than assumed from model size or agent count.

## Verification

Real GPU tests cannot be skipped. CI or scheduled studies cover CPU/GPU parity, 4/6/8/12/24-GB profiles, peak memory, bounded fallback, direct-versus-orchestrated quality, calibration, disagreement, token/cost telemetry, and secret/prompt-injection boundaries.
