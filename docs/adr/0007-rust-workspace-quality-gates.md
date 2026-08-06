# ADR 0007: Explicit Rust workspace and exact quality gates

- **Status:** Accepted
- **Date:** 2026-08-05
- **Decision owners:** Contextual Wisdom Lab
- **Supersedes:** None

## Context

Every later TEPP estimator depends on stable crate boundaries, deterministic
tooling, complete documentation, and exact validation evidence. An implicit
`crates/*` workspace can silently absorb experimental packages. Warning-only
lints permit quality regressions. Stable Rust line coverage is available, but
Rust branch coverage remains an unstable LLVM/compiler capability and therefore
cannot be honestly represented as a stable-only gate.

The initial planning repository contained no Rust workspace. Task 1 must create
the build and validation substrate without inventing placeholder APIs or
claiming that the temporal, event, database, GPU, or psychometric layers exist.

## Decision

1. Use a virtual Cargo workspace with an explicit ordered member list.
2. Pin the stable compiler to Rust 1.97.1, including the LLVM miscompilation
   correction published by the Rust Release Team.
3. Centralize package metadata and lints. Every member inherits:
   - `unsafe_code = "forbid"`;
   - `missing_docs = "deny"`;
   - warning denial; and
   - strict Clippy `all`, `pedantic`, and `cargo` groups with documented,
     minimal exceptions.
4. Create ten focused crates corresponding to the approved Temporal/Event
   Foundation plan. Skeleton crates contain module-level rustdoc and no public
   placeholder behavior.
5. Run `cargo-nextest` 0.9.140 without retries and run doctests separately.
6. Enforce stable line coverage with `cargo-llvm-cov` 0.8.6.
7. Enforce branch coverage with the same tool on pinned
   `nightly-2026-08-01`; parse LLVM JSON and require `covered == count`.
8. Report a zero denominator explicitly. It is valid only while the slice has
   no executable production units and must not be used as evidence that a
   domain implementation exists.
9. Enforce advisories, licenses, dependency bans, and source origins with
   `cargo-deny` 0.19.7.
10. Test repository-quality Python scripts at 100% statement and branch
    coverage with pinned Coverage.py 7.15.2.
11. Require all GitHub Actions and reusable workflows to use a full commit SHA.
12. Cache only pinned Cargo quality-tool executables. Cache keys include the OS,
    architecture, and exact tool versions; cached binaries are version-checked
    before use. Mutable Cargo registry, Git source, and build-output trees are
    deliberately excluded from the cache boundary.
13. Do not expose `NVIDIA_NIM_API_KEY`, reviewer credentials, publication
    credentials, or any LLM secret to ordinary Rust CI.

## Consequences

- Crate boundaries and package identities are reviewable before domain logic is
  introduced.
- A new crate cannot enter the workspace accidentally.
- Branch coverage is reproducible but depends on a separate nightly lane; stable
  compiler behavior remains the numerical and build reference.
- Cold CI still compiles pinned quality tools once, while later commits in the
  same protected PR lineage restore only verified binaries. Dependency source
  trees and build outputs remain fresh and reviewable.
- The foundation PR cannot claim scientific correctness, database readiness,
  GPU parity, or release readiness. Those require their later plan tasks and
  exact-head evidence.

## Validation

- Repository contract tests cover valid and hostile workspace states.
- Public-rustdoc tests cover crate-level and item-level documentation.
- LLVM coverage JSON validation rejects malformed, impossible, incomplete, or
  missing line/branch totals.
- CI runs formatting, compile, Clippy, nextest, doctest, rustdoc,
  cargo-deny, stable line coverage, and nightly branch coverage.
- The CI contract rejects unpinned Actions, forbidden credentials, and mutable
  Cargo registry/Git cache paths.

## References

The Cargo Team. (n.d.). *Workspaces*. In *The Cargo Book*. Retrieved August 5,
2026, from https://doc.rust-lang.org/cargo/reference/workspaces.html

GitHub. (2026). *Cache* (Version 5.0.5) [GitHub Action].
https://github.com/actions/cache

The Rust Release Team. (2026, July 16). *Announcing Rust 1.97.1*. Rust Blog.
https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/

Batchelder, N., & contributors. (2026). *Coverage.py* (Version 7.15.2)
[Computer software]. https://coverage.readthedocs.io/

Embark Studios. (2026). *cargo-deny* (Version 0.19.7) [Computer software].
GitHub. https://github.com/EmbarkStudios/cargo-deny

Endo, T. (2026). *cargo-llvm-cov* (Version 0.8.6) [Computer software]. GitHub.
https://github.com/taiki-e/cargo-llvm-cov

Nextest contributors. (2026). *cargo-nextest* (Version 0.9.140)
[Computer software]. https://nexte.st/
