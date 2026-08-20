# Changelog

All notable changes to TEPP are documented here. The format follows Keep a Changelog and semantic versioning once the first implementation release is cut.

## [Unreleased]

### Added

- `persistence_postgres` `audit_event` inserts call `operational_log::try_record` before SQL is rendered: author/customer/project source text, source identity, and blanket-mask grants cannot enter `INSERT INTO audit_event`; clear inspection still persists a validated action code (ADR 0009; ISO/IEC 29100:2024). No new migration number. `OperationalLogRecord::new` stays crate-private.
- `operational_log` source separation: `try_record` is the only recording API and inspects source text, source identity, and blanket-mask intent before creating a line; `OperationalLogRecord::new` is crate-private; a source-identity `&str` cannot become an `AnalyticalSubject`; privileged-export / identity-mapping / diagnosis action codes keep author, customer, and project memberships distinct; replayed lines match known truth at a higher computed rate than a collapsed single-action or collapsed-subject log (ADR 0009; ISO/IEC 29100:2024). The live docstring crate-root count is bound to `EXPECTED_CRATES` so the eleventh crate cannot fail a hard-coded `10`.
- `tepp_api` adaptive orchestration router (ADR 0010): versioned `direct`/`verify`/`committee`/`conductor`/`abstain` selection from CPU `f64` risk, ambiguity, evidence, and token-budget inputs; recorded stages, recursion, decomposition, access lists, and role-specific reasoning effort; fail-closed document-controlled policy/access/credentials; LLM plans remain proposals under deterministic statistical authority; comparable-budget ablation requires a direct baseline; credential-free contextual-orchestrator binding. Live NIM HTTP remains accepted-target.
- `tepp_api` purpose-bound provider-payload minimization: time-bounded `PurposeGrant` evaluation, fail-closed expired/not-yet-valid/inverted/cross-tenant/impossible-calendar denial, semantic UTC calendar validation, refusal to copy identity mappings into model-provider payloads or ordinary logs, preservation of opaque analytical identifiers and membership roles (no blanket PII mask), a separately authorized scientific re-identification path, and an internally bound FIPS 180-4 SHA-256 audit digest appended through `ReidentificationAuditSink` before disclosure.
- `persistence_postgres` backup/restore integrity: restored snapshots stay unusable until tenant, canonical `SHA-256`, knowledge-cutoff eligibility, temporal window order, and append-only triggers revalidate; SQL probes raise `restore integrity failed` (ADR 0013).
- `persistence_postgres` concurrent document-write stress: atomic revise `DO` block that requires exactly one open `system_to` close, SQLSTATE mapping onto `ConcurrentWriteConflict` / `DuplicateDocumentRecord`, and live multi-session insert/revise/append-only proofs. No new migration number.
- `tepp_api` naruon HTTP interchange: versioned `https` POST contracts for analysis-run create and modular export authorization that refuse table-access URLs, review/Copilot credential headers, reserved standard-header redefinition, principal-only export idempotency keys, and lexical inference claims (ADR 0011).
- `persistence_postgres` audit-event SQL contracts: append-only insert that refuses empty, oversized, or hostile `action_code` values before SQL is rendered.
- `persistence_postgres` event-instance SQL contracts: bitemporal insert and as-known-at lookup that refuse inverted valid/system windows and hostile type/lifecycle labels before SQL is rendered.
- `persistence_postgres` event-mention SQL contracts: mention identity cannot equal the instance it supports; confidence must be finite and in `(0, 1]`.
- `persistence_postgres` event-relation SQL contracts: closed ERD transition/provenance vocabulary bound to `transition_edge`, fail-closed unknown types and transition self-loops, live insert of `causes`/`references`.
- `persistence_postgres` source-artifact SQL contracts: append-only insert and primary-key lookup that refuse non-canonical `SHA-256` digests, negative sizes, and hostile media-type or object-store labels before SQL is rendered; identical-identity retries are `ON CONFLICT DO NOTHING` plus a stored-row match assertion, and a same-id payload change fails closed as `ConflictingSourceArtifact`.
- `persistence_postgres` typed membership assignment (migration `0006`): `entity_record`, `project_record`, and `text_segment` plus exactly-one observed-unit and target constraints that replace the polymorphic `membership_target_id` stub, with SQL insert/lookup, fail-closed inverted-window and backslash-label refusal, and live proof that one document persists two entity memberships and one project membership.
- Actions workflow fleet auditor (`scripts/actions_workflow_fleet.py`): paginated registry inventory bound to the exact default-branch SHA/tree, classification of present/orphan/disabled/GitHub-dynamic identities, and fail-closed orphan disable that confirms GitHub's official `disabled_manually` state.
- `persistence_postgres` temporal interval ordering migration (`0005`): multi-word CHECK constraints on `document_record`, `event_instance`, and `membership_assignment` that reject inverted valid/system windows and non-positive document revisions while preserving open-ended NULL upper bounds and equal point bounds; catalog validation and live inverted-window proof.
- `persistence_postgres` append-only immutability migration (`0004`): `reject_append_only_mutation`, statement-level `BEFORE UPDATE OR DELETE OR TRUNCATE` triggers on identity/manifest tables, `REVOKE UPDATE`/`DELETE`/`TRUNCATE` from `tepp_app_runtime`, executable DDL/rollback contracts, and live representative mutation proof.
- `persistence_postgres` model-run artifact chain: migration `0003_model_run_artifact_chain` for append-only `corpus_split_manifest`, `model_run`, and `model_artifact` with FORCE RLS; SQL insert/lookup contracts and live repository methods binding runs to reproducibility manifests and optional splits.
- `persistence_postgres` append-only reproducibility-manifest SQL contracts and live repository methods (`insert_reproducibility_manifest`, digest/id lookup) with fail-closed SHA-256 and commit identity validation for `reproducibility_manifest`.
- `persistence_postgres` tenant row-level security: migration `0002_tenant_row_level_security`, `tepp_app_runtime` role, session GUC `tepp.current_tenant_record_id`, multi-word isolation policies with FORCE RLS, session helpers, contract validation, and live isolation proof under `TEPP_LIVE_POSTGRES=1`.
- `persistence_postgres` live PostgreSQL CI: `live-postgres` job with Postgres 16 service, `TEPP_LIVE_POSTGRES=1` gate, and integration coverage for pool open, foundation+RLS migrations, document insert/revise/as-of, audit SQL, and tenant isolation.
- Repository release evidence tooling: `scripts/release_evidence.py` generates CycloneDX 1.5 SBOM, exact-head provenance, and SHA-256 checksums from `Cargo.lock`/`Cargo.toml`, with fail-closed validation and CI generation on every quality gate.
- `persistence_postgres` `live-sqlx` feature: real `SQLx`/`PgPool` open/execute behind validated `DATABASE_URL` and `LiveSqlxPoolOptions`, with offline/live executor backends and CI coverage exclusion for the transport module.
- `persistence_postgres` live pool open gate: validated `LiveSqlxPoolOptions`, fail-closed `open_live_sqlx_pool` / `LiveSqlxPool` (`SqlSession`) with offline test backend; optional `live-sqlx` attaches real `SQLx`/`PgPool` after `DATABASE_URL` validation.
- `validation_core` recovery metrics: parameter matching, RMSE/bias with standard errors, interval coverage with Wilson bounds, relation-edge precision/recall, temporal-order accuracy, Monte Carlo summaries, and SE-aware acceptance gates with machine-readable reports.
- `tepp_api` versioned analysis-run DTOs, content-redacting error envelopes, reproducibility manifests, JSON-LD and GraphML export contracts, purpose-bound export authorization (no blanket PII masking), plus committed schemas/examples.
- `relation_graph` forward-only state-transition DAG with past-pointing provenance edges and cycle rejection.
- `tepp_simulation` deterministic truth-corpus generator with delayed reporting, multilevel memberships, method-effect variants, relation noise, and digest-bound truth manifests.
- `corpus_split` leakage-safe knowledge-cutoff snapshots, relation-connected co-partition groups, rolling-origin windows, and group-normalized ESS weight contracts.
- `persistence_postgres` live SQL port: `SqlSession` transport, migration batch applicator, document/audit SQL contracts, `LiveDocumentRepository`, and fail-closed `DATABASE_URL`/`LiveSqlxConfig` gate for SQLx pool wiring (optional `live-sqlx` driver attaches `PgPool`).
- `membership_core` Kish effective sample size, design effect, and group-normalized ESS helpers for multiple-membership estimation inputs.
- Credential-separated hourly NVIDIA NIM/OpenCode product-development workflow (issue #2): proposal, independent verification, and late Maintainer-App publication with `NVIDIA_NIM_API_KEY` only for model work.
- Documented modular naruon consumer contract for TEPP analysis-run and export surfaces, with a committed example request payload.
- Documented contextual-orchestrator interpretation port boundary and credential separation for TEPP.
- Foundation validation/release-readiness ledger at `docs/validation/temporal-event-foundation.md` tracking capability maturity and scientific acceptance gates.
- Research doctoring for multilevel/multiple-membership measurement and atomistic fallacy prevention.
- `persistence_postgres` bitemporal foundation: multi-word migration contracts, knowledge-cutoff eligibility, and in-memory as-known-at / as-valid-at document replay (live SQLx/PostgreSQL execution remains accepted-target).
- `event_core` mention/instance separation with explicit promotion, typed roles, event-time validity, and fail-closed mention-as-instance refusal.
- `membership_core` time-varying weighted multiple-membership network with contextual roles, event-time validity, and atomistic-fallacy prevention contracts.
- Bounded Allen interval algebra and path-consistency reasoner in `temporal_core` (Task 4; PR #9), with identity-isolated variables, resource budgets, inverse/composition, and conservative provenance.
- Approved Temporal Event Psychometrics Platform PRD v0.4 baseline.
- Canonical technical documentation map, TRD, UML/scientific runtime views, logical/planned ERD, scientific test strategy, operability/recovery guide, and requirements/research/evidence traceability with explicit implementation maturity.
- Whole-conversation documentation fitness assessment plus canonical API/MSA contract, threat model, privacy/data-governance contract, CSAP/SOC 2/ISO/NIST assurance-readiness mapping, and adaptive LLM orchestration/test-time-compute contract.
- ADR policy separating architectural **Decision status** from **Implementation maturity**, defining partial supersession, and making the ADR index the canonical decision-ownership map.
- ADR 0009 for purpose-bound PII governance without blanket masking, ADR 0010 for adaptive direct-versus-multi-agent LLM orchestration, and ADR 0011 for standalone/modular CWL service authority.
- ADR 0012 for Temporal Relational Shared-Latent Topic Measurement (TRSL-TM), global topic identity/backend compatibility/method-effect/model-selection authority.
- ADR 0013 for bitemporal persistence, immutable reproducibility/run/split manifests, relation-aware partitions, recovery, and PostgreSQL adapter authority.
- ADR 0014 separating accepted design, protected-main implementation, scientific/product claim promotion, and release evidence authority.
- ADR 0015 separating autonomous model proposal, deterministic verification, publication, independent review, and merge/release authority.
- ADR 0016 separating Event Ontology observation, TDT detection/tracking, CHRONOS schema prediction, symbolic temporal consistency, and promoted transition authority.
- Verified APA 7 research traceability for ICLR 2026 TRINITY and Conductor, the 2026 Sakana Fugu technical report, ISO/IEC 42001:2023, ISO/IEC 23894:2023, NIST AI RMF/GAI Profile, AICPA Trust Services Criteria, and KISA CSAP guidance.
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
- Same-run exact missing-line and missing-branch diagnostics for failed 100% Rust coverage gates.

### Changed

- Clarified ADR 0001 so it owns Rust-first numerical/reference-backend authority while ADR 0011 owns cross-service MSA/service authority.
- Clarified ADR 0006 so it owns GPU/VRAM and model-credential boundaries; ADR 0010 now owns LLM orchestration policy and ADR 0015 owns autonomous repository-write/review/merge authority.
- Expanded ADR 0002–0005 and 0009–0011 with explicit implementation maturity, alternatives, failure/recovery, compatibility/migration, verification, and rollback/supersession boundaries where they were previously implicit.

### Security

- Disabled-state classification and live disable confirmation now accept GitHub's official `disabled_manually`, `disabled_fork`, `disabled_inactivity`, and `deleted` registry states so orphan bootstrap/repair identities can be retired without name-only heuristics.
- Prohibited `COPILOT_GITHUB_TOKEN` and reserved `NVIDIA_NIM_API_KEY` for approved LLM test and development workflows.
- Defined purpose-bound PII access, opaque analytical identifiers, separately protected identity mapping, selective model-provider disclosure, retention/deletion, and privileged audit controls instead of destructive blanket masking.
- Added explicit threat classes for temporal leakage, relation/membership poisoning, model/artifact poisoning, numerical divergence, cross-tenant disclosure, prompt injection, evidence substitution, resource exhaustion, and scientific-integrity failures.
- Removed the bootstrap branch's credential-co-resident OpenCode workflow: no model process may receive repository-write authority, and scheduled product development remains disabled until proposal, independent verification, and late publication authority are separated across fresh jobs.
- Removed completed bootstrap materializers, encoded payload fragments, readiness sentinels, and push probes from the reviewable tree.
- Required full-commit GitHub Action pins, minimum permissions, concurrency controls, immutable audit evidence, SBOM, and provenance.
- Kept ordinary Rust CI free of LLM and reviewer credentials and disabled persisted checkout credentials.
- Refused to cache mutable Cargo registry, Git source, or target trees; cached quality binaries are keyed and checked by exact version.
- Copied caller-provided source bytes and document text before acceptance and kept validated evidence fields private.
- Made empty, oversized, malformed-digest, invalid UTF-8-boundary, coordinate-mismatch, cross-document, nonfinite-geometry, and out-of-page evidence fail closed with content-redacting errors.
- Rejected malformed or extended wire payloads, unsupported schema versions, invalid identifiers and byte values, digest/content substitution, stale document ownership, and invalid nested geometry during reconstruction.

### Quality

- Required 100% production line and branch coverage and complete public API docstrings.
- Required true-parameter recovery, RMSE, bias, interval coverage, temporal leakage, graph recovery, invariance, and CPU/GPU parity evidence.
- Expanded documentation contracts to require the canonical threat/privacy/assurance/API/orchestration/fitness documents, ADR policy, and every numbered ADR 0001–0016 to remain indexed and structurally complete.
- Added deterministic validation that ADR files and the index have identical decision numbers and that every ADR declares valid decision status, implementation maturity, supersession scope, core decision sections, verification, and rollback behavior.
- Added 100% statement and branch coverage for the repository quality-gate scripts.
- Made a zero executable-code coverage denominator explicit for the skeleton-only slice rather than treating it as evidence of implemented behavior.
- Denied warnings, missing public documentation, and unsafe Rust across the workspace.
- Added known digest vectors, mutation detection, hostile multibyte Unicode, exact-coordinate, page-boundary, stable-error, and invalid-input regression tests for the first evidence slice.
- Added strict wire round trips, unknown-field and version rejection, digest reconstruction, configured-limit, hostile JSON, and generated multilingual span tests.

The repository has not yet cut a stable implementation release, so no compare reference is published for `[Unreleased]` yet.
