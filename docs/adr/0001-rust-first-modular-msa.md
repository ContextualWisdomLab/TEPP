# ADR 0001: Rust-First Modular MSA and CPU f64 Reference

**Status:** Accepted  
**Date:** 2026-08-05

## Context

TEPP combines numerical psychometrics, temporal/event reasoning, multilingual evidence processing, GPU acceleration, services, and visual applications. Components must work independently and when imported by CWL repositories, `naruon`, or `contextual-orchestrator`. Numerical behavior must be reproducible and auditable across backends.

## Decision

Production mathematical and psychometric arithmetic is implemented in Rust. The workspace is divided into focused crates with stable versioned interfaces and no hidden repository-global state. Services depend on domain contracts rather than the reverse.

Every estimator begins with a CPU `f64` reference implementation. Bounded multithreaded CPU and GPU implementations are optimizations and must demonstrate numerical parity, recovery, and failure equivalence against the reference. Python and R are limited to validation, independent oracles, interoperability, and reporting.

## Consequences

- Numerical correctness remains reviewable independently of GPU availability.
- Crates can be consumed as libraries or deployed as services.
- Backend-specific shortcuts cannot redefine the estimand.
- Duplicate Python production arithmetic is rejected.
- Performance work includes fixed worker pools, oversubscription controls, sparse layouts, and measured context-switch/NUMA behavior.

## Verification

CI requires Rust formatting, warnings-denied linting, rustdoc, all tests, 100% production line/branch coverage and public docstrings, true-parameter recovery, CPU/GPU parity where applicable, package/install smoke tests, dependency/security checks, SBOM, and provenance.
