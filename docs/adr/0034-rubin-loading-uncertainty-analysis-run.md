# ADR 0034 — Rubin loading uncertainty as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Last reviewed:** 2026-09-14
**Supersedes:** None; complements ADR 0005 (ESEM/DSEM interpretation), ADR 0014 (scientific claim promotion), and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main owns two deliberately different `psychometric_core` contracts:
`recover_loading_point_estimate_mean` computes a scaled, compensated mean of
posterior-draw OLS loading point estimates, while
`combine_draw_level_ols_loadings` computes Rubin (1996) `Q̄`, `Ū`, `B`, and
`T = Ū + (1 + 1/m)B` for complete-data OLS loadings. The latter's ordinary
loading accumulation is not a substitute for the former's robust point
estimate under large cancellation.

Operators still cannot request the joint result as a historical, digest-bound
Analysis Run output. The original branch also left several application
contracts weaker than the profile name implied: row availability lacked an
immutable snapshot identity, request/executor cutoffs were compared as RFC
3339 text, draw/matrix materialization was unbounded, artifact counts could
claim unreachable executions, serialized `T` was not checked against its
components, and terminal provider validation reused the scientific inference
label.

A second scientific boundary remains after those application repairs. The
current input can carry any finite complete-data draw matrix; it does not yet
identify the generator/imputer contract or validation evidence for the exact
generator/analysis pairing. Rubin-style variance validity is therefore not
implied by the algebra alone. The profile may calculate the bounded component
arithmetic, but broader inferential activation must remain scoped to a
validated draw-generation design until that provenance contract exists.

## Decision

Add `rubin_loading_uncertainty_v1` to `analysis_engine` as an application
composition over the protected-main scientific owners. The executor:

- requires every `RubinLoadingObservation` to carry the requested immutable
  `snapshot_id` and typed `AvailableTime`;
- rejects cross-snapshot observations and excludes same-snapshot observations
  with `AvailableTime > KnowledgeCutoff` before matrix/scientific admission;
- parses request cutoffs and compares temporal instants, while persisted
  artifacts retain one canonical RFC 3339 cutoff representation;
- invokes `recover_loading_point_estimate_mean` for the robust point estimate
  and `combine_draw_level_ols_loadings` independently for Rubin `Q̄/Ū/B/T`;
- limits the current application representation to at most 256 complete-data
  draws and 1,000,000 admitted observation-by-draw cells before transposition.
  These are resource envelopes, not psychometric validity recommendations;
- bounds total raw observation population with `MAX_EVIDENCE_UNITS` and makes
  imported artifact counts obey the same reachable envelope;
- validates imported `T` by recomputing the exact binary64 expression used by
  the scientific owner. Canonical JSON round-tripping preserves the component
  values, so exact equality is the chosen wire-integrity policy rather than a
  tolerance that could admit a different scientific result;
- applies the 256 KiB artifact envelope to both untrusted `from_json` and
  canonical `to_json`, with a maximal-valid escaping proof for the output
  direction;
- propagates artifact/digest errors instead of asserting that accepted request
  identifiers make serialization infallible;
- emits terminal `AnalysisResultSummary.validation_status = "validated"` and
  keeps `rubin_combined_ols_loadings_not_mislevy_pv` solely as the artifact's
  scientific inference boundary;
- emits the digest-bound, read-only projection status
  `descriptive_only_unbound_draw_generation_provenance` whenever the draw
  generator is not bound to claim-specific Validation Evidence. Callers cannot
  mutate that status through the public artifact API.

This remains draw-level OLS combination. It is not person-level plausible-value
pooling, an ESEM/DSEM sampler, CWC, persistence, or a causal estimator.

The current profile does not authorize arbitrary supplied draw sets as
generally valid multiple-imputation inference. Issue #505 owns the remaining
positive activation contract: a versioned generator/analysis pairing, exact
Validation Evidence identity/digest, and historical availability/cutoff binding.
Draft #506 implements the fail-closed negative projection rule now, rather than
waiting for a future approved pairing and leaving provenance-free artifacts
semantically ambiguous. Until an approved pairing exists, `Q̄/Ū/B/T` may be
computed descriptively but cannot be projected as validated interval/variance
inference.

## Historical replay invariant

For a fixed requested snapshot and knowledge cutoff, adding evidence that only
becomes available after that cutoff must not change the earlier admitted
factor-score/draw matrix or its scientific result. Such rows may change only
the excluded-after-cutoff count. A row from another immutable snapshot is not
historical censoring; it is a provenance violation and fails closed even when
its availability is later than the cutoff.

## Alternatives considered

1. Use `combine_draw_level_ols_loadings.mean_loading` for both fields — rejected
   because it bypasses the protected robust point-estimate contract and can
   differ under large cancellation.
2. Compare cutoff strings — rejected because legal RFC 3339 representations of
   one instant must not alter historical execution.
3. Drop late rows without snapshot provenance — rejected because unrelated
   snapshot data could be silently attributed to the requested run.
4. Accept any finite nonnegative serialized `T` — rejected because a digest-
   bound uncertainty artifact must be internally consistent with its own
   components and draw count.
5. Leave draws unbounded and rely on `MAX_EVIDENCE_UNITS` — rejected because
   matrix materialization is a separate multiplicative resource dimension.
6. Put Rubin arithmetic into `analysis_engine` — rejected because reusable
   scientific arithmetic remains `psychometric_core`-owned.
7. Treat every finite complete-data draw matrix as inferentially validated —
   rejected because formula correctness does not establish proper-imputation
   behavior or compatibility between the draw generator and analysis contract.
8. Require a universal mathematical proof of congeniality for all future draw
   sources — rejected as neither realistic nor necessary. Bind each supported
   generator/analysis pairing to versioned provenance and claim-specific
   validation evidence instead.
9. Pre-authorize Draft #504's generator from a branch-local test result —
   rejected because its exact-head workflows and independent approval are not
   complete. Activation evidence cannot be promoted by the same branch that is
   still establishing it.

## Scientific acceptance boundary

Known-truth/noiseless fixtures and edge contracts are regression evidence, not
commercial scientific acceptance. Issue #503 owns repeated true-loading
recovery, bias/RMSE with Monte Carlo uncertainty, an explicitly justified
interval construction and empirical coverage, attempted/recovered/failed
denominators, design sensitivity, and leakage-safe historical evaluation. Draft
#504 provides candidate evidence for its explicitly declared repeated-sampling
generator only. Neither LLM judgment nor synthetic unit fixtures may promote a
broader scientific claim.

Repeated-sampling evidence must preserve the same identity semantics as the
product path it exercises. Each newly generated Monte Carlo population is a
new immutable snapshot and each executor call is a distinct Analysis Run. A
rolling-origin comparison may reuse one replication-specific snapshot because
the early and late views refer to the same generated population, but those
views still use distinct run/idempotency identities. Reusing one snapshot or
accepted-run receipt across different generated populations is invalid
evidence even when the resulting numerical summaries are deterministic.

Issue #505 separately owns positive inferential activation. The Analysis Run
must not project #504's design-specific coverage as universal authorization for
an unknown or arbitrary draw generator. Promotion requires a versioned approved
generator/analysis pairing and the Validation Evidence that supports that
pairing, consistent with ADR 0014's separation of implementation authority from
scientific/product claim authority. Until those conditions are met, the
artifact's read-only projection status remains descriptive-only and is included
in the canonical digest.

Primary authorities for the current combining-rule and activation boundary are:

Rubin, D. B. (1987). *Multiple Imputation for Nonresponse in Surveys*. Wiley.
https://doi.org/10.1002/9780470316696

Rubin, D. B. (1996). Multiple imputation after 18+ years. *Journal of the
American Statistical Association, 91*(434), 473–489.
https://doi.org/10.1080/01621459.1996.10476908

Meng, X.-L. (1994). Multiple-imputation inferences with uncongenial sources of
input. *Statistical Science, 9*(4), 538–558.
https://doi.org/10.1214/ss/1177010269

Xie, X., & Meng, X.-L. (2017). Dissecting multiple imputation from a multi-phase
inference perspective: What happens when God's, imputer's and analyst's models
are uncongenial? *Statistica Sinica, 27*(4), 1485–1545.
https://doi.org/10.5705/ss.2014.067

Repository research authority is `docs/research/rubin-total-variance.md` plus
`docs/research/rubin-loading-uncertainty-scientific-acceptance.md` for the
current profile-level repeated-sampling evidence.

## Consequences

The profile has a narrower, auditable temporal and resource boundary, and its
artifact can no longer claim a Rubin total inconsistent with its serialized
components. Consumers can distinguish provider validation from the scientific
claim boundary, distinguish the robust point estimate from Rubin `Q̄`, and now
also distinguish descriptive combination arithmetic from an authoritative
projection backed by draw-generation Validation Evidence. The checked-in #504
evidence can support only its declared generator/design after its own gates are
satisfied; #506 does not silently promote it. The profile remains
Draft/Proposed and not implemented-main while #503, #505, and the normal
exact-head merge gates remain unresolved.

## Verification

Required exact-head verification includes:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

Regression contracts cover equivalent cutoff instants, future-evidence replay,
cross-snapshot refusal, robust-point versus naive-mean cancellation, exact and
exceeded draw/resource bounds, inconsistent Rubin totals, artifact count
bounds, terminal provider/domain-status separation, Monte Carlo snapshot/run-
identity non-aliasing, and the digest-bound descriptive-only projection state
for provenance-free draw matrices. #505 still requires the positive executable
activation contract for missing/unknown/mismatched/stale versus explicitly
approved versioned generator/analysis evidence; that step must not duplicate the
Rubin arithmetic or promote Draft #504 prematurely.

## Rollback and supersession

Rollback removes the `rubin_loading_uncertainty_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that preserves the
scientific owner split, temporal provenance, resource admission, claim-specific
validation evidence, and the Rubin-versus-Mislevy / draw-generation activation
boundaries.
