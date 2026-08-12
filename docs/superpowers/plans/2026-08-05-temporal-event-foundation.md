# Temporal/Event Foundation Implementation Plan

> **For agentic workers:** Use task-isolated development, test-driven implementation, independent review, and verification before completion.

**Goal:** Build the Rust temporal, event, relation, membership, persistence, split, simulation, and API foundation required by every later TEPP model.

**Architecture:** A Cargo workspace exposes small crates for immutable evidence, temporal algebra, event ontology, relation graphs, multiple membership, PostgreSQL persistence, leakage-safe splitting, simulation, and API schemas. The domain core is storage-independent. PostgreSQL and service adapters depend inward on versioned domain types.

**Tech stack:** Rust stable, Tokio where asynchronous I/O is needed, Serde, UUID v7, time/chrono after benchmark and license review, SQLx with PostgreSQL, Rayon only for bounded CPU-parallel workloads, proptest, cargo-nextest, cargo-llvm-cov, cargo-deny, rustdoc, JSON Schema, JSON-LD, GraphML.

## Global constraints

- Production mathematical and psychometric logic is Rust.
- Public and safety-relevant APIs have complete docstrings.
- Production line and branch coverage are 100%.
- Database object names contain at least two words and use `snake_case`.
- Event, assertion, document, system, availability, and cutoff time remain distinct.
- Historical queries enforce `available_time <= knowledge_cutoff`.
- Forward transition edges never reverse event time.
- Cross-classified and multiple-membership structures are first-class.
- Every acceptance claim has fresh focused and complete verification evidence.

---

## Task 1 — Cargo workspace and quality gates

**Files:** `Cargo.toml`, `rust-toolchain.toml`, `crates/*/Cargo.toml`, `.github/workflows/ci.yml`, `deny.toml`, `scripts/check_docstrings.py`.

**Produces:** compilable workspace, shared lint profile, exact test/coverage/docstring/security commands.

- [ ] Add a failing repository-contract test that requires all planned crate members, `unsafe_code = "forbid"` unless explicitly ADR-approved, warnings denied in CI, and workspace dependency centralization.
- [ ] Run the contract test and record the missing members.
- [ ] Create focused crate skeletons with module-level rustdoc and no placeholder production behavior.
- [ ] Run format, Clippy, rustdoc, unit tests, line/branch coverage, and dependency/license checks.
- [ ] Commit the workspace foundation with the exact verification transcript in the PR description.

## Task 2 — Immutable evidence identifiers and source records

**Files:** `crates/evidence_core/src/{lib.rs,identifier.rs,source_record.rs,source_span.rs,error.rs}` and matching tests.

**Produces:** `DocumentRecord`, `SourceArtifact`, `SourceSpan`, `EvidenceId`, SHA-256 verification, exact byte/character/page coordinates.

- [ ] Write failing tests for stable UUIDv7 identifiers, immutable content hashes, UTF-8 boundary validation, page/layout bounds, and mismatch rejection.
- [ ] Add property tests for arbitrary valid and invalid spans.
- [ ] Implement minimal validated constructors; make fields private and serialize through approved DTOs.
- [ ] Verify round trips, hostile Unicode, empty/oversized records, and mutation detection.
- [ ] Commit evidence contracts and update architecture/rustdoc.

## Task 3 — Six-clock temporal values and uncertain intervals

**Files:** `crates/temporal_core/src/{lib.rs,clock.rs,instant.rs,interval.rs,precision.rs,error.rs}` and tests.

**Produces:** distinct `EventTime`, `AssertionTime`, `DocumentTime`, `SystemTime`, `AvailableTime`, `KnowledgeCutoff`, bounded/open/uncertain intervals, precision metadata.

- [ ] Write failing compile-time and runtime tests proving clocks cannot be accidentally interchanged.
- [ ] Write interval tests for closed/open boundaries, unknown endpoints, date-only/month/quarter precision, timezone normalization, invalid ranges, and DST transitions.
- [ ] Implement typed wrappers and validated interval constructors.
- [ ] Add Serde/JSON Schema round-trip tests without losing precision or uncertainty.
- [ ] Verify 100% line/branch coverage and commit.

## Task 4 — Interval relations and temporal reasoner

**Files:** `crates/temporal_core/src/{relation.rs,reasoner.rs,partial_order.rs}` and tests.

**Produces:** before, after, meets, overlaps, starts, finishes, during, contains, equals, derived closure, contradiction evidence.

- [ ] Write failing table-driven tests for Allen/OWL-Time inverse and composition relations.
- [ ] Write property tests for antisymmetry, inverse consistency, transitive closure, and contradiction detection.
- [ ] Implement a bounded reasoner that returns derived relations with provenance rather than overwriting observations.
- [ ] Add cycle and complexity-limit failure tests.
- [ ] Compare curated examples with the standards register and commit.

## Task 5 — Event ontology domain model

**Files:** `crates/event_core/src/{lib.rs,event.rs,mention.rs,role.rs,subevent.rs,provenance.rs,error.rs}` and tests.

**Produces:** event instances and mentions, agents, factors, products, places, arguments, subevents, confidence, and exact evidence.

- [ ] Write failing tests separating event instance from document mention and requiring evidence for every inferred role.
- [ ] Test multiple mentions/languages/documents for one event and multiple candidate events for one ambiguous mention.
- [ ] Implement validated event and role types with versioned ontology identifiers.
- [ ] Add JSON-LD serialization and deterministic ordering tests.
- [ ] Commit the minimal-semantic-commitment event model.

## Task 6 — Typed relation graph and forward-transition invariant

**Files:** `crates/relation_graph/src/{lib.rs,node.rs,edge.rs,graph.rs,transition.rs,error.rs}` and tests.

**Produces:** observed/inferred document, segment, event, entity, revision, translation, citation, support, contradiction, retrospective, and transition edges.

- [ ] Write failing tests showing citation/revision may point backward while state transitions may not.
- [ ] Test confidence, evidence, direction, relation version, missing-edge semantics, and observed/inferred separation.
- [ ] Implement typed edge classes and a transition validator using partial-order evidence.
- [ ] Add cycle, duplicate, self-edge, contradictory-edge, and bounded-depth tests.
- [ ] Export deterministic GraphML/JSON-LD and commit.

## Task 7 — Time-varying cross-classified multiple membership

**Files:** `crates/membership_core/src/{lib.rs,entity.rs,role_assignment.rs,membership.rs,weights.rs,error.rs}` and tests.

**Produces:** authors, departments, organizations, customers, partners, competitors, projects, opportunity pools, templates, languages, locations, and episode assignments.

- [ ] Write failing tests for simultaneous memberships, time-varying roles, nonnested classifications, normalized and intentionally nonnormalized weights, and evidence confidence.
- [ ] Demonstrate that one organization can be customer, partner, and competitor in different contexts and intervals.
- [ ] Implement validated assignments without permanent role typing.
- [ ] Add aggregation tests preventing document-level atomistic conclusions about higher-level entities.
- [ ] Commit membership contracts and methodological notes.

## Task 8 — Bitemporal PostgreSQL schema and repository adapters

**Files:** `migrations/*.sql`, `crates/persistence_postgres/src/*`, `tests/postgres/*`.

**Produces:** `document_record`, `source_artifact`, `source_span`, `temporal_interval`, `event_instance`, `event_mention`, `event_relation`, `document_relation`, `segment_relation`, `relation_evidence`, `entity_record`, `entity_role_assignment`, `membership_assignment`, `audit_event`.

- [ ] Write migration-contract tests rejecting single-word object names and requiring temporal, foreign-key, exclusion/uniqueness, tenant, and immutable-audit constraints.
- [ ] Write failing integration tests for as-known-at and as-valid-at queries and `available_time <= knowledge_cutoff`.
- [ ] Implement forward and rollback migrations and SQLx repositories.
- [ ] Test concurrent writes, idempotency, revision history, invalid overlaps, deletion policy, and transaction rollback.
- [ ] Generate schema documentation and commit.

## Task 9 — Leakage-safe corpus snapshots and relation-aware splits

**Files:** `crates/corpus_split/src/{lib.rs,snapshot.rs,connected_group.rs,rolling_origin.rs,error.rs}` and tests.

**Produces:** knowledge-cutoff snapshots, relation-connected components, grouped train/validation/test and rolling-origin splits.

- [ ] Write failing tests excluding late-available retrospective documents from earlier cutoffs.
- [ ] Test that translations, revisions, copied variants, and same-episode records never cross partitions.
- [ ] Implement connected-group construction, deterministic seeded assignment, and rolling-origin windows.
- [ ] Add duplicate-aware effective-sample-size and group-normalized-weight contracts.
- [ ] Commit split algorithms and leakage audit output.

## Task 10 — Realistic temporal/event truth simulator

**Files:** `crates/tepp_simulation/src/{lib.rs,configuration.rs,latent_event.rs,document_process.rs,relation_process.rs,missingness.rs,truth_manifest.rs}` and tests.

**Produces:** known event states, temporal orders, memberships, document/report delays, revisions, translations, copied templates, observed/inferred relations, and truth manifests.

- [ ] Write failing deterministic-seed and truth-manifest tests.
- [ ] Simulate event occurrence separately from document creation and availability, including retrospective and delayed reporting.
- [ ] Simulate multilevel/multiple-membership effects, missingness, uncertain dates, relation noise, and template/copy method effects.
- [ ] Implement parameterized scenarios and verify generated invariants.
- [ ] Commit simulator and example datasets small enough for CI.

## Task 11 — Recovery metrics and Monte Carlo acceptance

**Files:** `crates/validation_core/src/{lib.rs,matching.rs,rmse.rs,bias.rs,coverage.rs,graph_metrics.rs,monte_carlo.rs}` and tests.

**Produces:** parameter matching, RMSE, bias, interval coverage, relation precision/recall, temporal-order accuracy, calibration, Monte Carlo uncertainty.

- [x] Write failing oracle tests for every metric, including degenerate and missing cases.
- [x] Implement confidence intervals or standard-error-aware acceptance rather than raw nominal point thresholds.
- [x] Add end-to-end truth-versus-recovered foundation studies.
- [x] Emit machine-readable and human-readable validation artifacts.
- [x] Commit metrics with formula and primary-source traceability.

## Task 12 — Versioned service/API contracts and exports

**Files:** `crates/tepp_api/src/*`, `schemas/*.json`, `examples/*.json`, API tests.

**Produces:** versioned ingestion, temporal query, event/relation, membership, snapshot/split, simulation, and validation contracts.

- [ ] Write failing schema tests for unknown fields, bounds, hostile nesting, tenant identifiers, exact evidence, temporal precision, and error redaction.
- [ ] Implement domain-to-DTO adapters without leaking persistence internals.
- [ ] Add JSON-LD and GraphML exports plus reproducibility manifests.
- [ ] Test backward compatibility and explicit version rejection.
- [ ] Commit API contracts and examples.

## Task 13 — Complete foundation verification and release-readiness report

**Files:** `.github/workflows/ci.yml`, `docs/validation/temporal-event-foundation.md`, `CHANGELOG.md`, SBOM/provenance configuration.

**Produces:** exact-head verification, benchmarks, security evidence, documentation, and a release decision.

- [ ] Run format, Clippy, rustdoc, all tests, property/fuzz suites, PostgreSQL integration, migration rollback, and package/install smoke tests.
- [ ] Run production line and branch coverage and public-docstring gates at 100%.
- [ ] Run deterministic and Monte Carlo temporal/event/membership/relation recovery studies and report RMSE, bias, coverage, precision/recall, and uncertainty.
- [ ] Generate SBOM, provenance, checksums, dependency/license/advisory reports, and reproducibility manifest.
- [ ] Update ADRs, architecture, research citations, CHANGELOG, operating limits, rollback, and next-phase interfaces; release only if every protected gate passes.
