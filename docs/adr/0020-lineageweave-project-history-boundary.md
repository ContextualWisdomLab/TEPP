# ADR 0020 — LineageWeave project-history service boundary

**Decision status:** Accepted  
**Implementation maturity:** active-PR  
**Date:** 2026-08-21  
**Supersedes:** None; narrows ADR 0011 for the project-history projection.

## Context

LineageWeave owns authorization, source-post selection, and buyer navigation;
TEPP owns temporal eligibility and deterministic project-history projection.
The products need a versioned boundary that preserves this ownership split
without sharing application tables, provider credentials, or psychometric
claims.

## Decision

TEPP publishes the credential-free `POST /v1/project-histories` contract and
the `lineageweave_project_history_exchange` builder. LineageWeave supplies a
bounded, already-authorized set of explicit source events, an opaque tenant and
project identity, and a knowledge cutoff. TEPP validates the cutoff, orders
events deterministically, recomputes only explicit non-causal findings, and
returns a `temporal_association_only` projection. The contract is versioned,
strict JSON, bounded to 256 KiB, and contains no provider or caller
credentials.

## Non-goals

- TEPP does not authorize or discover LineageWeave source records.
- The projection is not a causal conclusion, psychometric score, theta,
  confidence value, or completed model result.
- The boundary does not grant cross-service database access or production TLS
  deployment authority.

## Alternatives considered

1. **Shared LineageWeave/TEPP tables** — rejected because it couples
   authorization, migrations, retention, and service ownership.
2. **A TEPP endpoint that fetches LineageWeave records by name** — rejected
   because authorization and evidence selection belong to LineageWeave.
3. **A credential-bearing provider request** — rejected because the boundary
   needs only an evidence contract, not browser, reviewer, or model authority.
4. **A versioned bounded evidence-in/projection-out contract** — accepted.

## Consequences

The services can run independently and compose through a stable API. Every
event and finding remains traceable to opaque submitted identities. Consumers
must reduce the authorized evidence bundle when the bounded response cannot be
represented; TEPP never silently truncates evidence or upgrades temporal order
to causation.

## Failure and recovery

Malformed, future-leaking, duplicate, oversized, credential-bearing, or
unsupported payloads fail closed with content-redacting errors. A retry may
reuse the same validated evidence and cutoff. An unavailable TEPP service does
not become a fabricated buyer result; the consumer records deferred/unavailable
state and retries through its own controlled adapter.

## Security, privacy, and scientific integrity

Only authorized bounded evidence and opaque identities cross the boundary.
Purpose-bound identity disclosure remains governed by ADR 0009. Deterministic
ordering and explicit finding recomputation preserve the distinction between
observed evidence, temporal association, and scientific inference.

## Verification

The project-history contract tests cover strict JSON, unknown fields, cutoff
leakage, deterministic ordering, finding recomputation, credential-free
headers, request/response size limits, and the near-limit generated-response
failure path. Documentation validation, Rust quality gates, and independent
current-head review remain required before merge.

## Rollback

Disable the modular adapter while retaining standalone TEPP operation. Do not
replace the contract with direct table access or reinterpret historical
projection records. A changed endpoint, ownership boundary, evidence meaning,
or credential policy requires a superseding ADR.

## Related authority

- ADR 0002 owns knowledge-cutoff and temporal eligibility.
- ADR 0008 owns strict bounded wire reconstruction.
- ADR 0009 owns purpose-bound PII governance.
- ADR 0011 owns standalone and modular service authority.
- ADR 0019 owns symmetric project-history wire-size enforcement.
- `docs/doctoring/LINEAGEWEAVE_PROJECT_HISTORY_REFERENCES.md` records the
  contract sources and APA 7th references.
