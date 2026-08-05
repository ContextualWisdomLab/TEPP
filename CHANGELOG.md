# Changelog

All notable changes to TEPP are documented here. The format follows Keep a Changelog and semantic versioning once the first implementation release is cut.

## [Unreleased]

### Added

- Approved Temporal Event Psychometrics Platform PRD v0.4 baseline.
- Eight-phase delivery roadmap and Temporal/Event Foundation implementation plan.
- Immutable evidence, six-clock temporal semantics, interval reasoning, event ontology, typed relation graph, and time-varying multiple-membership contracts.
- Shared-latent multilingual topic measurement architecture with native lexical channels and language-profile validation.
- Longitudinal ESEM/DSEM and continuous-time structural modeling requirements.
- Rust-first CPU `f64`, multithreaded CPU, GPU, VRAM-adaptive streaming, and CPU/GPU parity requirements.
- Topic correlation, consensus clustering, TDT, CHRONOS, and evidence-grounded LLM interpretation requirements.
- APA 7th research traceability, source archive manifests, ADRs, governance, security, and contribution contracts.
- Hourly centralized PR-maintenance workflow and a documented requirement for a future credential-separated NVIDIA NIM/OpenCode product-development loop.
- Rust 1.97.1 virtual Cargo workspace with ten explicit modular foundation crates.
- Repository contract, public-rustdoc, line-coverage, and nightly branch-coverage gates.
- Pinned `cargo-nextest` 0.9.140, `cargo-llvm-cov` 0.8.6, `cargo-deny` 0.19.7, and Coverage.py 7.15.2 quality tooling.
- Task 1 architecture decision and workspace-foundation validation report.
- Version-keyed, executable-only GitHub Actions caches for pinned Rust quality tools.
- Immutable `evidence_core` records with independent RFC 9562 `UUIDv7` identities, canonical `SHA-256` content digests, owned source bytes and UTF-8 text, exact byte/Unicode-scalar spans, and bounded page-layout coordinates.
- ADR 0008 and APA 7 doctoring for evidence identity, hashing, UTF-8 boundaries, Unicode segmentation limits, and future W3C PROV integration.
- Same-run exact missing-line and missing-branch diagnostics for failed 100% Rust coverage gates.

### Security

- Prohibited `COPILOT_GITHUB_TOKEN` and reserved `NVIDIA_NIM_API_KEY` for approved LLM test and development workflows.
- Removed the bootstrap branch's credential-co-resident OpenCode workflow: no model process may receive repository-write authority, and scheduled product development remains disabled until proposal, independent verification, and late publication authority are separated across fresh jobs.
- Removed completed bootstrap materializers, encoded payload fragments, readiness sentinels, and push probes from the reviewable tree.
- Required full-commit GitHub Action pins, minimum permissions, concurrency controls, immutable audit evidence, SBOM, and provenance.
- Kept ordinary Rust CI free of LLM and reviewer credentials and disabled persisted checkout credentials.
- Refused to cache mutable Cargo registry, Git source, or target trees; cached quality binaries are keyed and checked by exact version.
- Copied caller-provided source bytes and document text before acceptance and kept validated evidence fields private.
- Made empty, oversized, malformed-digest, invalid UTF-8-boundary, coordinate-mismatch, cross-document, nonfinite-geometry, and out-of-page evidence fail closed with content-redacting errors.

### Quality

- Required 100% production line and branch coverage and complete public API docstrings.
- Required true-parameter recovery, RMSE, bias, interval coverage, temporal leakage, graph recovery, invariance, and CPU/GPU parity evidence.
- Added 100% statement and branch coverage for the repository quality-gate scripts.
- Made a zero executable-code coverage denominator explicit for the skeleton-only slice rather than treating it as evidence of implemented behavior.
- Denied warnings, missing public documentation, and unsafe Rust across the workspace.
- Added known digest vectors, mutation detection, hostile multibyte Unicode, exact-coordinate, page-boundary, stable-error, and invalid-input regression tests for the first evidence slice.

[Unreleased]: https://github.com/ContextualWisdomLab/TEPP/compare/HEAD...HEAD
