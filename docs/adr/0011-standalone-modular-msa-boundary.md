# ADR 0011 — Standalone operation and modular CWL MSA boundary

**Decision status:** Accepted
**Implementation maturity:** partial — Rust crates are independently usable; naruon HTTP interchange (`POST /v1/analysis-runs` and `/v1/exports`, fail-closed table-access and credential headers) implemented on the active PR (not implemented-main); `service_tls` production rustls bind gates and orchestrator live-port refusal of loopback plaintext are on this PR; remaining live HTTP listeners and production persistence integrations remain accepted-target
**Date:** 2026-08-10
**Supersedes:** The broad cross-service ownership wording in ADR 0001. ADR 0001 remains authoritative for Rust-first numerical architecture.

## Context

TEPP must be independently deployable while also composing with ContextualWisdomLab services such as `naruon`, `contextual-orchestrator`, and organization control-plane workflows. Hidden database coupling or implicit cross-repository authority would make the product difficult to deploy, audit, version, acquire, or reuse.

## Decision

TEPP owns its evidence-domain contracts, temporal/event/membership state, psychometric/statistical model authority, model/artifact registry, run provenance, and TEPP-owned persistence. Other services integrate only through stable versioned APIs or immutable artifacts. Direct cross-service application-table reads/writes are prohibited.

Standalone deployments may run CPU-only and may select local/private providers. Modular deployments preserve the same scientific and authorization contracts.

Authority boundaries:

- `naruon` may submit authorized evidence/analysis work and consume versioned TEPP results; it does not replace TEPP inference with lexical heuristics or directly query TEPP tables.
- `contextual-orchestrator` may execute approved model-routing/orchestration requests but does not own TEPP's source evidence, statistical truth, scientific gates, model registry, or release authority.
- organization `.github` workflows own CI/review/security/release-control functions only; they are not runtime scientific authority.
- external PostgreSQL/object-storage/model providers remain separately authenticated trust domains.

## Alternatives considered

1. **Shared organization database/schema** — rejected because it couples lifecycle, authorization, migrations, recovery, and acquisition boundaries.
2. **Repository-specific bespoke adapters without a common contract** — rejected because semantics drift and become difficult to validate.
3. **Standalone core plus versioned ports/artifacts** — accepted.

## Consequences

- Public contracts carry version, identity, provenance, error, bounded-resource, and compatibility semantics.
- Each service owns its credentials, migrations, retention, and application persistence.
- Cross-service workflows use opaque identifiers and explicit authorization rather than implicit shared state.
- Breaking contract changes require compatibility/migration notes and an ADR where product/scientific meaning changes.
- Integration tests exercise both standalone and representative modular paths.

## Failure and recovery

If an integration service is unavailable, TEPP either uses an approved local/provider fallback or returns a bounded degraded/deferred state. It never fabricates external state or bypasses TEPP validation. Recovery revalidates artifact identity, authorization, temporal cutoff, and version compatibility before resuming.

## Security/privacy

Least-privilege service identities and purpose-bound access apply at every interface. A service receives only the evidence/artifact fields it is authorized to process. No service credential is reused as a model/reviewer/release credential simply because the services share an organization.

## Compatibility and migration

Every public API/artifact contract is versioned. A breaking consumer/provider change requires migration/rollback guidance and dual-version or negotiated compatibility where necessary. Persistence migrations remain TEPP-owned under ADR 0013; consumers never migrate TEPP tables directly.

## Verification

Required tests cover contract version negotiation, unauthorized cross-service access, idempotency, stale artifact/model identities, missing dependencies, standalone CPU operation, contextual-orchestrator optional integration, naruon consumer contracts, and absence of direct cross-service database coupling.

## Rollback and supersession

Rollback removes/disables an integration adapter without breaking standalone TEPP or rewriting scientific artifacts. Supersede only through an ADR that establishes a clearer ownership model without reducing standalone deployability, scientific authority separation, migration safety, or auditability.
