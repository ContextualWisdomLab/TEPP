# ADR 0017 — Consumer-scoped modular analysis-run ingress

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-20
**Supersedes:** None; narrows ADR 0011 for shared modular analysis-run ingress and leaves production TLS/deployment authority unchanged.

## Context

TEPP must operate standalone and as a modular CWL service. Its first live loopback analysis-run ingress admitted only `naruon`. LineageWeave already owns authorized source-post selection, lineage reconstruction, and Buyer navigation, and needs to submit a bounded TEPP analysis-run request without sharing application tables or forwarding browser, reviewer, model-provider, or database credentials.

Admitting another consumer by duplicating the listener would create divergent validation, idempotency, error, and security behavior. Reusing one tenant-scoped idempotency namespace without the consumer identity would also allow two legitimate products to collide when they independently choose the same tenant and caller key.

## Decision

TEPP publishes one consumer-neutral `/v1/analysis-runs` ingress and a closed modular-consumer registry. The initial admitted identities are:

- `naruon`;
- `lineageweave`.

The request body remains the versioned `AnalysisRunRequest`. The transport requires a matching `idempotency-key`, `tepp-contract-version`, `tepp-consumer`, JSON content type, and a loopback host in the current live proof. Accepted-run replay identity is scoped by:

```text
consumer_code + tenant_workspace_id + idempotency_key
```

A retry from the same consumer returns the original accepted run only when the complete validated request is semantically identical. The same tenant/key used by a different consumer has a separate namespace. A changed payload under the same consumer/tenant/key fails closed.

Consumer-specific client builders may set only the published consumer identity. They reuse the shared request validation and must not add credentials. The Naruon compatibility listener remains available while new consumers use `AnalysisRunLiveService`.

An HTTP `202 Accepted` response means only that TEPP accepted a durable analysis-run identity for later execution. It is not a completed temporal model, calibrated score, theta estimate, uncertainty statement, or scientific claim.

## Non-goals

- This ADR does not authorize direct access to another product's tables or object store.
- It does not define production TLS termination, public routing, service discovery, or tenant authentication; those remain separate deployment/security work.
- It does not make arbitrary consumer strings self-registering.
- It does not authorize a consumer to submit raw credentials, prompt text, provider secrets, or unrestricted PII.
- It does not define the completed-result contract.

## Alternatives considered

1. **One listener per consumer** — rejected because validation and security behavior would drift and every new CWL product would require another transport implementation.
2. **Tenant plus caller key only** — rejected because distinct modular consumers can legitimately reuse a key and must not replay or conflict with each other's accepted run.
3. **Trust any `tepp-consumer` value** — rejected because an open consumer namespace defeats purpose-bound admission and weakens auditability.
4. **Forward the caller's bearer token or provider credential** — rejected because TEPP should receive a bounded service contract, not inherit browser, reviewer, or model-provider authority.
5. **Closed consumer registry plus shared ingress and consumer-scoped idempotency** — accepted.

## Consequences

- LineageWeave and Naruon can use one validated analysis-run boundary without sharing databases.
- Adding another consumer requires a reviewed code change, contract tests, and an ADR/index update when the authority boundary changes.
- Idempotent retries remain deterministic within one product while cross-product collisions are prevented.
- The accepted acknowledgement remains operational evidence only and cannot be promoted to a measurement result.
- The shared listener carries a larger compatibility responsibility and therefore must preserve the strictest existing size, header, host, timeout, and error-redaction behavior.

## Failure and recovery

Unknown consumers, credential-bearing headers, malformed or duplicate headers, transfer encoding, non-loopback hosts, invalid content length, oversized payloads, unsupported contract versions, idempotency mismatches, and changed replay payloads fail closed with a redacted versioned error envelope.

Socket timeout or malformed I/O does not create an accepted run. A retry is safe when it reuses the same consumer, tenant, key, and semantically identical request. Recovery from a deployment outage replays the original bounded request; callers must not fabricate a succeeded run or infer that a missing acknowledgement means the computation failed after acceptance.

## Security, privacy, scientific-integrity, and governance impact

- No authorization, review, Copilot, NIM, OpenAI, database, or browser credential crosses the consumer boundary.
- The closed consumer registry is purpose-bound; consumer identity is included in replay/audit identity.
- Host validation, bounded header/body parsing, read/write deadlines, and content-redacting errors limit SSRF-style, request-smuggling, resource-exhaustion, and data-disclosure risks in the current loopback proof.
- Tenant/workspace and snapshot identities remain opaque service references.
- `202 Accepted` cannot be used as evidence of convergence, calibration, uncertainty, validity, or production release readiness.

## Compatibility and migration

The existing Naruon listener and `naruon_analysis_run_exchange` remain compatibility surfaces. LineageWeave uses `lineageweave_analysis_run_exchange`, which changes only the consumer header and preserves the shared payload contract. Existing Naruon idempotent retries retain their result within the new consumer-qualified namespace.

Production HTTP/TLS adapters may replace the loopback transport while preserving the same consumer registry, request semantics, credential prohibition, idempotency namespace, and redacted error contract. Consumer removal requires a deprecation window and retained historical audit interpretation.

## Verification

The falsifiable acceptance evidence is:

- LineageWeave receives HTTP `202` with a valid `AnalysisRunAccepted` response;
- Naruon remains accepted through the compatibility listener;
- same-consumer, semantically identical retries return the original run identity;
- Naruon and LineageWeave using the same tenant/key do not replay each other;
- a changed request under the same consumer/tenant/key is rejected;
- unpublished consumers are rejected;
- credential headers, non-loopback hosts, table-access-like hosts, malformed framing, unsupported versions, and oversized inputs are rejected;
- the LineageWeave exchange contains no credential header;
- formatting, Clippy with warnings denied, all-target Rust tests, public rustdoc, production line/branch coverage, documentation validation, and dependency policy pass on the exact PR head;
- independent current-head review remains required before merge.

## Rollback and supersession

Rollback returns callers to the last validated consumer-specific ingress while preserving accepted-run audit identities. It must not collapse existing consumer-qualified replay keys into a shared tenant/key namespace.

A superseding ADR is required to open dynamic consumer registration, change idempotency identity, permit credential delegation, remove the closed registry, or promote the accepted acknowledgement into a completed-result claim. Production TLS/deployment changes may complement this ADR but must preserve its authority and credential boundaries unless explicitly superseded.

## Related authority

- ADR 0011 owns standalone/modular CWL service and persistence boundaries.
- ADR 0002 owns knowledge-cutoff temporal eligibility.
- ADR 0008 owns immutable evidence and strict wire reconstruction.
- ADR 0009 owns purpose-bound PII governance.
- ADR 0014 owns scientific claim and release promotion.
