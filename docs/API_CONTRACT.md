# TEPP API and Modular Integration Contract

**Status:** Accepted target contract; exact endpoints are introduced only with executable services.  
**Last reviewed:** 2026-08-24
**Last reviewed:** 2026-08-21

## 1. Authority boundary

TEPP must work both as a standalone product and as a modular CWL component. Integrations with `naruon`, `contextual-orchestrator`, `.github`, or other repositories use explicit versioned API/artifact contracts. Cross-service direct table access is prohibited.

Current protected main exposes Rust library/domain contracts. The active stack adds a loopback HTTP/1.1 listener for naruon analysis-run, LineageWeave temporal-context, and export POSTs, including `POST /v1/project-histories` on the `AnalysisRunLiveService` contract boundary. `tepp-loopback` runs the shared consumer listener on `127.0.0.1:18081` by default; a caller may pass another loopback socket address and an optional maximum request count as its two arguments. The container is intended for a trusted same-host or shared-network-namespace sidecar, checks readiness through a synthetic bounded temporal-context request, and deliberately cannot bind a public or bridge address. It is not a production TLS/`$PORT` service. Endpoint examples below that are not covered by `NaruonLiveService` or `AnalysisRunLiveService` remain target interface shapes; export retrieval stays a target shape until an executable export route ships.

## 2. Contract families

| Contract | Owner | Consumers | Maturity |
|---|---|---|---|
| evidence record/span wire v1 | TEPP `evidence_core` | future TEPP services/adapters | implemented-main |
| temporal clock/interval wire | `temporal_core` | relation/event/persistence | implemented-main (`temporal-core/v1`; wire `schema_version=1`; merged PR #8; [`wire_contract.rs`](../crates/temporal_core/tests/wire_contract.rs), [`schema_semantics_contract.rs`](../crates/temporal_core/tests/schema_semantics_contract.rs), [`temporal-event-foundation.md`](validation/temporal-event-foundation.md)) |
| interval relation/reasoner API | `temporal_core` | event/relation validation | implemented-main (`temporal-core/v1`; in-memory reasoner; merged PR #9; Allen, 1983; [`relation_contract.rs`](../crates/temporal_core/tests/relation_contract.rs), [`reasoner_contract.rs`](../crates/temporal_core/tests/reasoner_contract.rs), [`temporal-event-foundation.md`](validation/temporal-event-foundation.md)) |
| event/relation/membership API | future TEPP crates/services | naruon, analytics, UI | accepted-target |
| semantic/topic measurement API | future TEPP measurement service | naruon, batch jobs, visual analytics | accepted-target |
| topic-context posterior plausible values | `analysis_engine` `tepp.topic_context_posterior.v2` | fast-mlsirm, LineageWeave | contract-only active-PR |
| LLM interpretation provider port | `orchestrator_live` loopback `POST /v1/interpretation-runs` | contextual-orchestrator | partial |
| LLM interpretation provider port | `tepp_api` orchestration router + future HTTP gateway | contextual-orchestrator | partial |
| model/artifact/export API | `tepp_api` export envelopes + future HTTP service | standalone UI/CWL consumers | partial |
| analysis-run request/accepted/status/terminal-result contracts | `tepp_api` v1 wire DTOs | naruon, orchestrator, UI | active product branch |
| corpus-split leakage-audit manifest | `tepp_api` `CorpusSplitManifest` v1 | naruon, auditors, future UI | active-PR |
| temporal-context ordering contract | `tepp_api` v1 wire DTOs | LineageWeave | active-PR |
| cutoff-safe analysis-run readiness execution | `analysis_engine` bounded Rust crate | `tepp_api`, future HTTP/service adapters | active product branch |
| project-history projection contract | `tepp_api` v1 wire DTOs | LineageWeave | active-PR |
| analysis-run status/terminal-result contracts | `tepp_api` v1 wire DTOs | naruon, orchestrator, UI | active-PR #157 |
| cutoff-safe analysis-run readiness execution | `analysis_engine` bounded Rust crate | `tepp_api`, future HTTP/service adapters | active-PR |

## 3. Versioning

Every externally consumable contract has an explicit semantic contract version independent of software package version. Breaking changes require a new contract version, migration/compatibility notes, contract tests, and an ADR when they change measurement meaning, temporal semantics, ontology, evidence identity, or authorization.

The temporal semantic contract is `temporal-core/v1`. Its JSON representation keeps `schema_version: 1` as a separate wire-schema field; changing either identifier requires its own compatibility evidence.

Wire payloads:

- reject unknown fields unless a version explicitly defines extensibility;
- use opaque identifiers, not positional/numeric public authority;
- preserve source/model/config/provenance identities;
- reconstruct through domain validation rather than deserialize directly into private state;
- use stable machine-readable error codes plus content-redacting messages.

The TEPP-owned Event Lineage criterion artifact is
`tepp.lineage_criterion_anchor.v1` (`schemas/lineage_criterion_anchor_v1.json`).
A LineageWeave analysis run requests model contract
`tepp-lineage-criterion-v1` and output profile
`lineage_pair_criterion_anchor`. The artifact carries TEPP's accepted or
rejected criterion-validity outcome bound to one fast-mlsirm estimation run,
snapshot, cutoff, and validated pair count. The contract does not authorize a
consumer to self-assert validity; no weight vector activates until TEPP's
registered implementation returns the digest-bound artifact.

## 4. Target HTTP resource model

When the service layer is introduced, use resources such as:

```text
POST   /v1/evidence-imports
GET    /v1/evidence-imports/{import_id}
POST   /v1/interpretation-runs
POST   /v1/analysis-runs
POST   /v1/temporal-context
GET    /v1/analysis-runs/{run_id}
POST   /v1/analysis-runs/{run_id}/cancel
GET    /v1/model-artifacts/{artifact_id}
GET    /v1/exports/{export_id}
```

Long-running analysis is durable asynchronous work. `POST /v1/analysis-runs` accepts an idempotency key, immutable input snapshot identity, knowledge cutoff, versioned model contract/configuration, and requested output profile. A retry with the same principal/idempotency key and semantically identical request returns the same run identity; a conflicting body fails closed.

The typed status/read contract returns `accepted`, `running`, `succeeded`, or
`failed`. Accepted and running statuses contain no measurement result. A
terminal status contains exactly one request-bound `AnalysisRunTerminalResult`;
consumers validate its request, receipt, snapshot, cutoff, model, profile, and
idempotency bindings before treating it as measurement evidence.

The stacked `analysis_engine` slice provides the first executable service-side
path behind these DTOs. It consumes a bounded identity-free snapshot, excludes
evidence unavailable at the historical cutoff, preserves multiple-membership
counts, and emits a digest-bound terminal result or a redacted failure. It is
not a substitute for approved topic or psychometric estimators.

`POST /v1/temporal-context` is a bounded LineageWeave read contract. It accepts
only events whose availability time is at or before `knowledge_cutoff`, orders
them by event time and opaque event ID, and emits adjacent forward temporal
associations plus `candidate_not_causal` transition gaps. It does not infer
causality, mutate TEPP state, or return a completed psychometric result.

The typed status/read contract returns `accepted`, `running`, `succeeded`, or
`failed`. Accepted and running statuses contain no measurement result. A
terminal status contains exactly one request-bound
`AnalysisRunTerminalResult`; consumers must validate its request, receipt,
snapshot, cutoff, model, profile, and idempotency bindings before treating the
run as measurement evidence. The Rust DTO is available before the future HTTP
service is deployed.

The stacked `analysis_engine` slice provides the first executable service-side
path behind these DTOs. It consumes a bounded identity-free snapshot, excludes
evidence unavailable at the historical cutoff, preserves multiple-membership
counts, and emits a digest-bound terminal result or a redacted failure. For the
`trsl_topic_lineage_v1` profile it invokes the ADR-0012 `topic_measurement`
reference estimator and publishes validated fitted associations, counts,
candidate-fit evidence, and separate source-snapshot and numerical-input digests
in `tepp.trsl_topic_lineage.v2`; it does not infer causality or replace
production `K` selection. This remains active product-branch evidence until its exact-head
checks and protected merge pass.

Version 1 artifacts cannot be upgraded by filling fields: they do not contain
the candidate-fit evidence or complete numerical-input digest required by v2.
Clients retain v1 as historical evidence and rerun its immutable source snapshot
at the original knowledge cutoff to produce v2; the parser rejects v1 rather
than inventing missing scientific provenance.

The separate `tepp.topic_context_posterior.v2` artifact carries per-post
posterior logistic-normal plausible values, a declared event clock, opaque
stable topic identities, artifact-local coordinate order, topic activity,
explicit topic-lineage events, admitted Event Lineage document relations, and
provenance-bound time-valid business-unit, PU, team, and person memberships.
The ordered `topic_ids` array defines coordinate order; `topic_basis_sha256`
binds that order to `posterior_draw_set_id` and fails closed on relabeling. Each lineage,
document-relation, or membership assertion identifies its immutable evidence
resource, provenance assertion, and digest so consumers can materialize
normalized qualified provenance. It is the only admitted handoff to the
fast-mlsirm context-influence estimator. Consumers may not threshold its
coordinates into binary responses, collapse its draws to an error-free point,
or substitute labels, keywords, or LineageWeave-local scores.
Serialization sorts every record collection by its stable identity/time key,
so input permutations produce the same canonical JSON and SHA-256. The sole
document-relation kind in v1 is `event_lineage_precedes`; its source document
event time cannot follow its target document event time.
The CPU reference path emits bound joint posterior draws and materializes the
complete document-by-draw plausible-value grid through
`assemble_topic_context_posterior`. No accepted GPU or asynchronous producer
result exists.
The JSON Schema is the bounded record-shape contract. Cross-record invariants
that require joining opaque document identities—at least two distinct
documents, a complete document-by-draw grid, and all four time-covering
membership dimensions for every document—are enforced by the Rust domain
validator and cannot be established by schema-only acceptance. A missing
optional lineage target and an explicit JSON `null` both deserialize to the
same Rust `Option` value; the schema therefore admits both representations.

## 5. Analysis request authority

An analysis request cannot supply arbitrary facts that bypass validated domain state. The service resolves and validates:

- authorized tenant/workspace;
- immutable evidence/corpus snapshot;
- `knowledge_cutoff` and availability eligibility;
- relation-aware split policy;
- model/backend contract version;
- language validation profile;
- optional LLM/provider policy;
- output/export authorization.

LLM/provider settings are execution policy, not permission to alter source evidence or scientific acceptance criteria.

## 6. Run lifecycle

Canonical target states:

```text
accepted -> validating -> queued -> running -> verifying -> completed
                                    |           |
                                    v           v
                                 failed      rejected
                                    |
                                    v
                                 retryable

accepted/running -> cancelling -> cancelled
```

State transitions are server-authoritative, versioned, idempotent, and auditable. `completed` means required deterministic/scientific verification for the run profile passed; it does not imply a software release or causal validity claim.

## 7. Error envelope

Target error shape:

```json
{
  "error_code": "temporal_evidence_unavailable",
  "message": "Evidence is not eligible for the requested historical cutoff.",
  "request_id": "opaque-request-id",
  "retryable": false
}
```

Error payloads never echo credentials, unrestricted source text, raw provider responses, internal paths, SQL, or hidden policy details.

## 8. Pagination, limits, and streaming

Collection APIs use bounded cursor pagination. Large imports/exports use streaming or object references rather than unbounded in-memory JSON. Every endpoint defines maximum request size, item count, graph size/depth, timeout/deadline, and response size. Resource exhaustion produces explicit bounded failure/defer states.

## 9. Standalone and CWL composition

### Standalone

TEPP owns its application/API state, authorized evidence, model runs, and artifacts. A standalone deployment may use local or private LLM providers and CPU-only compute.

### naruon

`naruon` may submit evidence/analysis requests or consume versioned topic/event/psychometric artifacts. It must not treat lexical heuristics as TEPP topic inference and must not read TEPP database tables directly. HTTP interchange is `tepp_api::naruon_analysis_run_exchange` / `naruon_export_exchange` (`POST /v1/analysis-runs` and `/v1/exports` over `https` only). Detailed modular surfaces and failure modes are recorded in [`docs/connectors/naruon-artifact-consumer.md`](connectors/naruon-artifact-consumer.md).

### Provider payload minimization

Before any naruon, contextual-orchestrator, or NVIDIA NIM submission, callers must build the payload through `tepp_api::minimize_provider_payload`. That function preserves opaque analytical identifiers and membership roles, applies purpose-bound source-text rules, refuses expired, not-yet-valid, inverted, cross-tenant, or impossible-calendar grants, and never copies a direct identity mapping into the provider body or ordinary log. Re-identification is a separate elevated scientific path (`disclose_identity_mapping`), not a provider header or prompt field.

### contextual-orchestrator

TEPP may call a provider-neutral interpretation/orchestration port for semantic unitization, blinded model review, and evidence-bounded interpretation. Callers first obtain a plan from `tepp_api::route_orchestration` and may bind it with `tepp_api::bind_contextual_orchestrator` using an evidence-manifest digest. The standalone `orchestrator_live::OrchestratorLiveService` also serves a loopback-only `POST /v1/interpretation-runs` proof listener; the listener is not TLS termination. A production live port must pass `service_tls::authorize_orchestrator_live_port` (valid rustls PEM on an `https` bind); loopback plaintext is refused and loopback `https` with valid PEM is authorized as production TLS. The orchestrator does not own TEPP's statistical truth, source evidence, model registry, merge/release authority, or scientific acceptance. Detailed port boundary and credential separation are recorded in [`docs/connectors/contextual-orchestrator-interpretation-port.md`](connectors/contextual-orchestrator-interpretation-port.md).

### organization `.github`

Organization workflows provide CI/review/security/release control-plane services but do not become runtime scientific authority.

## 10. Artifact contract

Every analytical/export artifact binds at least:

- source/evidence manifest identity;
- relation-aware split identity;
- knowledge cutoff;
- preprocessing/concept dictionary version;
- model contract and engine version;
- configuration and seed manifest;
- compute backend/precision contract;
- Git/dependency/provenance identity;
- validation/uncertainty status;
- artifact content hash.

Cross-format exports (JSON-LD, GraphML, CSV, Arrow/Parquet, SVG/PDF) must be semantically consistent with the same source artifact model; graphical output is never the sole representation of exact values.

## 11. Compatibility tests

Consumer/provider contract tests must cover version negotiation, unknown fields, size/depth limits, idempotency, stale/invalid snapshot identity, future evidence, tenant/purpose denial, cancellation, retry semantics, artifact digest mismatch, and graceful handling of unsupported model/language capabilities.
