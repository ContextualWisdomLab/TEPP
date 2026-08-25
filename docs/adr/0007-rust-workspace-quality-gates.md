# ADR 0007 — Explicit Rust workspace and exact quality gates

**Decision status:** Accepted  
**Implementation maturity:** implemented-main  
**Date:** 2026-08-05  
**Decision owners:** Contextual Wisdom Lab  
**Supersedes:** None. ADR 0014 governs scientific/product claim promotion and release authority beyond repository-quality tooling.

## Context

Every later TEPP estimator depends on stable crate boundaries, deterministic tooling, complete documentation, and exact validation evidence. An implicit `crates/*` workspace can silently absorb experimental packages. Warning-only lints permit quality regressions. Stable Rust line coverage is available, but Rust branch coverage remains an LLVM/compiler capability that requires an explicitly pinned lane and must not be represented as a vague best-effort gate.

The initial planning repository contained no Rust workspace. Task 1 therefore created the build and validation substrate without inventing placeholder APIs or claiming that temporal, event, database, GPU, or psychometric layers already existed.

## Decision

1. Use a virtual Cargo workspace with an explicit ordered member list.
2. Pin the stable compiler to the approved Rust toolchain version and update it only through reviewed compatibility evidence.
3. Centralize package metadata and lints. Every member inherits `unsafe_code = "forbid"`, `missing_docs = "deny"`, warning denial, and strict Clippy groups with documented minimal exceptions.
4. Use focused crates corresponding to approved bounded contexts. Skeleton crates contain module-level rustdoc and no public placeholder behavior.
5. Run `cargo-nextest` without hidden retries and doctests separately.
6. Enforce stable line coverage with pinned `cargo-llvm-cov`.
7. Enforce branch coverage in the pinned LLVM/nightly lane and require exact covered/count equality.
8. Treat a zero executable denominator explicitly as absence of implemented production behavior, never as 100% domain coverage.
9. Enforce advisories, licenses, dependency bans, and source origins with pinned `cargo-deny`.
10. Test repository-quality Python scripts at exact statement/branch coverage under pinned Coverage.py.
11. Require GitHub Actions/reusable workflows to use immutable full commit SHA references.
12. Cache only verified pinned quality-tool executables; mutable dependency source trees and build-output trees are outside the trusted cache boundary.
13. Do not expose `NVIDIA_NIM_API_KEY`, reviewer credentials, publication credentials, or any LLM secret to ordinary Rust CI.

## Alternatives considered

1. **Implicit wildcard workspace and warning-only linting** — rejected because experiments can become production members silently and regressions remain advisory.
2. **Coverage best-effort thresholds below 100% for owned production logic** — rejected because the repository contract explicitly requires exact owned-code coverage and docstrings.
3. **Explicit workspace, pinned tooling, fail-closed exact gates** — accepted.

## Consequences

- Crate boundaries and package identities are reviewable before domain logic is introduced.
- A new crate cannot enter the workspace accidentally.
- Branch coverage is reproducible only under the governed pinned lane; stable compiler behavior remains the build/numerical reference.
- Cached tools are validated by exact version while dependency/source/build trees remain fresh.
- Green quality tooling does not by itself claim scientific, database, GPU, operational, or release readiness; ADR 0014 governs those promotions.

## Failure and recovery

Malformed coverage evidence, zero-production-unit ambiguity, unpinned actions/tools, advisory/license/source-policy failure, secret leakage, or warning/docstring failure blocks the gate. Recovery fixes the underlying source/tooling contract and reruns the exact gate; the check is never converted to success by excluding relevant production behavior.

## Security, privacy, and governance impact

Ordinary CI remains credential-minimal. Supply-chain and workflow provenance are part of release evidence. Autonomous model/repository authority is governed separately by ADR 0015.

## Compatibility and migration

Toolchain/version upgrades require lock/update evidence, compiler/package compatibility tests, and any necessary source migration. A tool upgrade cannot silently weaken coverage, lint, supply-chain, or credential boundaries.

## Verification

Repository contract tests cover valid and hostile workspace states. Public-rustdoc tests cover crate/item documentation. LLVM coverage JSON validation rejects malformed, impossible, incomplete, missing, or vacuous totals. CI runs formatting, compile, Clippy, nextest, doctest, rustdoc, cargo-deny, line coverage, branch coverage, and repository documentation/security contracts.

## Rollback and supersession

Rollback restores the previous pinned toolchain/workspace policy and reruns the full quality suite. Supersede only with an ADR that preserves explicit membership, non-vacuous exact validation, complete public documentation, supply-chain integrity, and credential separation.

## References

The Cargo Team. (n.d.). *Workspaces*. In *The Cargo Book*. Retrieved August 5, 2026, from https://doc.rust-lang.org/cargo/reference/workspaces.html

The Rust Release Team. (2026, August 20). *Announcing Rust 1.98.0*. Rust Blog. https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/

Batchelder, N., & contributors. (2026). *Coverage.py* [Computer software]. https://coverage.readthedocs.io/

Embark Studios. (2026). *cargo-deny* [Computer software]. GitHub. https://github.com/EmbarkStudios/cargo-deny

Endo, T. (2026). *cargo-llvm-cov* [Computer software]. GitHub. https://github.com/taiki-e/cargo-llvm-cov

Nextest contributors. (2026). *cargo-nextest* [Computer software]. https://nexte.st/
