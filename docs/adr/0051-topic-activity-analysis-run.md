# ADR 0051 — Topic activity/dormancy/reactivation as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0012 (P0 topic identity / activity) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already keeps a global P0 topic identity across activity,
dormancy, and reactivation inside `topic_lineage`. Operators still cannot
request that identity contract as a digest-bound analysis-run output.
Fitted same-topic sequence edges are a different profile
(`trsl_topic_lineage_v1`). Schwarz fitted candidate-`K` is a different
profile. Full Bayesian sampling, GPU, and topic birth/split/merge remain
later GAP-004 work and are not this slice.

Reactivation must not mint a new topic identity. An LLM label cannot define
topic identity.

## Decision

Add the `topic_activity_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes a durable `TopicIdentity` plus ordered activity transitions;
- requires the request snapshot and knowledge cutoff to match the offered
  construction;
- invokes `TopicLineageRecord` dormancy/reactivation, 
  `refuse_new_identity_on_reactivation`, and `identity_recovery_rate`
  without reimplementing those gates;
- emits a canonical SHA-256-digested `tepp.topic_activity.v1` artifact with
  the surviving identity, final activity, transition count, recovery rate,
  and inference status `reactivation_is_not_new_topic_not_birth_split_merge`;
- does not invent a Bayesian sampler, persist rows, split or merge topics,
  or emit fitted sequence edges.

This is activity-state identity, not birth/split/merge and not fitted
topic-lineage edges.

## Alternatives considered

1. Invent a Bayesian sampler or birth/split/merge engine — rejected because
   those functions do not exist on protected main and must not be faked.
2. Bind the Pareto `select_candidate_k` gate — rejected for this slice
   because it is a different model-selection surface from activity identity.
3. Bind GAP-013 interpreter/verifier — already live as a separate analysis-run
   profile.
4. Bind existing `topic_lineage` activity/dormancy/reactivation to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe topic activity as a digest-bound terminal
result. The artifact does not claim Bayesian sampling, GPU parity, or topic
birth/split/merge. Snapshot/profile/cutoff mismatch, illegal transitions,
reminted reactivation identities, and invalid recovery payloads fail closed.

## Verification

The PR includes Rust unit and integration tests for dormancy-then-reactivation
identity preservation, remint refusal, illegal transitions, minted-replacement
recovery rates, snapshot/profile/cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `topic_activity_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps reactivation
from minting a new topic identity and keeps activity distinct from
birth/split/merge.
