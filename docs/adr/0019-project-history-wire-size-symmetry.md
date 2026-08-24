# ADR 0019 — Symmetric LineageWeave wire-size enforcement

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-21
**Supersedes:** None; narrows ADR 0008 for the project-history and temporal-context DTO boundaries.

## Context

The LineageWeave project-history and temporal-context request/response pairs
use symmetric wire-size ceilings when parsing JSON. A request can be valid and
close to its ceiling while its deterministic projection adds response
metadata. Without output guards, TEPP can construct a response that its own
parser rejects, leaving callers with an internally inconsistent success path.

## Decision

`ProjectHistoryRequest::to_json` and `ProjectHistoryProjection::to_json` both
enforce `DEFAULT_PROJECT_HISTORY_BYTE_LIMIT`. The
`project_history_projection` builder serializes and validates the generated
projection before returning it. A projection that cannot be represented by the
published wire contract fails closed with `ApiError::LimitExceeded`.

`TemporalContextRequest::to_json` and `TemporalContextResponse::to_json`
likewise enforce `DEFAULT_TEMPORAL_CONTEXT_BYTE_LIMIT`. The live adapter cannot
return a success body that the published response parser rejects.

## Alternatives considered

1. **Only increase the response limit** — rejected because it silently changes
   the published boundary and allows asymmetric resource consumption.
2. **Reserve an undocumented request headroom** — rejected because the
   request-to-response size delta depends on event content and findings.
3. **Guard only the HTTP adapter** — rejected because callers can use the
   standalone DTO builder and bypass that adapter.
4. **Validate every serialized request and generated projection at the shared
   DTO boundary** — accepted.

## Consequences and failure recovery

Valid small projections are unchanged. Near-limit requests that would produce
an oversized response now fail deterministically before a success is exposed;
the caller can submit a smaller authorized evidence bundle. No event is
silently dropped and no truncation is introduced.

## Security, privacy, and scientific integrity

The shared bound limits memory and transport amplification without exposing
payload contents in errors. The complete explicit evidence set remains the
scientific input; rejecting an unrepresentable projection is safer than
silently changing temporal associations or findings.

## Verification

Contract tests construct project-history and temporal-context payloads whose
serialized forms exceed their response ceilings and assert `LimitExceeded`.
Existing round-trip, unknown-field, cutoff, ordering, and finding-invariant
tests remain required.

## Rollback

Rollback requires a superseding ADR because removing the guard would
reintroduce a self-rejecting success path.

## Related authority

- ADR 0008 owns strict versioned wire reconstruction and bounded evidence.
- ADR 0011 owns standalone and modular service boundaries.
- `docs/doctoring/LINEAGEWEAVE_PROJECT_HISTORY_REFERENCES.md` records the
  standards and APA 7th sources for this contract.
