# TEPP API and Modular Integration Contract

**Status:** Accepted target contract; exact endpoints are introduced only with executable services.  
**Last reviewed:** 2026-08-16

## 1. Authority boundary

TEPP must work both as a standalone product and as a modular CWL component. Integrations with `naruon`, `contextual-orchestrator`, `.github`, or other repositories use explicit versioned API/artifact contracts. Cross-service direct table access is prohibited.

Current protected main exposes Rust library/domain contracts. The active stack adds a loopback HTTP/1.1 listener for naruon analysis-run, LineageWeave temporal-context, and export POSTs. `tepp-loopback` runs the shared consumer listener on `127.0.0.1:18081` by default; a caller may pass another loopback socket address as its sole argument. The container is intended for a trusted same-host or shared-network-namespace sidecar, checks readiness through a synthetic bounded temporal-context request, and deliberately cannot bind a public or bridge address. It is not a production TLS/`$PORT` service. Endpoint examples below that are not covered by `NaruonLiveService` remain target interface shapes.

## 2. Contract families

| Contract | Owner | Consumers | Maturity |
|---|---|---|---|
| evidence record/span wire v1 | TEPP `evidence_core` | future TEPP services/adapters | implemented-main |
| temporal clock/interval wire | `temporal_core` | relation/event/persistence | active-PR #5 |
| interval relation/reasoner API | `temporal_core` | event/relation validation | active-PR #6 |
| event/relation/membership API | future TEPP crates/services | naruon, analytics, UI | accepted-target |
| semantic/topic measurement API | future TEPP measurement service | naruon, batch jobs, visual analytics | accepted-target |
| LLM interpretation provider port | `tepp_api` orchestration router + future HTTP gateway | contextual-orchestrator | partial |
| model/artifact/export API | `tepp_api` export envelopes + future HTTP service | standalone UI/CWL consumers | partial |
| analysis-run request/accepted contracts | `tepp_api` v1 wire DTOs | naruon, orchestrator, UI | active-PR |
| temporal-context ordering contract | `tepp_api` v1 wire DTOs | LineageWeave | active-PR |

## 3. Versioning

Every externally consumable contract has an explicit semantic contract version independent of software package version. Breaking changes require a new contract version, migration/compatibility notes, contract tests, and an ADR when they change measurement meaning, temporal semantics, ontology, evidence identity, or authorization.

Wire payloads:

- reject unknown fields unless a version explicitly defines extensibility;
- use opaque identifiers, not positional/numeric public authority;
- preserve source/model/config/provenance identities;
- reconstruct through domain validation rather than deserialize directly into private state;
- use stable machine-readable error codes plus content-redacting messages.

## 4. Target HTTP resource model

When the service layer is introduced, use resources such as:

```text
POST   /v1/evidence-imports
GET    /v1/evidence-imports/{import_id}
POST   /v1/analysis-runs
POST   /v1/temporal-context
GET    /v1/analysis-runs/{run_id}
POST   /v1/analysis-runs/{run_id}/cancel
GET    /v1/model-artifacts/{artifact_id}
GET    /v1/exports/{export_id}
```

Long-running analysis is durable asynchronous work. `POST /v1/analysis-runs` accepts an idempotency key, immutable input snapshot identity, knowledge cutoff, versioned model contract/configuration, and requested output profile. A retry with the same principal/idempotency key and semantically identical request returns the same run identity; a conflicting body fails closed.

`POST /v1/temporal-context` is a bounded LineageWeave read contract. It accepts
only events whose availability time is at or before `knowledge_cutoff`, orders
them by event time and opaque event ID, and emits adjacent forward temporal
associations plus `candidate_not_causal` transition gaps. It does not infer
causality, mutate TEPP state, or return a completed psychometric result.

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

TEPP may call a provider-neutral interpretation/orchestration port for semantic unitization, blinded model review, and evidence-bounded interpretation. Callers first obtain a plan from `tepp_api::route_orchestration` and may bind it with `tepp_api::bind_contextual_orchestrator` using an evidence-manifest digest. The orchestrator does not own TEPP's statistical truth, source evidence, model registry, merge/release authority, or scientific acceptance. Detailed port boundary and credential separation are recorded in [`docs/connectors/contextual-orchestrator-interpretation-port.md`](connectors/contextual-orchestrator-interpretation-port.md).

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
