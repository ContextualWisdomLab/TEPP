# TEPP Technical Requirements Document

**Status:** Accepted technical baseline aligned to approved PRD v0.4  
**Last reviewed:** 2026-08-09

## 1. Technical objective

TEPP is a multilingual temporal-event psychometrics platform whose executable core is a set of independently usable Rust crates and versioned service/data contracts. It separates immutable evidence, six-clock temporal semantics, event/relation reasoning, shared-latent topic measurement, multilevel longitudinal psychometrics, compute backends, event intelligence, interpretation, and visual artifacts so scientific validity can be tested at each boundary.

## 2. Current implementation maturity

Protected main currently contains the Rust workspace foundation plus immutable evidence records, exact source spans, strict versioned evidence JSON, stable content-redacting errors, and repository quality contracts. PR #5 adds typed six-clock temporal values and uncertain intervals. PR #6, stacked on #5, adds Allen relation algebra and bounded path-consistency closure. Those PR capabilities remain active-PR until merged.

The remaining PRD architecture — event ontology, relation graph, multiple membership, PostgreSQL persistence, leakage-safe corpus splits, simulations, multilingual semantic units, topic measurement, GPU/VRAM compute, model selection, TDT/CHRONOS, longitudinal ESEM/DSEM, network analysis, interpretation, and visual analytics — is accepted-target, not as-built.

## 3. Immutable evidence requirements

Every source artifact and document must have immutable opaque identity distinct from SHA-256 content identity. Source bytes/text are bounded before allocation, owned immutably, and rehashed on reconstruction. Exact spans use half-open UTF-8 byte and Unicode-scalar coordinates plus optional page geometry; cross-document, mid-code-point, inconsistent, non-finite, out-of-page, reversed, or empty locations fail closed.

Evidence wire formats are explicitly versioned, reject unknown fields, and reconstruct through domain validation rather than bypassing invariants.

## 4. Temporal requirements

TEPP treats event/valid time, assertion time, document time, system time, available time, and knowledge cutoff as distinct nominal types. Analyses enforce `available_time <= knowledge_cutoff`. Exact, uncertain, open-ended, and unknown intervals preserve source precision and boundary semantics.

Forward state-transition/input→process→outcome edges must satisfy temporally valid partial order. Retrospective, revision, citation, translation, support, and contradiction edges may point backward as provenance but never create reverse state transitions.

PR #6's bounded Allen closure, if merged, establishes path consistency only within its stated algebra/limits; it must not be documented as a proof of global satisfiability for unrestricted disjunctive interval networks.

## 5. Event/relation/membership target

Event mentions and event instances are distinct evidence/latent objects. Relations are typed and provenance-bearing. Entity roles such as customer, partner, competitor, author, department, project, and opportunity pool are time-varying assignments rather than static entity types.

Observation/model structures support cross-classified and multiple-membership assignments with explicit weights and validity intervals. A document or segment may belong to multiple organizational/project/event contexts simultaneously.

## 6. Multilingual measurement target

All supported languages share global topic identities and latent coordinates while native lexical/morphological channels remain language-specific. Concept/semantic-unit mapping must be span-grounded and versioned. Unknown meaning is isolated rather than silently forced into a known concept.

Language support is a validation claim, not a feature flag: each language profile requires alignment/invariance/error evidence. Repeated template/style/report wording is modeled as method/background structure rather than removed through indiscriminate stopword/TF-IDF/BM25 heuristics.

## 7. Topic and psychometric target

Shared-latent temporal/relational topic estimation provides posterior uncertainty and covariate effects. Topic proportions are compositional; downstream correlation/ESEM uses logistic-normal coordinates or appropriate orthonormal log-ratio coordinates rather than naïve raw-proportion Pearson correlation.

Longitudinal ESEM/DSEM must distinguish stable between-unit differences from within-unit temporal change, test measurement invariance where comparisons require it, account for irregular intervals when necessary, and propagate topic-posterior uncertainty through plausible values or joint estimation.

## 8. Compute requirements

Production mathematical/psychometric arithmetic is Rust. CPU `f64` is the numerical reference. CPU parallelism uses bounded fixed worker pools/thread-local sufficient statistics to reduce context switching and oversubscription. GPU execution is introduced only when computationally material, streamed under a VRAM budget, and parity-tested against the CPU reference. OOM triggers bounded batch reduction and safe CPU fallback rather than uncontrolled failure.

## 9. Persistence and data-contract target

PostgreSQL is the reference relational store. Persistent objects use descriptive two-or-more-word `snake_case` names and explicit tenant, provenance, temporal validity/system-time, version, lifecycle, and audit dimensions where applicable. Exact physical tables/migrations are not implemented on current main; `docs/ERD.md` distinguishes current domain objects from planned persistence.

Exports/API artifacts are versioned and may use JSON Schema/JSON-LD, GraphML, Arrow/Parquet, and accessible SVG/PDF/tabular views where appropriate. Export formats must carry source/model/config/provenance hashes sufficient for reproducibility.

## 10. Leakage-safe data splits

Training/validation/test and rolling-origin evaluation must respect availability time and related-document/event lineage. Revision, translation, copied template, shared episode, or related-document variants cannot leak across partitions when doing so would inflate validation.

## 11. Scientific validation

Every estimator/reasoner has synthetic known-truth tests appropriate to its claim: parameter recovery, bias, RMSE, interval coverage, convergence, temporal ordering, graph/edge recovery, clustering recovery, invariance, alignment, calibration, and CPU/GPU parity. Monte Carlo acceptance accounts for simulation error rather than arbitrary point thresholds.

## 12. LLM boundary

Documents and LLM outputs are untrusted data. Live model tests use `NVIDIA_NIM_API_KEY`; `COPILOT_GITHUB_TOKEN` is prohibited. LLMs may assist semantic unitization, model review, interpretation, or verification only behind strict schemas/evidence bundles and cannot replace deterministic/statistical acceptance, mutate source evidence, execute document instructions, or gain merge/release authority.

## 13. Quality and release

Production Rust line and branch coverage are exactly 100%; public API rustdoc is complete; format/build/Clippy/tests/rustdoc/supply-chain/security gates are warning-free/current-head. Releases additionally require validated migrations/rollback, SBOM/provenance, reproducible artifacts, current protected-head review/security, CHANGELOG/version consistency, operational recovery, and no unresolved scientific blocker.