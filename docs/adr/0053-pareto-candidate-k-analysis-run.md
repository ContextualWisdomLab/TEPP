# ADR 0053 — Pareto candidate-`K` selection as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0012 (candidate-`K` / Pareto gates) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already admits a unique `K` from a Pareto-filtered statistical
front inside `model_selection::select_candidate_k` and scores selected-`K`
RMSE against known truth. Operators still cannot request that gate as a
digest-bound analysis-run output. Schwarz fitted candidate-`K` selection is a
different profile. Joint Gauss-Newton Laplace draws are a different profile.
Topic activity/dormancy is a different profile. Full Bayesian sampling, GPU,
and topic birth/split/merge remain later GAP-004 work and are not this slice.

The original branch treated already-aggregated candidate diagnostics as
"cutoff-safe" without carrying the evidence availability that produced them.
That permitted diagnostics derived from post-cutoff evidence to enter a
historical run. It also compared RFC 3339 cutoff text rather than temporal
instants, passed candidate count as terminal evidence count, and admitted an
unbounded candidate population into the quadratic Pareto-dominance scan.

## Decision

Add the `pareto_candidate_k_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes the existing `ModelCandidate` values and
  `selected_k_root_mean_square_error` contract without reimplementing Pareto
  dominance or RMSE;
- requires `ParetoCandidateKInput` to carry the immutable source snapshot,
  typed `KnowledgeCutoff`, and the complete `AvailableTime` vector for the
  evidence universe used to construct both the diagnostics and selected-`K`
  replications;
- rejects construction if any source evidence became available after that
  cutoff. The engine does not attempt to subtract future rows from diagnostics
  that were already aggregated by another owner;
- binds request, executor and input cutoffs by `KnowledgeCutoff::instant()` so
  equivalent legal RFC 3339 spellings represent one instant;
- bounds source evidence and selected replications by
  `MAX_EVIDENCE_UNITS`, and bounds the quadratic Pareto candidate set to 256
  before `select_candidate_k` executes. The 256 ceiling is an application-path
  CPU bound (at most 65,536 ordered candidate comparisons), not a scientific
  assertion about admissible `K`;
- reports the source evidence-universe cardinality as
  `AnalysisResultSummary.evidence_count`; candidate/statistical counts remain
  artifact fields;
- keeps provider validation status (`validated`) separate from the artifact
  inference claim `pareto_statistical_front_not_fitted_schwarz_sampler`;
- exposes completed artifact fields read-only through accessors. Untrusted
  artifact construction continues through bounded `from_json` validation;
- refuses LLM-vote-only authority and empty candidate sets; and
- does not invent a Bayesian sampler, persist rows, select GPU backends, or
  emit topic-lineage edges.

The profile therefore proves admission and claim boundaries around an existing
statistical gate. It does not itself establish scientific recovery quality.
Issue #500 owns the missing profile-level recovery evidence: true-`K` recovery,
selected-`K` RMSE/bias with Monte Carlo uncertainty, convergence/failure
denominators, stability across the declared design, and leakage-safe temporal
evaluation where applicable.

## Alternatives considered

1. Trust the caller's statement that candidate diagnostics are cutoff-safe —
   rejected because the original API carried no evidence with which to verify
   that claim.
2. Drop post-cutoff rows after model diagnostics have already been aggregated —
   rejected because candidate likelihood/complexity diagnostics cannot be
   causally repaired by subtracting metadata after fitting.
3. Compare cutoff strings exactly — rejected because legal RFC 3339 strings can
   encode the same instant with different offsets.
4. Leave candidate population unbounded — rejected because
   `select_candidate_k` performs a quadratic dominance scan.
5. Bind another Schwarz `select_fitted_candidate_k` profile — rejected because
   that bind is already a separate analysis-run profile.
6. Invent a Bayesian sampler or topic birth/split/merge engine — rejected
   because those functions do not exist on protected main.

## Consequences

A successful artifact is bound to evidence that was already available at the
requested historical cutoff, and the terminal summary no longer mistakes
model candidates for source evidence. Future evidence fails at input
construction rather than contaminating a historical model-selection result.
Cross-snapshot or cutoff-rebound input fails closed at execution.

The explicit 256-candidate ceiling constrains current O(n²) request cost. If a
buyer path requires a larger candidate universe, the owner must first replace
or profile the algorithm/representation and establish a new resource contract;
the ceiling must not be raised merely to pass a fixture.

Scientific acceptance remains open under #500. This ADR is `Proposed` while the
implementation is confined to an unmerged Draft branch; it does not authorize
an `Accepted` or implemented-main claim.

## Verification

The PR includes contracts for:

- equivalent RFC 3339 cutoff spellings;
- post-cutoff source-evidence refusal before Pareto selection;
- snapshot/cutoff provenance binding;
- the 256-candidate and `MAX_EVIDENCE_UNITS` replication bounds;
- source evidence count distinct from candidate count;
- provider-validation/domain-inference separation;
- LLM-vote non-authority and empty candidate sets;
- positive selected-`K` RMSE and artifact tampering; and
- immutable public artifact access through validated read-only accessors.

Run on the exact surviving head:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

These contracts do not replace #500 scientific recovery evidence or required
hosted security, CodeQL, coverage and independent-review gates.

## Rollback and supersession

Rollback removes the `pareto_candidate_k_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps Pareto
statistical selection distinct from LLM votes, Schwarz fitted selection,
joint Laplace draws, and Bayesian sampling while preserving leakage-safe
provenance and an explicit resource envelope.
