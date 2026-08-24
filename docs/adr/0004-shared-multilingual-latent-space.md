# ADR 0004 — Shared multilingual latent semantic space

**Decision status:** Accepted  
**Implementation maturity:** accepted-target — corpus-background-versus-unique-content identity in `corpus_background` on the active PR; shared-space estimators remain accepted-target  
**Implementation maturity:** accepted-target — modality-versus-unique-content identity in `modality_source` on the active PR; shared-space estimators remain accepted-target  
**Implementation maturity:** accepted-target — copied-versus-unique-content identity in `copied_text` on the active PR; shared-space estimators remain accepted-target  
**Implementation maturity:** accepted-target — style-versus-unique-content identity in `style_source` on the active PR; shared-space estimators remain accepted-target  
**Implementation maturity:** partial — default stopword-deletion refusal is `stopword_deletion` on the active PR; shared-space estimators, language profiles, and TF-IDF/BM25 inferential-weight refusal remain accepted-target
**Date:** 2026-08-05  
**Supersedes:** None. ADR 0012 governs the complete topic-estimator/backend/global-topic contract built on this multilingual measurement decision.

## Context

TEPP must compare meaning across Korean, English, Japanese, Chinese, Vietnamese, Indonesian, French, German, Turkish, and later language profiles without fitting unrelated monolingual topic systems and matching them post hoc. Translation-only preprocessing can erase culturally or technically meaningful lexical differences, while a single language-neutral tokenizer is not a realistic guarantee for every script.

The product therefore needs a language-independent analytical contract while preserving language-specific segmentation, morphology, lexical evidence, uncertainty, and validation status.

## Decision

TEPP learns one shared latent semantic and topic space across languages. Equivalent meanings share concept prototypes, topic identities, and document coordinates. Language-specific morphology, script, syntax, lexical emissions, and content deviations remain explicit rather than being forced to match.

This ADR owns the multilingual measurement substrate. The product topic-estimator contract is **Temporal Relational Shared-Latent Topic Measurement (TRSL-TM)** under ADR 0012. Temporal topic identity follows the dynamic topic-model family (Blei & Lafferty, 2006). An STM-style logistic-normal document-coordinate model (Roberts et al., 2014, 2019) is the **reference family**, not a claim that every compliant backend is already shipped. Implementation maturity for this ADR and for TRSL-TM remains accepted-target.

Original text and exact source spans are preserved. Segmentation and morphology are language-tailored. Universal POS/dependency information may act as soft source evidence or priors but does not authorize irreversible deletion. LLM-proposed semantic units must resolve to exact source evidence and a versioned concept/semantic contract; unknown meaning can remain unresolved rather than being forced into a known concept.

Stopword deletion is not the default; TF-IDF and BM25 are not inferential weights for the statistical estimator. Repeated template/section/copied/style/prompt/modality/background wording is modeled as method/background structure so boilerplate does not masquerade as substantive latent meaning.

Language profiles are labeled validated, calibrated, provisional, or unresolved based on alignment, reliability, fairness, and measurement-invariance evidence. “Supported by architecture” is not the same as “validated for interpretation.”

## Non-goals

- do not claim tokenizer-free text processing in the literal linguistic sense;
- do not treat machine translation as lossless measurement equivalence;
- do not grant every long-tail language the same validity claim merely because the model can accept its text;
- do not define the topic estimator itself here; ADR 0012 owns that layer.

## Alternatives considered

1. **Translate all content to one pivot language** — rejected as the primary measurement architecture because translation error and culture/domain-specific lexical distinctions become hidden measurement error.
2. **Fit separate monolingual latent spaces and align them after estimation** — rejected because longitudinal/cross-language topic identity and invariance become post-hoc.
3. **One shared latent semantic space with explicit native-language channels and validation profiles** — accepted.

## Consequences

Long-tail languages can enter the same versioned measurement system without receiving unsupported validity claims. Equivalent content can be compared in one coordinate system while language-specific expression remains inspectable. Boilerplate and source-method effects are preserved as modeled variance rather than removed invisibly.

## Failure and recovery

Span mismatch, unsupported language segmentation, concept ambiguity, low calibration, cross-language misalignment, fairness/invariance failure, or provider unavailability produces provisional/unresolved status rather than a false equivalence claim. Recovery may use corrected human/semantic mappings or a new language-profile version while retaining the original evidence and prior model artifact.

## Security, privacy, and governance impact

Language, dialect, names, and lexical style can be identifying or sensitive. Purpose-bound access under ADR 0009 applies to semantic units, concept mappings, and provider payloads. Documents remain untrusted input and cannot modify prompts, tool policy, concept governance, or release authority.

## Compatibility and migration

Concept dictionaries, segmentation profiles, language tags, prompt/model contracts, and alignment artifacts are versioned. Changes that alter latent meaning or equivalence rules require a superseding ADR/PRD update; backend-only changes follow ADR 0012 compatibility rules.

## Verification

Parallel/comparable corpora and human-reviewed evidence test exact-span F1, concept precision/recall, confidence calibration/Brier score, topic-coordinate RMSE/ICC where meaningful, language alignment, code switching, lexical/semantic drift, protected-group error slices, and configural/metric/scalar or partial invariance as required for the intended comparison.

## Rollback and supersession

Rollback selects the prior validated language/concept/profile version and does not silently remap already-published artifacts. Supersede only through a decision that maintains cross-language comparability and explicit validity status or intentionally changes the measurement target with new PRD/validation evidence.

## References

Blei, D. M., & Lafferty, J. D. (2006). Dynamic topic models. In *Proceedings of the 23rd International Conference on Machine Learning* (pp. 113–120). ACM. https://doi.org/10.1145/1143844.1143859

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for structural topic models. *Journal of Statistical Software, 91*(2), 1–40. https://doi.org/10.18637/jss.v091.i02

Roberts, M. E., Stewart, B. M., Tingley, D., Lucas, C., Leder-Luis, J., Gadarian, S. K., Albertson, B., & Rand, D. G. (2014). Structural topic models for open-ended survey responses. *American Journal of Political Science, 58*(4), 1064–1082. https://doi.org/10.1111/ajps.12103
