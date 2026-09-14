# Exhaustive case-deletion analysis-run composition

**Active slice:** ADR 0056 / `case_deletion_refit_v1`
**Protected-main status:** not implemented-main
**ADR maturity:** Proposed

`analysis_engine` already owns the scientific operation that fits the complete corpus and every actual `D \ {i}` corpus through `fit_exhaustive_case_deletion`. This slice does not duplicate that fitter. It supplies the Analysis Run historical-admission boundary that the reusable fitter deliberately does not own.

The predecessor branch was not actually cutoff-safe. `CaseDeletionDocument` had no snapshot or availability provenance, every supplied document reached fitting, request/executor cutoffs were compared as RFC 3339 text, and terminal `validation_status` reused the domain inference label. The exhaustive runner also stores `n(n-1)` retained document identities across deletion results plus full/deletion posteriors, so a generic artifact byte cap or `MAX_EVIDENCE_UNITS` alone was not an adequate worker-resource bound.

Current repair lineage:

- RED `bc877e152fc9ad239de88f7050fd95854cdefab6` adds equivalent-cutoff, provider-status, and oversized-census/no-fitter-call contracts against the predecessor implementation.
- Source repair `eff6eafcec5d644d3414332bd6ce750c344652bd` adds aligned immutable snapshot IDs and `AvailableTime`, compares `KnowledgeCutoff::instant()`, rejects cross-snapshot rows, excludes same-snapshot future-unavailable rows before scientific admission, keeps visible duplicates fail-closed through the protected-main runner, and separates terminal `validated` from artifact inference.
- The same repair caps current exhaustive representation at 65,280 retained document identities (`256 * 255`), so at most 256 admitted documents and 257 full/deletion fitter invocations are allowed. An oversized admitted census is rejected before any fitter call.
- `6d0fb9081334e30dbe062aa11bc594eed801f3e7` promotes canonical `corpus_split::cutoff_eligible` to a runtime dependency.
- `4a85e41519bf806334d2d4f41cde9938d9cd8d6c` migrates the integration contract and proves future-duplicate replay invariance, cross-snapshot/misaligned-provenance refusal, visible-duplicate refusal, equivalent cutoff instants, and zero fitter calls at 257 admitted documents.
- `7705111516eda65b38e4cf7d8d3069f786bab65d` bounds every cutoff-visible document identity before the runner's quadratic identity cloning and prevalidates the derived full-fit seed domain so malformed/oversized identity state cannot consume the full refit budget before failing artifact construction.
- ADR repairs `816917b5089320595f6dcf97db88f8929225a8b0` and `8f2aca3ef9afd5d9ed573e9b15197ccb3b75f50f` record the temporal/resource decision and return ADR 0056 from premature `Accepted` authority to `Proposed`.

Historical replay invariant: evidence from the requested snapshot whose `AvailableTime` is after the requested cutoff cannot alter the earlier admitted corpus, artifact, or terminal result. Cross-snapshot evidence is a provenance violation and fails closed rather than being silently censored.

The resource ceiling is representation-specific, not a scientific claim that case-deletion analysis is intrinsically limited to 256 documents. The profile now bounds retained identity count and each visible identity's bytes and caps the number of fitter-owned posteriors. It still cannot make a truthful generic byte-size claim for arbitrary scientific evidence `D` or posterior `P`. Issue #499 owns that remaining production fitter/posterior resource-envelope gap; a larger ceiling or complete worker-memory/SLO claim requires a concrete fitter-owned resource contract or a runner representation change with exact measurement.

Raw posteriors remain with the scientific fitter. Reweighting, a fixed posterior, or a diagonal approximation is not an acceptable substitute for an actual `D \ {i}` fit. This profile is not a Bayesian sampler, GPU execution, or topic birth/split/merge.

The large shared `docs/adr/README.md`, `docs/TRACEABILITY.md`, and product-gap baseline remain consolidation surfaces. ADR 0056 must not be advertised as `Accepted` there before protected merge; shared current-state edits belong to the canonical documentation/consolidation lane rather than being treated as complete by this doctoring note.

Merge authority is the unchanged surviving exact head after conflict-resolving consolidation. Live organization ruleset 18156473 requires one qualifying current-head approval, dismisses stale approvals after pushes, requires all review threads resolved, and enforces organization required workflows. Thread resolution or predecessor checks are not approval/merge evidence. Exact-head owned-production line/branch coverage, Rust/documentation/security/CodeQL gates, and the qualifying independent approval must be reacquired after every head or base change.
