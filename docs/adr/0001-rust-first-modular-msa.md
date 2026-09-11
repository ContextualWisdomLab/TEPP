# ADR 0001 — Rust-first numerical core and CPU `f64` reference

**Decision status:** Accepted
**Implementation maturity:** partial — workspace/quality foundation is implemented-main; checkpoint-versus-estimator authority is `checkpoint_authority` on the active PR; estimator/reference-backend implementations remain accepted-target
**Date:** 2026-08-05
**Supersession:** ADR 0011 governs cross-service ownership and standalone/MSA integration. This ADR remains authoritative for Rust-first numerical implementation and backend parity.

## Context

TEPP combines numerical psychometrics, temporal/event reasoning, multilingual evidence processing, GPU acceleration, services, and visual applications. Numerical behavior must be reproducible and auditable across backends, and components need stable boundaries that can be consumed independently.

The original wording also covered broad CWL service integration. That ownership is now made explicit in ADR 0011 so numerical architecture and service authority cannot drift together implicitly.

## Decision

Production mathematical and psychometric arithmetic is implemented in Rust. The workspace is divided into focused crates with stable versioned interfaces and no hidden repository-global state. Domain services depend on validated domain contracts rather than redefining numerical semantics.

Every estimator begins with a CPU `f64` reference implementation. Bounded multithreaded CPU and GPU implementations are optimizations and must demonstrate numerical parity, recovery, and failure equivalence against the reference. Python and R are limited to validation, independent oracles, interoperability, experimentation, and reporting; they do not become a second production owner of likelihoods, gradients, scoring, calibration, or other TEPP-owned numerical kernels.

`validation_core` production numerical execution requires a 64-bit pointer-width target. This is a numerical/resource contract rather than an x86-only contract: a 64-bit target may be admitted after its required CPU/backend, packaging, and release evidence passes. A 32-bit build may be used only for explicit portability characterization and is not a supported production target for `validation_core`. The crate fails at compile time on a non-64-bit pointer width so target-dependent `usize` arithmetic cannot silently broaden the supported scientific domain.

Service deployment, cross-repository persistence ownership, credential ownership, and direct database-coupling rules are not governed here; ADR 0011 is authoritative for those questions.

## Alternatives considered

1. **Python/R-first production arithmetic with Rust accelerators** — rejected because the production estimand would be owned in multiple languages and parity failures could be hidden behind wrappers.
2. **GPU-first implementation without a CPU oracle** — rejected because correctness and low-resource fallback would be difficult to audit.
3. **Rust-first domain/numerical core with CPU `f64` reference and optimized parity-checked backends** — accepted.
4. **Implicit support for every Rust pointer width** — rejected for `validation_core` because target-width-dependent counts and allocations would make resource admission and exact-integer proofs vary without an explicit production contract.

## Consequences

- Numerical correctness remains reviewable independently of GPU availability.
- Crates can be consumed as libraries or composed into services through ADR 0011 contracts.
- Backend-specific shortcuts cannot redefine the estimand.
- Duplicate Python/R production arithmetic is rejected.
- `validation_core` consumers and release artifacts must use 64-bit pointer-width targets; 32-bit portability experiments cannot be promoted as production evidence.
- Performance work includes fixed worker pools, oversubscription controls, sparse layouts, and measured context-switch/NUMA behavior.

## Failure and recovery

If an optimized CPU/GPU backend diverges from the reference, exceeds numerical tolerances, or cannot satisfy resource bounds, TEPP falls back to the validated CPU `f64` path or fails explicitly. It never changes model specification or precision silently merely to obtain success.

A non-64-bit `validation_core` target fails during compilation rather than executing with a narrower `usize` domain. Recovery selects a supported 64-bit build; changing that target contract requires an explicit ADR revision with cross-target numerical and resource evidence.

## Security, privacy, and governance impact

Numerical backends receive only authorized analysis inputs and cannot broaden provider, persistence, or service authority. Sensitive-data handling follows ADR 0009. Release and claim promotion follow ADR 0014.

## Compatibility and migration

Crate/public contract changes are versioned. A replacement numerical backend must reproduce the same estimand and pass recovery/parity evidence before promotion. A change to the estimand requires a new/superseding ADR and PRD change rather than being treated as a backend optimization.

The 64-bit `validation_core` target contract constrains production build admission but does not by itself choose an ISA or operating system. New 64-bit targets still require their normal build, test, numerical-parity, packaging, security, and recovery evidence.

## Verification

CI requires Rust formatting, warnings-denied linting, rustdoc, all tests, 100% production line/branch coverage and public documentation, true-parameter recovery, CPU/GPU parity where applicable, package/install smoke tests, dependency/security checks, SBOM, and provenance. Claim promotion additionally follows ADR 0014.

`validation_core` carries a compile-time non-64-bit refusal and a 64-bit contract test. The latter also fixes the exact integer bound used by current rational-scale reasoning: `floor(sqrt(u128::MAX)) == usize::MAX` on the supported pointer width. CI/release evidence from a different pointer width cannot satisfy this production contract.

## Rollback and supersession

Rollback selects the last validated Rust/reference implementation and compatible artifacts. Supersede only if a later decision provides an equally auditable single production arithmetic authority and reference-oracle strategy.
