# TEPP Technical Requirements Document

**Status:** Accepted technical baseline aligned to approved PRD v0.4  
**Last reviewed:** 2026-08-16

## 1. Technical objective

TEPP is a multilingual temporal-event psychometrics platform whose executable core is a set of independently usable Rust crates and versioned service/data contracts. It separates immutable evidence, six-clock temporal semantics, event/relation reasoning, shared-latent topic measurement, multilevel longitudinal psychometrics, compute backends, event intelligence, interpretation, and visual artifacts so scientific validity can be tested at each boundary.

## 2. Current implementation maturity

Protected main contains the Rust workspace foundation, immutable evidence records, exact source spans, strict versioned evidence JSON, stable content-redacting errors, repository quality contracts, and the canonical ADR/documentation authority graph. Typed six-clock values and uncertain intervals are implemented-main (merged PR #8 / `temporal_core`). Allen interval algebra and bounded path-consistency are implemented-main (merged PR #9 / `temporal_core`; Allen, 1983). Superseded PRs #5 and #6 are historical lineage only and are not current-product claims.

Capability maturity for later layers is recorded in [`docs/TRACEABILITY.md`](TRACEABILITY.md). Multilingual semantic units, TRSL-TM topic measurement, GPU/VRAM compute, model selection, TDT/CHRONOS (Allan, 2002; Anagnostopoulos et al., 2013), longitudinal ESEM/DSEM (Asparouhov & Muthén, 2009; Asparouhov et al., 2018; Marsh et al., 2014), network analysis, interpretation, and visual analytics remain accepted-target unless a TRACEABILITY row says otherwise. Unmerged or draft PRs are not implemented-main claims.

## 3. Immutable evidence requirements

Every source artifact and document must have immutable opaque identity distinct from SHA-256 content identity. Source bytes/text are bounded before allocation, owned immutably, and rehashed on reconstruction. Exact spans use half-open UTF-8 byte and Unicode-scalar coordinates plus optional page geometry; cross-document, mid-code-point, inconsistent, non-finite, out-of-page, reversed, or empty locations fail closed.

Evidence wire formats are explicitly versioned, reject unknown fields, and reconstruct through domain validation rather than bypassing invariants.

## 4. Temporal requirements

TEPP treats event/valid time, assertion time, document time, system time, available time, and knowledge cutoff as distinct nominal types. Analyses enforce `available_time <= knowledge_cutoff`. Exact, uncertain, open-ended, and unknown intervals preserve source precision and boundary semantics. Interval topology follows Allen (1983); event/time-marking vocabulary is aligned with ISO-TimeML (International Organization for Standardization, 2012) and may map outward to OWL-Time (Hobbs & Pan, 2017).

Merged PR #8 implements the typed-value, interval, wire, and schema primitives for this requirement on protected `main`. Historical-snapshot enforcement is owned by persistence/corpus-split layers (see TRACEABILITY); an unmerged PR is not protected-main enforcement.

Forward state-transition/input→process→outcome edges must satisfy temporally valid partial order. Retrospective, revision, citation, translation, support, and contradiction edges may point backward as provenance but never create reverse state transitions.

The Task 4 bounded Allen closure on protected `main` (merged PR #9) establishes path consistency only within its stated algebra/limits; it must not be documented as a proof of global satisfiability for unrestricted disjunctive interval networks.

## 5. Event/relation/membership target

Event mentions and event instances are distinct evidence/latent objects. Relations are typed and provenance-bearing. Entity roles such as customer, partner, competitor, author, department, project, and opportunity pool are time-varying assignments rather than static entity types.

Observation/model structures support cross-classified and multiple-membership assignments with explicit weights and validity intervals. A document or segment may belong to multiple organizational/project/event contexts simultaneously.

## 6. Multilingual measurement target

All supported languages share global topic identities and latent coordinates while native lexical/morphological channels remain language-specific. Concept/semantic-unit mapping must be span-grounded and versioned. Unknown meaning is isolated rather than silently forced into a known concept.

Language support is a validation claim, not a feature flag: each language profile requires alignment/invariance/error evidence. Repeated template/style/report wording is modeled as method/background structure rather than removed through indiscriminate stopword/TF-IDF/BM25 heuristics.

## 7. Topic and psychometric target

Shared-latent temporal/relational topic estimation provides posterior uncertainty and covariate effects. The product contract is TRSL-TM (ADR 0012); an STM-style logistic-normal family is the reference, not a shipped-backend claim (Blei & Lafferty, 2006; Roberts et al., 2014, 2019). Topic proportions are compositional (Aitchison, 1982); downstream correlation/ESEM uses logistic-normal coordinates or appropriate orthonormal log-ratio coordinates rather than naïve raw-proportion Pearson correlation.

Longitudinal ESEM/DSEM (Asparouhov & Muthén, 2009; Asparouhov et al., 2018; Marsh et al., 2014) must distinguish stable between-unit differences from within-unit temporal change, test measurement invariance where comparisons require it (American Educational Research Association, American Psychological Association, & National Council on Measurement in Education, 2014), account for irregular intervals when necessary, and propagate topic-posterior uncertainty through plausible values or joint estimation. These psychometric targets remain accepted-target.

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

## 14. Event Lineage criterion and Project Journey posterior contracts

- `tepp.lineage_pair_criterion_posterior.v2` binds exact pair identities,
  continuous independent-criterion and event-time draws, run/snapshot/cutoff,
  TDT/CHRONOS configuration, unique anchor alignment, and CPU/MLX receipts.
- `tepp.project_journey_posterior.v1` is a posterior DAG with no fixed start or
  total order. It preserves multiple predecessors, branches, transitions,
  exact ties, relation uncertainty, evidence ids, and record/event clock
  separation.
- Both strict JSON contracts reject unknown fields, mixed cardinalities,
  backward transition draws, missing evidence, ambiguous anchors, and
  method-derived parity failure.
- `event_core::fit_independent_criterion_posterior` is the Rust CPU reference
  for independently observed binary TDT link criteria. `analysis_engine`
  preserves exact pair identity and temporal draw arrays while fitting this
  Jeffreys posterior; it does not infer event dates from record order.
- `event_core::materialize_event_time_posterior` converts identified integer
  posterior mass over unique event-clock atoms into canonical complete draws.
  It performs no date inference, probability repair, or nearest-date fallback;
  atom/mass estimation remains an owning temporal-model responsibility.
- Apple Silicon acceleration uses a macOS-native Rust-owned MLX Metal service.
  Compose calls it through authenticated Unix-socket or host-gateway transport;
  Colima/Linux never claims Metal execution. Rust owns all arithmetic;
  `rust_cpu` f64 is a tested portability reference/fallback. The independent
  criterion CPU estimator is implemented; full temporal posterior production,
  artifact assembly, and real MLX parity remain separate acceptance gates.
  ADR 0025 normatively owns host authentication,
  actual-backend receipts, parity, fallback, operability, tests, and rollback.

## References

The full APA 7th register is [`docs/research/standards-and-literature.md`](research/standards-and-literature.md). Method claims in this TRD use:

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B, 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural equation models. *Structural Equation Modeling, 25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling. *Structural Equation Modeling, 16*(3), 397–438. https://doi.org/10.1080/10705510903008204

Blei, D. M., & Lafferty, J. D. (2006). Dynamic topic models. In *Proceedings of the 23rd International Conference on Machine Learning* (pp. 113–120). ACM. https://doi.org/10.1145/1143844.1143859

Hobbs, J. R., & Pan, F. (2017). *Time ontology in OWL* (W3C Recommendation). World Wide Web Consortium. https://www.w3.org/TR/owl-time/

International Organization for Standardization. (2012). *Language resource management—Semantic annotation framework (SemAF)—Part 1: Time and events (SemAF-Time, ISO-TimeML)* (ISO 24617-1:2012).

Marsh, H. W., Morin, A. J. S., Parker, P. D., & Kaur, G. (2014). Exploratory structural equation modeling: An integration of the best features of exploratory and confirmatory factor analysis. *Annual Review of Clinical Psychology, 10*, 85–110. https://doi.org/10.1146/annurev-clinpsy-032813-153700

Roberts, M. E., Stewart, B. M., Tingley, D., Lucas, C., Leder-Luis, J., Gadarian, S. K., Albertson, B., & Rand, D. G. (2014). Structural topic models for open-ended survey responses. *American Journal of Political Science, 58*(4), 1064–1082. https://doi.org/10.1111/ajps.12103

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for structural topic models. *Journal of Statistical Software, 91*(2), 1–40. https://doi.org/10.18637/jss.v091.i02
