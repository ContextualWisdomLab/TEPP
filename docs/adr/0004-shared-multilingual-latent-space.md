# ADR 0004: Shared Multilingual Latent Space

**Status:** Accepted  
**Date:** 2026-08-05

## Decision

TEPP learns one shared latent semantic and topic space across languages. Equivalent meanings share concept prototypes, topic identities, and document coordinates. Language-specific morphology, script, syntax, lexical emissions, and content deviations remain explicit.

The system preserves original text, uses language-tailored segmentation and morphology, treats Universal POS as a soft source prior, and validates LLM-proposed semantic units against exact spans and a versioned concept dictionary. Stopword deletion is not the default; TF-IDF and BM25 do not weight inferential estimation.

Language profiles are labeled validated, calibrated, provisional, or unresolved based on alignment, reliability, fairness, and measurement-invariance evidence.

## Consequences

Separate monolingual topics are not merely matched after fitting. Long-tail languages can enter the same model without receiving unsupported validity claims. Template, section, copied-text, style, prompt, modality, and corpus-background sources prevent boilerplate from masquerading as substantive meaning.

## Verification

Parallel and comparable corpora test concept accuracy, topic-coordinate RMSE/ICC, alignment, configural/metric/scalar conditions as applicable, code switching, linguistic drift, protected-group errors, and unsupported-language uncertainty.
