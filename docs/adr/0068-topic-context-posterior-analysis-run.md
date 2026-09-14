# ADR 0068 — Posterior topic-context producer as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0022 (cutoff-safe analysis-run execution) and ADR 0024 (posterior topic-context producer contract).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already validates digest-bound posterior topic-context
artifacts inside `analysis_engine::TopicContextPosteriorArtifact`. The
producer contract keeps full-rank logistic-normal coordinates, refuses
collapsed missing draws, and labels the claim boundary
`posterior_topic_coordinates_not_importance`. Operators still cannot
request that validator as a cutoff-safe analysis-run output.

Independent TDT link-criterion fitting, location-membership refusals,
copied-text residue refusals, provenance-is-not-transition refusals, and
composed fitted-lineage remain different profiles. Full Bayesian sampling,
GPU, and invented topic birth/split/merge remain later GAP-004 work and
are not this slice. ADR 0064 through ADR 0067 are already taken by live
sibling PRs.

## Decision

Add the `topic_context_posterior_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes an already-constructed `TopicContextPosteriorArtifact`;
- requires an authoritative snapshot manifest to bind the request and artifact
  snapshot identity, source digest, cutoff, exact artifact digest, and every
  represented document's availability time;
- compares request, artifact, and manifest knowledge cutoffs by temporal instant
  while requiring canonical RFC 3339 for persisted artifact/manifest evidence;
- validates and serializes the snapshot manifest through the same bounded 16 MiB
  wire envelope used by the posterior artifact rather than accepting an
  unbounded in-memory side contract;
- rejects missing, extra, malformed, or post-cutoff document availability and
  artifacts not emitted under the approved `trsl-tm-v1` producer contract;
- invokes the existing producer `sha256`/validate path without
  reimplementing TRSL-TM fitting;
- emits a digest-bound terminal result under
  `tepp.topic_context_posterior.v1`; provider validation remains `validated`,
  while the artifact retains the scientific inference boundary
  `posterior_topic_coordinates_not_importance`;
- refuses reuse of `lineage_criterion_v1`, `case_deletion_refit_v1`,
  `composed_fitted_lineage_v1`, `fitted_candidate_k_v1`,
  `trsl_topic_lineage_v1`, and `method_effects_v1` as this profile;
- does not invent a Bayesian sampler, persist rows, select GPU backends,
  infer topic importance, or emit invented birth/split/merge events.
  Lineage events remain producer-supplied.

Supporting lineage/relation/membership evidence-resource availability is not yet
represented by the current manifest shape. The profile therefore remains
Proposed: it must not be promoted to implemented-main or scientific acceptance
until those support resources are independently cutoff-bound instead of relying
only on represented-document availability.

This is posterior topic coordinates, not importance and not a sampler.

## Alternatives considered

1. Bind another refusal or lineage-criterion profile — rejected because
   those binds are already live as separate analysis-run profiles.
2. Invent a Bayesian sampler or topic birth/split/merge engine — rejected
   because those functions do not exist on protected main as executors.
3. Collapse missing draws into a point estimate — rejected because the
   producer contract already fails closed on incomplete draw sets.
4. Compare RFC 3339 cutoff strings byte-for-byte — rejected because equivalent
   offset representations can denote the same instant.
5. Treat artifact inference language as terminal provider validation — rejected
   because scientific interpretation and execution validation are distinct claims.
6. Bind the existing producer validator to ADR 0022's analysis-run
   profile — selected, subject to the remaining support-evidence availability gate.

## Consequences

Operators can exercise the draft profile with instant-safe cutoff comparison and
a bounded snapshot-manifest wire contract. The artifact does not claim topic
importance, Bayesian sampling, GPU parity, or invented birth/split/merge.
Snapshot/profile/cutoff/digest mismatch, incomplete document cutoff eligibility,
and producer-contract refusal fail closed. Support evidence-resource availability
remains an explicit merge blocker rather than being silently treated as document
availability.

## Verification

The PR includes Rust integration tests for equivalent cutoff instants, terminal
validation/inference separation, bounded manifest round-trip and rejection,
successful digest-bound coordinates, incomplete draw refusal, run-identity
mismatch, snapshot/profile/source/artifact-digest mismatch, future document
evidence, producer-contract mismatch, and reuse of live sibling profiles. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `topic_context_posterior_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that keeps
posterior coordinates distinct from importance, sampling, and invented
lineage events and preserves leakage-safe support-evidence admission.
