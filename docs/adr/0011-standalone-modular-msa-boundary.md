# ADR 0011 — Standalone operation and modular CWL MSA boundary

**Decision status:** Accepted
**Implementation maturity:** partial
**Date:** 2026-08-10
**Supersedes:** The broad cross-service ownership wording in ADR 0001. ADR 0001 remains authoritative for Rust-first numerical/backend requirements.

## Context

TEPP must be independently deployable while also composing with ContextualWisdomLab services such as `naruon`, `contextual-orchestrator`, `fast-mlsirm`, and organization control-plane workflows. Hidden database coupling, copied scientific kernels, or implicit cross-repository authority would make the product difficult to deploy, audit, version, acquire, or reuse.

The approved PRD defines TEPP's product and measurement target as multilingual temporal relational psychometrics. It does not require TEPP to duplicate reusable static psychometric kernels that have a separate canonical owner. The delivery recovery exposed both duplicated/static psychometric fragments in TEPP and an upstream `fast-mlsirm` model-specification/dependence contract under development. The ownership boundary therefore needs to be explicit without changing the approved measurement target.

## Decision

TEPP owns its evidence-domain contracts, six-clock temporal semantics, event ontology and temporal graph, time-varying multilevel/cross-classified/multiple-membership composition, longitudinal invariance/drift, temporal state evolution, temporal recovery policy, TEPP model/run provenance, and TEPP-owned persistence.

`fast-mlsirm` owns reusable static/generalized-mixed/dependence-aware psychometric model specification and reusable numerical kernels, including reusable LSIRM, MLSIRM, and DLSJM computation. TEPP consumes only released/versioned `fast-mlsirm` contracts through an anti-corruption layer and composes TEPP-owned temporal/event semantics around the full upstream candidate identity. An open upstream PR or branch is not a production dependency.

Reusable computation discovered in TEPP that belongs to the `fast-mlsirm` domain is migrated through the canonical-owner path: establish a versioned public contract, prove numerical/recovery parity, switch TEPP to the adapter, then remove the duplicate production source. TEPP does not keep a second canonical implementation merely to avoid cross-repository work.

Standalone deployments may run CPU-only and may select local/private execution backends. Modular deployments preserve the same scientific, temporal, and authorization contracts.

Authority boundaries:

- `fast-mlsirm` owns reusable static/generalized-mixed/dependence model specification and reusable psychometric kernels; it does not own TEPP event ontology, six-clock semantics, knowledge-cutoff policy, temporal graph, or temporal state-composition policy.
- `naruon` may submit authorized evidence/analysis work and consume versioned TEPP results; it does not replace TEPP inference with lexical heuristics or directly query TEPP tables.
- `contextual-orchestrator` owns model-provider execution, routing, fallback, verifier/adjudicator execution, credentials, and model-call provenance; it does not own TEPP source evidence, numerical scientific truth, claim promotion, or release authority.
- organization `.github` workflows own CI/review/security/release-control functions only; they are not runtime scientific authority.
- external PostgreSQL, object-storage, accelerator, and model-provider systems remain separately authenticated trust domains.

## Alternatives considered

1. **Shared organization database/schema** — rejected because it couples lifecycle, authorization, migrations, recovery, and acquisition boundaries.
2. **Copy reusable psychometric kernels into TEPP** — rejected because duplicated numerical authority drifts and makes parity, recovery, and maintenance ambiguous.
3. **Repository-specific bespoke adapters without a common contract** — rejected because semantics drift and become difficult to validate.
4. **Standalone TEPP temporal/event core plus released versioned owner ports/artifacts** — accepted.

## Consequences

- Public contracts carry version, identity, provenance, error, bounded-resource, compatibility, and owner semantics.
- Each service owns its credentials, migrations, retention, and application persistence.
- Cross-service workflows use opaque identifiers and explicit authorization rather than implicit shared state.
- TEPP temporal dependence compilation is generic over the complete released upstream candidate identity rather than hard-coded LSIRM/MLSIRM/DLSJM family wrappers.
- Auto-expansion does not imply activation: novel combined static/dependence/temporal formulations remain `research_candidate` until equations, identification, estimator, citations, and recovery evidence are complete.
- Breaking contract changes require compatibility/migration notes and an ADR where product/scientific meaning changes.
- Integration tests exercise standalone and representative modular paths, including contract-version/digest refusal.

This decision clarifies service authority without changing PRD v0.4's approved product or measurement target, so it does not itself require a PRD version increment under AGENTS contract 14.

## Failure and recovery

If an integration service is unavailable, TEPP either uses an approved local/backend fallback that preserves the same scientific contract or returns a bounded degraded/deferred state. It never fabricates external state, copies an unreleased owner implementation, or bypasses TEPP validation. Recovery revalidates artifact identity, authorization, temporal cutoff, contract version/digest, and owner provenance before resuming.

## Security/privacy

Least-privilege service identities and purpose-bound access apply at every interface. A service receives only the evidence/artifact fields it is authorized to process. No service credential is reused as a model/reviewer/release credential simply because the services share an organization. Cross-service SQL is prohibited.

## Compatibility and migration

Every public API/artifact contract is versioned. A breaking consumer/provider change requires migration/rollback guidance and dual-version or negotiated compatibility where necessary. Persistence migrations remain TEPP-owned under ADR 0013; consumers never migrate TEPP tables directly.

Static-kernel migration follows: TEPP duplicate -> parity/recovery evidence -> released `fast-mlsirm` contract -> TEPP ACL/adaptor -> duplicate removal. Rollback restores the last released compatible adapter contract; it does not restore a divergent duplicate as canonical authority.

## Verification

Required tests cover contract version/digest negotiation, unauthorized cross-service access, idempotency, stale artifact/model identities, missing dependencies, standalone CPU operation, contextual-orchestrator integration boundaries, naruon consumer contracts, absence of direct cross-service database coupling, and parity/recovery before any duplicated static kernel is removed.

Architecture fitness tests must also reject dependency inversion in which `fast-mlsirm` imports TEPP temporal ontology or TEPP deployable behavior binds directly to an unreleased upstream PR head.

## Rollback

Rollback removes or disables an integration adapter without breaking standalone TEPP or rewriting scientific artifacts. A failed owner migration returns to the last released compatible contract while retaining parity/recovery evidence and migration provenance. Supersede this ADR only through a repository-wide unique ADR that establishes a clearer ownership model without reducing standalone deployability, scientific authority separation, migration safety, or auditability.