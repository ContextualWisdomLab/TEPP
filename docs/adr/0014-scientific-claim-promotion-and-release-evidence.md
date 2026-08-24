# ADR 0014 — Scientific claim promotion and release evidence authority

**Decision status:** Accepted
**Implementation maturity:** partial — claim/promotion authority documented; repository SBOM/provenance evidence generator and CI validation implemented; checkpoint-versus-estimator refusal is `checkpoint_authority` on the active PR; full package/image release bundle and scientific claim promotion packages remain accepted-target
**Date:** 2026-08-12
**Supersedes:** None; extends ADR 0007 from repository quality tooling to product/scientific claim authority.

## Context

TEPP can have green unit tests while a scientific claim is still unsupported, or an accepted design while no protected-main implementation exists. Conversely, a model may pass a numerical study while security, provenance, migration, accessibility, or operational evidence is incomplete. Conflating these evidence classes creates misleading product maturity and release claims.

## Decision

TEPP separates four authorities:

1. **Decision authority** — accepted PRD/ADR architecture.
2. **Implementation authority** — source integrated on protected `main`.
3. **Scientific/product claim authority** — exact implementation plus claim-specific validation evidence.
4. **Release authority** — one exact protected head satisfying repository, scientific, security, provenance, operational, and review gates together.

The canonical maturity labels in `docs/adr/ADR_POLICY.md` and `docs/TRACEABILITY.md` are mandatory. An `Accepted` ADR never means the capability is implemented. An `implemented-main` claim requires source integrated on protected `main` plus the relevant exact-head tests, scientific/recovery/validation evidence, security/supply-chain evidence, and qualifying review required by live policy.

Scientific claims require evidence appropriate to the claim, including as applicable true-parameter recovery, bias, RMSE, interval/credible coverage, convergence, invariance, calibration, graph/network recovery, language alignment, CPU/GPU parity, temporal leakage prevention, or realistic external-validation evidence. Correlation or fit alone cannot substitute for parameter recovery or validity evidence.

A software/model release additionally requires exact protected-head CI/security, 100% owned production coverage/public documentation, package/image reproducibility, dependency lock, SBOM/provenance, compatibility, migrations/rollback/recovery where applicable, privacy/security evidence, accessibility for shipped UI, operator acceptance, CHANGELOG/version/tag consistency, and post-publish artifact verification.

## Alternatives considered

1. **Treat green CI as release readiness** — rejected because CI can pass while scientific, operational, privacy, or provenance claims remain unproven.
2. **Treat accepted architecture as implemented capability** — rejected because it misrepresents active-PR/planned work.
3. **Use separate evidence authorities and explicit promotion rules** — accepted.

## Consequences

- PR bodies, chat, planning packs, queued checks, predecessor-head results, model judgments, or local-only experiments cannot promote shipped maturity by themselves.
- release evidence is bound to one exact protected commit and cannot be transferred to a changed head;
- scientific claim language must match the identification design and validation scope;
- the acquisition/commercialization bar is a prioritization target, not a valuation result or certification claim.

## Failure and recovery

If evidence is absent, stale, ambiguous, failed, skipped-required, from another head, or from an inapplicable environment, the corresponding claim remains unpromoted. Recovery requires reproducing the relevant gate on the exact candidate head rather than editing wording to imply success.

## Security, privacy, and governance impact

Security/privacy failures and unsupported scientific claims are integrity failures. Independent reviewer/model roles remain separate from merge/release authority. Neither an LLM nor an automated status comment can manufacture qualifying approval or scientific truth.

## Compatibility and migration

A claim-promotion policy change must update Traceability, Test Strategy, Operability, release guidance, and relevant ADRs. Existing published artifacts retain the policy/version under which they were accepted; reclassification requires explicit new evidence rather than retroactive relabeling.

## Verification

Repository tests must validate maturity vocabulary, documentation-to-implementation status, exact-head evidence binding, non-vacuous coverage gates, no skipped required tests, and release-manifest consistency. Domain validation suites provide the claim-specific numerical/scientific evidence defined by their owning ADRs.

## Rollback and supersession

A release candidate that loses a required gate is withdrawn rather than patched around the gate. Supersede only with a decision that preserves the separation between design, implementation, scientific claim, and release authority and provides stronger auditable promotion rules.
