# Rust workspace and quality-tooling register

This engineering doctoring note records the first-party sources and exact
versions used by Temporal/Event Foundation Task 1. It supplements
`standards-and-literature.md`; it does not replace the scientific references
required by later estimators.

## Rust compiler

TEPP pins Rust 1.97.1 for the stable build reference. The point release repairs
an LLVM optimization miscompilation and therefore supersedes 1.97.0 for this
foundation. A future compiler update requires exact-head formatting, Clippy,
rustdoc, test, coverage, and numerical-parity evidence before adoption.

## Cargo workspace

The workspace uses an explicit member list and workspace-inherited package
metadata and lints. Cargo's official workspace reference defines `members`,
`default-members`, and `[workspace.lints]`; TEPP does not use a wildcard member
glob because accidental crate admission would silently expand the trusted build
surface.

## Test and coverage tooling

- `cargo-nextest` 0.9.140 runs process-isolated tests without retries.
- Doctests run separately because nextest does not currently execute doctests.
- `cargo-llvm-cov` 0.8.6 produces stable line coverage.
- Branch coverage uses the same tool on `nightly-2026-08-01` because the
  upstream project identifies Rust branch coverage as unstable and
  nightly-only.
- Coverage thresholds are evaluated from LLVM JSON totals. A nonzero line or
  branch denominator passes only when all units are covered.
- Coverage.py 7.15.2 measures the repository-quality Python scripts at 100%
  statement and branch coverage.

## Dependency policy

`cargo-deny` 0.19.7 checks advisories, yanked packages, licenses, duplicate or
wildcard dependencies, and unapproved registries or Git sources. This check is
a policy gate rather than legal advice; procurement and release review still
own final license acceptance.

## Security boundary

Ordinary Rust CI has read-only repository permission, does not persist checkout
credentials, and receives no LLM, reviewer, publisher, or deployment secret.
The dedicated `NVIDIA_NIM_API_KEY` remains reserved for reviewed LLM workflows
and is not needed for deterministic foundation validation.

## APA 7th references

Batchelder, N., & contributors. (2026). *Coverage.py* (Version 7.15.2)
[Computer software]. https://coverage.readthedocs.io/

Embark Studios. (2026). *cargo-deny* (Version 0.19.7) [Computer software].
GitHub. https://github.com/EmbarkStudios/cargo-deny

Endo, T. (2026). *cargo-llvm-cov* (Version 0.8.6) [Computer software]. GitHub.
https://github.com/taiki-e/cargo-llvm-cov

Nextest contributors. (2026). *cargo-nextest* (Version 0.9.140)
[Computer software]. https://nexte.st/

The Cargo Team. (n.d.). *Workspaces*. In *The Cargo Book*. Retrieved August 5,
2026, from https://doc.rust-lang.org/cargo/reference/workspaces.html

The Rust Release Team. (2026, July 16). *Announcing Rust 1.97.1*. Rust Blog.
https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/
