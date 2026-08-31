# ADR 0030 — Analysis-run wait loopback CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0029 for the operator-visible wait client. Does not supersede ADR 0014 claim-promotion authority. This ADR number is unique on the GET-status lineage; other live PRs may reuse 0030 on unrelated stacks (scientific-acceptance loopback CLI).

## Context

ADR 0029 lets operators inspect one status GET, but they still had to write a poll loop to learn when an accepted or running run became succeeded or failed. Duplicating status CLI (#392), GET-by-id HTTP (#359), lifecycle POST (#360), cancel/create/retry/lookup/retry-lineage CLIs, or Leiden would collide with live PRs.

## Decision

`tepp_api` publishes a loopback-only `tepp-analysis-runs wait` verb on this GET-status lineage:

- `wait` polls `GET /v1/analysis-runs/{run_id}` with `--run-id` and `--idempotency-key` until `succeeded` or `failed`, or until `--timeout-ms` elapses.
- Default timeout is 1000 ms (max 60000). Default poll interval is 10 ms (max 1000). Interval longer than a nonzero timeout fails closed. Timeout `0` polls once and fails closed if the run is still accepted or running.
- Stdout reuses ADR 0029 gates. Accepted/running/failed stay metric-free. `tepp.scientific_acceptance.v1` appears only on succeeded `scientific_acceptance_v1`.
- Non-loopback hosts, unpublished consumers, credential-shaped flags, nonempty stdin, unknown verbs, and oversized budgets fail closed.
- This slice does not implement GET-by-id HTTP or lifecycle POST.
- Persistence, Compose recovery, and psychometric execution remain GAP-003B.

## Alternatives considered

1. **Keep status GET as the only client** — rejected because operators still write poll loops after ADR 0029.
2. **Add `wait` onto retry-lineage CLI (#403)** — rejected because that head already owns `tepp-retry-lineage`.
3. **Busy-wait without a timeout** — rejected because a hung accepted run must fail closed.
4. **Loopback wait CLI stacked on ADR 0029** — accepted.

## Consequences

- Operators can wait for terminal status without writing HTTP or a poll loop.
- Wait stdout cannot treat accepted/running as measurement evidence.
- CLI success is not release evidence.

## Failure and recovery

Non-loopback hosts return authorization denied. Unknown verbs, nonempty stdin, unpublished consumers, credential flags, and oversized budgets fail closed. An accepted or running run past `--timeout-ms` returns limit exceeded. Unknown identities remain refused by ADR 0027.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- The CLI remains loopback-only and time-bounded.
- Process exit 0 on wait is not an ADR 0014 claim.

## Compatibility and migration

Status GET HTTP, status CLI, create POST, temporal-context, and project-history paths are unchanged. Parallel `tepp-analysis-runs` verbs on other stacks merge by combining verbs.

## Verification

Falsifiable evidence:

- wait of a failed run returns metric-free failed status without `tepp.scientific_acceptance.v1`;
- wait of an accepted run with `--timeout-ms 0` fails closed;
- non-loopback host, credential flags, nonempty stdin, and unknown verbs fail closed;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review remain required.

## Rollback and supersession

Rollback removes the wait verb; status GET and status CLI remain valid. A superseding ADR is required to persist the registry, bind a public address, or treat wait success as an ADR 0014 claim.

## Related authority

- ADR 0029 owns loopback status CLI.
- ADR 0027 owns loopback GET-by-id status.
- ADR 0018 owns consumer-scoped ingress.
- ADR 0014 owns scientific claim promotion.
- RFC 9110 owns GET semantics (Fielding, Nottingham, & Reschke, 2022). It does not authorize scientific claims.
