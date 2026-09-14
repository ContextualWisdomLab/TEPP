# Pareto candidate-`K` analysis-run composition

**Active slice:** ADR 0053 / `pareto_candidate_k_v1`
**Protected-main status:** active PR only; not implemented-main

`model_selection` owns Pareto selection and selected-`K` RMSE arithmetic. This
slice binds that existing gate to the Analysis Run boundary without copying the
numerical algorithm.

The original profile was not actually leakage-safe: candidate diagnostics had
no source snapshot or availability provenance, request/executor cutoffs were
compared as RFC 3339 text, candidate count was reported as evidence count, and
the quadratic Pareto scan had no application-path population ceiling.

Current branch repair lineage:

- RED `34ad6d4072a5af5136f91471d7d08b9b28f38ef9` requires explicit source
  snapshot/cutoff/availability provenance, rejection of post-cutoff evidence,
  equivalent-instant cutoff binding, source evidence count distinct from model
  candidate count, and pre-selection population limits;
- repair `c3173908a3ac90049886e0fd198564001404ace5` binds the diagnostics to the
  complete source-evidence availability vector, validates the binding again at
  execution, limits the O(n²) candidate scan to 256 candidates, bounds
  replications by `MAX_EVIDENCE_UNITS`, compares cutoff instants, reports the
  source-evidence denominator, and separates terminal `validated` status from
  the domain inference claim;
- RED `1b1bc4b26a02fac85a859ebe6fce891a79291a23` requires external consumers to
  use read-only artifact accessors rather than mutating public fields;
- repair `3cc15b76ab4f359850d3658349366d7cb2ef5af0` makes completed artifact fields
  private while retaining validated serde parsing and explicit accessors;
- ADR repair `1cd3bdf49a02dec84e7d4986d627394b3e24732d` returns ADR 0053 from premature
  `Accepted` authority to `Proposed` and records the temporal/resource/claim
  boundaries;
- doctoring repair `4f4c84500a0676c9edd8a84f219585ed29086e88` aligns this profile description
  with those boundaries;
- test-only `03a2a9bbe2225b6110d0f1f715332cb01493fbfd` exercises invalid/empty source
  provenance, exact and +1 source-evidence/replication/candidate ceilings, and
  input-cutoff revalidation; and
- repair `71086e9eacff6e1a4db025a73132d497902027ff` proves a maximal valid artifact
  with worst-case JSON-escaped 256-byte identifiers remains below the 256 KiB
  input wire limit, keeps the untrusted `from_json` limit, and removes only the
  unreachable post-validation egress-size branch.

Historical admission is intentionally conservative. The input represents the
complete evidence universe that produced its candidate diagnostics and
selected-`K` replications. If any contributing evidence has
`AvailableTime > KnowledgeCutoff`, construction fails closed. The engine does
not pretend it can subtract a future row from likelihood/complexity diagnostics
that were already fitted elsewhere.

The 256-candidate ceiling is an operational bound on the current quadratic
selection path, not a scientific restriction on valid topic count. LLM votes
remain non-authoritative for the numerical optimum.

Profile-level scientific acceptance is still open. Issue #500 requires
true-`K` recovery/selection frequency, selected-`K` RMSE and bias with Monte
Carlo uncertainty, explicit attempted/recovered/failed denominators, stability
across the declared design, and leakage-safe temporal evaluation where the
buyer path is longitudinal. Unit fixtures and deterministic RMSE examples do
not satisfy that evidence obligation.

Shared `docs/TRACEABILITY.md`, `docs/product-technical-gap-baseline.md`, and the
ADR index are consolidation surfaces owned by the #435 documentation lane. A
PR body or comment is handoff evidence, not checked-in current-state authority.

The profile stays Draft until its valid source/tests/ADR/doctoring delta is
inherited by the surviving Analysis Run vehicle, exact-head Rust/coverage/
security/CodeQL/documentation checks are current, #500's scientific acceptance
boundary is respected, all valid review findings are resolved, and the live
ruleset's qualifying current-head approval is present. It is not Schwarz fitted
selection, not joint Gauss-Newton Laplace draws, not a Bayesian sampler, not GPU
execution, and not topic birth/split/merge.
