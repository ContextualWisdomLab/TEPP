# ADR 0069 — Membership-target refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0003 (language, episode, template, department, and opportunity-pool memberships are typed targets, not entity/project columns) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0068 (topic-context posterior), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0063 (lineage-criterion fitting), or ADR 0058 (copy-identity / template-copy).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to collapse one membership-target kind into
another via `membership_target::MembershipTargetKind` and
`refuse_collapsed_target`. Persistence currently stores only an entity or
a project. Operators still cannot request that typed-target census as a
digest-bound analysis-run output.

Location-membership (#430 / ADR 0066) refuses location as entity identity
or as a language channel and does not bind `membership_target`.
Membership-posterior ICC (#398) binds a psychometric ICC estimator.
Copied-text (#427 / ADR 0065) binds unique-content/stopword vocabulary.
Copy-identity (#416 / ADR 0058) binds template-copy identity.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `membership_target_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `MembershipTargetDocument` rows with closed
  `MembershipTargetKind` values;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- invokes `refuse_collapsed_target` without reimplementing the
  language/episode/template/department/opportunity-pool/entity/project
  vocabulary;
- requires at least one typed non-entity/project kind and at least one
  entity or project treatment so the census is mixed;
- emits a canonical SHA-256-digested `tepp.membership_target.v1` artifact
  with per-kind counts, matching refusal counts, and inference status
  `language_episode_template_department_opportunity_pool_are_not_entities`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate location-membership (#430 / ADR 0066) — rejected because
   that profile binds `location_membership` LocationKind refusals, not
   `MembershipTargetKind`.
2. Duplicate membership-posterior ICC (#398) — rejected because that
   profile binds a psychometric ICC estimator and does not bind
   `membership_target`.
3. Duplicate copied-text (#427) or copy-identity (#416) — rejected
   because those profiles bind residue/template identity, not typed
   membership-target kinds.
4. Put `identity_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
5. Bind the existing membership-target refusals to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe membership-target refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, location-membership, membership-posterior ICC, copied-text,
copy-identity, citation-edge, corpus-background, method-effect
estimation, or topic birth/split/merge. Snapshot/profile/cutoff
mismatch, empty or single-class corpora, and duplicate document
identities fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed
language/episode/template/department/opportunity-pool/entity/project
corpora, empty/single-class/duplicate refusal, snapshot / profile /
cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `membership_target_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that keeps
language, episode, template, department, and opportunity-pool targets
distinct from entity/project columns and from `identity_recovery_rate`
inspect metrics.
