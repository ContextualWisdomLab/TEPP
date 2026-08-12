# ADR 0006 — VRAM-adaptive GPU compute and model-credential boundary

**Decision status:** Accepted  
**Implementation maturity:** accepted-target  
**Date:** 2026-08-05  
**Supersession:** LLM orchestration-selection and test-time-compute policy is superseded by ADR 0010. Autonomous development/review/merge authority separation is governed by ADR 0015. This ADR remains authoritative for GPU/VRAM execution and the model-credential boundary.

## Context

TEPP must accelerate computationally material topic/psychometric workloads without redefining the estimand or excluding low-VRAM/CPU-only deployments. The earlier version of this ADR also mixed in LLM orchestration policy. That overlap is now removed so GPU resource authority, LLM reasoning policy, and repository automation authority cannot be confused.

## Decision

The CPU `f64` path remains the numerical reference under ADR 0001. GPU computation is streamed and backend-neutral, with CUDA as the primary NVIDIA performance path and WGPU/CubeCL or equivalent portable acceleration when separately validated. Full-corpus document-by-topic responsibilities are never retained in GPU memory.

A VRAM controller measures availability, reserves safety memory, predicts peak usage, autotunes micro-batches, uses approved mixed precision only for transient computation, releases temporary tensors, retries OOM with bounded batch reduction, falls back to CPU, and records allocation, transfer, retry, kernel, precision, and fallback telemetry. Final convergence/diagnostic quantities that require reference precision are evaluated against CPU `f64` evidence.

Local LLM and topic-model GPU phases do not coexist on small devices unless a validated resource profile proves that coexistence preserves the browser/UI/runtime reserve and numerical acceptance criteria.

Approved live model tests and autonomous development model calls use the GitHub secret `NVIDIA_NIM_API_KEY`. `COPILOT_GITHUB_TOKEN` is prohibited as a model/development credential. Existing independent review-agent credentials remain separate. How reasoning budget is allocated is defined by ADR 0010; how model jobs are separated from repository write/review/merge authority is defined by ADR 0015.

## Alternatives considered

1. **Full-corpus responsibility tensors on GPU** — rejected because memory scales with corpus-token/event count times topic count and becomes the dominant VRAM risk.
2. **GPU-only production path** — rejected because it removes an auditable reference and low-resource fallback.
3. **Streamed VRAM-budgeted GPU with CPU `f64` oracle/fallback** — accepted.

## Consequences

- GPU availability improves throughput without changing the estimand.
- 4/6/8/12/24-GiB profiles can select different micro-batches while preserving the same model contract.
- OOM is a tested operating state rather than an unhandled exception.
- provider/orchestration policy changes do not silently alter numerical backend semantics.

## Failure and recovery

OOM, device loss, non-finite output, parity failure, unsupported precision, or backend initialization failure triggers bounded cleanup/retry and then CPU fallback or explicit failure. TEPP never silently reduces model complexity, drops observations, changes a cutoff, or accepts a numerically non-equivalent result to fit memory.

## Security, privacy, and governance impact

GPU buffers, caches, model-provider payloads, and telemetry follow purpose-bound data policy under ADR 0009. Model secrets are isolated from ordinary CI and from unrelated repository/reviewer credentials. Device telemetry must not contain raw sensitive source text.

## Compatibility and migration

Compute backends expose one versioned contract. New CUDA/WGPU/CubeCL/runtime versions require parity and resource-profile evidence. An implementation may be disabled without changing serialized scientific artifacts or estimands.

## Verification

Real accelerator lanes cannot be skipped when hardware acceleration is claimed. Tests/studies cover CPU/GPU objective and parameter parity, deterministic artifacts under stated tolerances, 4/6/8/12/24-GiB profiles, peak VRAM/RSS/transfers, bounded batch adaptation, OOM/device-loss recovery, mixed-precision boundaries, and CPU fallback. Secret-policy tests prove prohibited credential co-residence and leakage paths are absent.

## Rollback and supersession

Rollback disables the affected accelerator/backend and uses the validated CPU reference. Supersede only if a later compute architecture preserves the same estimand, low-resource fallback, resource observability, credential separation, and parity evidence.
