# ADR 0025 — macOS-native Rust-owned MLX Metal execution boundary

**Decision status:** Accepted  
**Implementation maturity:** accepted-target — contract active, native service pending  
**Date:** 2026-08-26  
**Supersedes:** None; specializes ADR 0001 and ADR 0006 for Apple Silicon.

## Context

MLX Metal requires the macOS host and cannot execute inside Colima's Linux VM.
Putting numerical logic in a Python sidecar would violate TEPP's Rust ownership,
while claiming Metal from a Linux container would falsify execution provenance.

## Decision

Apple Silicon acceleration runs in a macOS-native, Rust-owned computation
service that invokes MLX Metal through a memory-safe FFI or isolated native
adapter. Compose clients use an authenticated local Unix socket when available,
or an authenticated host-gateway transport bound to the local machine. Python
may package or transport data but never owns, reimplements, or repairs the
mathematics.

Every result persists the actual backend, host/accelerator class, objective and
parameter/draw digests, execution identity, and method-derived numerical parity
evidence against the Rust CPU f64 reference. `mlx_metal_macos_native` is valid
only when native Metal execution occurred. Linux and container CI may record
only `rust_cpu`, `mlx_cpu`, `mlx_cuda`, or `rust_opencl` when that backend
actually executes. A macOS-native MLX CPU execution records
`mlx_cpu_macos_native`; it is never relabeled as Metal. `mlx_opencl` is not a
backend. Missing authentication, backend receipt, or
parity evidence fails closed.

## Cross-repository contract

TEPP produces backend-bearing posterior artifacts. fast-mlsirm consumes them
without changing the backend label or parity bound. LineageWeave persists and
authorizes the artifact but neither recomputes nor exposes implementation names
to customer UI. Compose configuration never mounts the Metal device into
Colima; it connects to the authenticated host service.

## Alternatives considered

1. Run MLX Metal inside Colima — rejected as technically unsupported.
2. Python-owned MLX arithmetic — rejected because production math belongs to
   Rust and would create a second estimator.
3. Label container CPU results as Metal-equivalent — rejected as false
   provenance.
4. Native Rust-owned MLX service plus authenticated Compose boundary — accepted.

## Consequences

Native acceleration remains usable while containers retain reproducible
portability. Deployment gains a host service and authentication lifecycle.
Backend claims become auditable rather than inferred from machine type.

## Security and privacy

The socket/gateway uses mutually authenticated, short-lived local credentials,
strict payload bounds, replay identity, peer verification, and content-redacted
errors. It binds only to local transport and receives opaque analytical ids;
source content is not logged. A container cannot select or forge the receipt.

## Operability and failure recovery

Health/readiness distinguish service availability from Metal readiness. OOM,
driver loss, host sleep, timeout, authentication failure, and parity failure are
terminal for that accelerated attempt. A caller may explicitly replay on the
Rust CPU f64 portability path and records a new receipt; it never relabels the
fallback as Metal.

## Verification, testing and acceptance

- the native MLX CPU probe executes a known objective, compares its output with
  the Rust CPU reference, and emits `mlx_cpu_macos_native` only on macOS;
- macOS hardware E2E proves native MLX Metal execution and receipt binding;
- container E2E proves authenticated host-gateway/Unix-socket access and that
  Linux cannot emit a Metal receipt;
- true-parameter recovery and method-derived CPU/MLX numerical parity pass;
- authentication, replay, payload, timeout, OOM, fallback, and forged-receipt
  tests cover all branches.

## Rollback and supersession

Disable the native service and use an explicitly receipted Rust CPU f64 run.
Preserve issued artifacts and never rewrite backend provenance. A different
accelerator, transport, or numerical authority requires a superseding ADR and
coordinated TEPP/fast-mlsirm contract update.
