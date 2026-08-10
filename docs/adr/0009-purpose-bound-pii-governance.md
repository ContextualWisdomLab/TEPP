# ADR 0009 — Purpose-bound PII governance without blanket masking

**Status:** Accepted  
**Date:** 2026-08-10

## Context

TEPP's psychometric, longitudinal, event, authorship, customer/partner/competitor, and cross-classified multiple-membership analyses can require identity-bearing or relationship-bearing data. Blanket masking can destroy the very grouping, linkage, time, authorship, or adjudication information needed for valid measurement. At the same time, unrestricted PII propagation into models, logs, exports, or cross-service integrations is unacceptable.

## Decision

TEPP does not use blanket masking as its primary privacy control. It uses purpose-bound authorization, opaque analytical identifiers, separately protected identity mapping, encryption, tenant isolation, selective disclosure, provider minimization, retention/deletion policy, and auditable privileged access.

Direct identity is removed from ordinary compute artifacts whenever the estimand does not require it, while scientifically relevant time-varying relationships are preserved through opaque identifiers. Sensitive derived results receive protection appropriate to their source and re-identification risk.

## Alternatives considered

1. **Mask all PII before ingestion** — rejected because it can invalidate author, entity-role, longitudinal, linkage, and multiple-membership measurement.
2. **Store raw identity everywhere and rely on network perimeter** — rejected because it creates ambient disclosure and poor purpose control.
3. **Purpose-bound separation with selective disclosure** — accepted because it preserves analytical utility while minimizing disclosure.

## Consequences

- API and persistence contracts require purpose/tenant/role/lifetime evidence for protected operations.
- Model/provider payloads are evidence-minimized and version/audit bound.
- Re-identification mappings are separately authorized and encrypted where feasible.
- Deletion/retention/legal-hold behavior becomes a product/release requirement.
- Derived relation/topic/factor data is not automatically classified as non-sensitive.

## Failure and recovery

An authorization/purpose decision that cannot be evaluated fails closed. If a provider or export would exceed the approved disclosure scope, TEPP abstains or requires explicit higher-authority approval. Privacy incidents preserve bounded forensic/audit evidence without copying unnecessary source text.

## Verification

Cross-tenant denial, expired-purpose denial, provider-payload minimization, log/source separation, export/re-identification authorization, retention/deletion, audit replay, and realistic multi-membership cases are required tests.

## Supersession

Supersede only if a later ADR demonstrates an equally usable privacy architecture that preserves the psychometric/temporal estimands and provides stronger verifiable controls.