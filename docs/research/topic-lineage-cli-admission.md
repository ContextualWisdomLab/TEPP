# Offline topic-lineage admission: dense state and evidence status

**Maturity:** active-PR repair; offline CPU evidence only, not protected-main,
posterior calibration, GPU parity, production coverage, or release acceptance.

## Scientific authority before refusal

[PRD v0.4](../product/prd-v0.4-approved.md) §3 preserves observed relations
and provenance/confidence for inferred fields. [ADR 0012](../adr/0012-temporal-relational-shared-latent-topic-measurement.md)
explicitly restricts its harmonic network penalty to observed predecessor/successor
relations. The primary mathematical source for that network regularization
estimand is Mei et al. (2008); it does not authorize a heuristic/LLM proposal to
become an observed edge. The typed owning boundary
`relation_graph/src/provenance.rs` independently distinguishes `Observed` from
unpromoted `Inferred`.

The repair enforces that existing contract in `collect_transition_pairs`.
Inferred proposals stay validated graph provenance, never enter the objective,
relation gradient, degrees, curvature, connected counts, or sequence edges.
No eligible observed transition pair means `InvalidModelInput`, surfaced by the
CLI as `analysis_engine_rejected`, exit 3, no result. Observed plus inferred
parallel edges cannot double-weight an observed pair. Cutoff, endpoint,
interval, reverse-transition, duplicate, and cycle checks still apply to proposals;
this change does not relax graph admission or provide a promotion mechanism.

This corrects a source defect, not the approved estimand. PRD v0.4 and wire
schema/model identities remain unchanged. The PRD's posterior-aware shared-latent
measurement target remains in force; this offline CLI result is not posterior
acceptance or identified causation. ADR 0012 records the explicit correction.

## Dense allocation trace

Sparse term counts and bounded work do not bound dense state. A vocabulary of
100,000 and 49,998 topics with four nonzeros, one seed, and two iterations
passes the old sparse-work gate while one `beta` alone requires
39,998,400,000 bytes. Tests call admission only; they never allocate that matrix
or claim to have observed an OOM.

Let D be document rows, V vocabulary columns, K topics, and F the admitted
prevalence features (intercept, time, distinct active `(role, group_id)` keys).
The inspected CPU fitting path in `topic_measurement/src/reference.rs` has:

| Stage | Simultaneously retained dense payload |
|---|---|
| `fit_reference_topic_model` | Best converged seed's beta K×V, eta D×(K−1), coefficients F×(K−1) while fitting another seed |
| `fit_seed` | Current beta K×V, eta D×(K−1), coefficients F×(K−1), shared design D×F |
| `expectation` | Current and best state above, theta D×K, document counts D×K, beta_counts K×V |
| `prevalence_means` / objective | Above plus means D×(K−1) |
| `update_beta` | Consumes/normalizes beta_counts in place; assignment replaces old beta, no fourth K×V matrix |
| `update_eta` | Retained best/current state, theta, counts, means, gradient D×(K−1), one edge delta K; beta_counts has been consumed |
| `build_result` | Selected beta/coefficient state is moved, eta remains while theta/variance rows are formed; no new K×V matrix |

A conservative envelope (not an exact allocator high-water measurement) is:

`B = 8 * [3*K*V + 6*D*K + 2*F*(K−1) + D*F + K] <= 128 MiB`.

Three beta-sized buffers reserve best, current and expectation counts even for
one seed. Six D×K dominate the two eta arrays, theta, counts, means and relation
gradient. Coefficients reserve both best/current; design and edge delta are
separate. Seed execution is serial: do not multiply retained matrices by seed
count. Existing seed/iteration work caps still apply.

`ReferenceTopicInput::new`/`build_design` checks `8*D*F <= 16 MiB`
with checked arithmetic before feature expansion and dense design allocation.
F includes intercept, time, sparse covariate columns through the highest
populated index, and active `(role, group_id)` contexts; empty declared columns
are not newly activated. Feature sums and high sparse indices fail closed on
overflow. `fit_reference_topic_model` checks the full shared 128 MiB envelope
with actual D,V,K,F before entering `fit_seed`. It returns the typed
`TopicMeasurementError::ResourceLimitExceeded`, never a partial fit.

The CLI reuses these shared planners. `check_limits` retains its conservative
F=0 check on offered D before admission, then `check_dense_design_limits`
checks admitted D and active F. Existing stricter count/work caps and the
16 MiB operator design cap remain unchanged. All products/sums are checked;
arithmetic overflow fails closed.

128 MiB is a shared payload admission policy, not a published scientific
threshold, available-memory detector, hard RSS limit, or OOM recovery guarantee.
It excludes Vec headers, feature metadata, allocator overhead,
input/JSON/sparse/graph structures, artifact serialization and the separately
bounded joint precision/Cholesky path (4096-coordinate cap unchanged).
The previous CLI-only statement that standalone callers own dense admission
is superseded: direct library and executor consumers receive the shared typed
refusal; model-selection treats resource-refused candidates as failed fits,
returning `NoSuccessfulFit` if none succeed. No elapsed-time or deadline
guarantee follows from a work or payload cap.

## Verification scope

Dimension-only tests cover exact 128 MiB, one vocabulary/document/topic/feature
increment, arithmetic overflow and small positive input. A small sparse
high-K/high-V witness establishes preallocation refusal; the many-context
fixture separately exercises F×K refusal before reference construction.
CLI tests cover inferred-only and late-observed-only failure, mixed-edge
artifact/terminal equivalence to deleting proposals, observed positives and
invalid inferred temporal edges. The native reference test compares complete
observed-only and observed-plus-proposals fits exactly. These regressions
establish admission semantics, not new scientific recovery
acceptance. Shared dimension-only tests add exact 16 MiB design, one-column
excess, actual high sparse covariate indices, feature-count overflow, independent
beta/document/coefficient state witnesses and actual direct fit refusal. Direct
executor tests require the nested resource error and no artifact. Actual
candidate-K tests require all-refused `NoSuccessfulFit` and a mixed candidate
set where the admissible statistical fit wins despite an oversized LLM vote.
Direct sparse positives preserve unused declared covariate columns and existing
known-truth recovery, observed/mixed graph equivalence and precision tests.
The successor native gate scope explicitly includes `model_selection` as well
as `analysis_engine`, `topic_measurement`, and `tepp_api`; earlier three-package
results did not include model-selection. No oversized dense witness is allocated.

## Reference

Mei, Q., Cai, D., Zhang, D., & Zhai, C. (2008). Topic modeling with network
regularization. In *Proceedings of the 17th International Conference on World
Wide Web* (pp. 101–110). Association for Computing Machinery.
https://doi.org/10.1145/1367497.1367512
