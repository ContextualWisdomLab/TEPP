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
- Strict versioned JSON wire contracts for artifacts, documents, exact spans, and nested page locations without exposing private domain storage.
- ADR 0008 and APA 7 doctoring for evidence identity, hashing, JSON interchange, UTF-8 boundaries, Unicode segmentation limits, and future W3C PROV integration.
- Same-run exact missing-line, uncovered-region, zero-count-function, and missing-branch diagnostics for failed 100% Rust coverage gates.
- Sealed `EventTime`, `AssertionTime`, `DocumentTime`, `SystemTime`, `AvailableTime`, and `KnowledgeCutoff` nominal types over nanosecond-resolution absolute instants.
- Exact, bounded, lower-open, upper-open, and explicitly unknown `TemporalInterval<T>` values with included, excluded, and unbounded boundaries plus explicit precision and certainty.
- Strict temporal JSON wire version `1` and Draft 2020-12 JSON Schemas whose timestamp constraints match the runtime RFC 3339 known-offset profile.
- Task 3 architecture, ADR, and APA 7 doctoring for absolute timestamps, temporal uncertainty, wire interchange, and the boundary between typed intervals and future relation reasoning.
- All thirteen elementary Allen interval relations, exact inverse pairs, compact relation sets, stable elementary-order iteration, complete composition, and proper bounded-interval classification.
- Resource-bounded path-consistency closure with inverse propagation, direct-versus-derived status, conservative accepted-assertion provenance, contradiction evidence, instance-scoped identifiers, and atomic rollback.
- Task 4 ADR and APA 7 doctoring that delimit path consistency from global satisfiability, complete scenario search, and minimal contradiction cores.

### Security

- Prohibited `COPILOT_GITHUB_TOKEN` and reserved `NVIDIA_NIM_API_KEY` for approved LLM test and development workflows.
- Removed the bootstrap branch's credential-co-resident OpenCode workflow: no model process may receive repository-write authority, and scheduled product development remains disabled until proposal, independent verification, and late publication authority are separated across fresh jobs.
- Removed completed bootstrap materializers, encoded payload fragments, readiness sentinels, and push probes from the reviewable tree.
- Required full-commit GitHub Action pins, minimum permissions, concurrency controls, immutable audit evidence, SBOM, and provenance.
- Kept ordinary Rust CI free of LLM and reviewer credentials and disabled persisted checkout credentials.
- Refused to cache mutable Cargo registry, Git source, or target trees; cached quality binaries are keyed and checked by exact version.
- Copied caller-provided source bytes and document text before acceptance and kept validated evidence fields private.
- Made empty, oversized, malformed-digest, invalid UTF-8-boundary, coordinate-mismatch, cross-document, nonfinite-geometry, and out-of-page evidence fail closed with content-redacting errors.
- Rejected malformed or extended wire payloads, unsupported schema versions, invalid identifiers and byte values, digest/content substitution, stale document ownership, and invalid nested geometry during reconstruction.
- Rejected local timestamps without offsets, RFC 3339's semantically unknown `-00:00` marker, shortened or malformed offsets, leap seconds, bracketed zones, unsupported RFC 9557 suffixes, malformed calendars, cross-clock wire payloads, and semantically inconsistent temporal boundaries.
- Kept temporal errors stable and content-redacting so rejected timestamps and hostile wire values are not echoed into logs or downstream messages.
- Bounded qualitative reasoner variables, accepted constraints, and propagation work; rejected foreign identifiers and empty relation sets; and restored pre-closure state after contradiction or resource exhaustion.

### Quality

- Required 100% production line and branch coverage and complete public API docstrings.
- Required true-parameter recovery, RMSE, bias, interval coverage, temporal leakage, graph recovery, invariance, and CPU/GPU parity evidence.
- Added 100% statement and branch coverage for the repository quality-gate scripts.
- Made a zero executable-code coverage denominator explicit for the skeleton-only slice rather than treating it as evidence of implemented behavior.
- Denied warnings, missing public documentation, and unsafe Rust across the workspace.
- Added known digest vectors, mutation detection, hostile multibyte Unicode, exact-coordinate, page-boundary, stable-error, and invalid-input regression tests for the first evidence slice.
- Added strict wire round trips, unknown-field and version rejection, digest reconstruction, configured-limit, hostile JSON, and generated multilingual span tests.
- Added six-clock parity, nanosecond ordering, known-offset and daylight-saving normalization, unknown-offset rejection, exact and uncertain interval, hostile temporal wire, schema/runtime parity, and stable-error regression tests.
- Split Task 3 claims from deferred Allen relation algebra, closure, event graphs, persistence, leakage snapshots, and synthetic-truth recovery so documentation cannot overstate executable evidence.
- Added independent exhaustive composition-oracle, converse-law, all-relation classification, stable relation-set iteration, closure-idempotence, contradiction, provenance, identifier-isolation, limit, and atomic-rollback tests for Task 4.

[Unreleased]: https://github.com/ContextualWisdomLab/TEPP/compare/HEAD...HEAD
