# Temporal Event Psychometrics Platform — Proposed PRD v0.5 delta

**Status:** Proposed; not an approved or implemented capability  
**Date:** 2026-08-15  
**Base:** Approved PRD v0.4  
**Owning decision:** ADR 0017  
**Change scope:** Section 6 multilingual evidence measurement, Section 12 LLM responsibilities, Section 16 persistence, Section 18 delivery phase 2, and Section 19 release evidence.

## 1. Reason for the version proposal

Approved PRD v0.4 establishes multilingual evidence, exact source spans, optional LLM semantic-unit proposals, and a phase for multilingual evidence and semantic units. It also mentions language-tailored boundaries. The current product requirement is stricter:

> The base semantic-span and embedding-budget pipeline must remain correct without knowing, trusting, or branching on the language of the input.

This proposal does not remove language metadata, native lexical channels, language-profile validation, or measurement-invariance studies. It separates those scientific evaluation responsibilities from the correctness of source segmentation and model-budget enforcement.

## 2. Product outcome

TEPP converts any accepted Unicode source into exact, hierarchical semantic spans whose final embedding payloads cannot exceed the active embedding model's verified input limit.

A buyer can:

- index mixed-language and unknown-language evidence without selecting a tokenizer by language;
- trace every retrieved unit to immutable source offsets;
- retrieve precise leaf evidence and restore its section, document, and neighboring context;
- change embedding providers or limits through a versioned profile rather than code changes;
- audit why a boundary was chosen and whether processing degraded to structure-only mode;
- prove that no request exceeded the model limit and no source was silently truncated.

## 3. Functional requirements

### FR-SS-001 — Language-independent base control flow

The system SHALL accept absent, mixed, unresolved, or incorrect language metadata without changing the base segmentation, packing, overflow, or recursive fallback algorithm.

Language metadata MAY be used for stratified evaluation, lexical rendering, model-invariance analysis, and optional annotation adapters. It SHALL NOT be a required control input.

### FR-SS-002 — Typed source blocks

The system SHALL preserve headings, paragraphs, list items, table row groups, code blocks, captions, dialogue turns, DOM regions, and unknown blocks as typed exact-span evidence.

### FR-SS-003 — Micro-unit construction

The system SHALL construct micro-units from source structure and Unicode-safe boundary hints without requiring whitespace-delimited words, morphology, stopwords, TF-IDF, BM25, or translation.

### FR-SS-004 — Versioned model profile

Every embedding request SHALL reference an immutable model profile containing provider, model/revision, tokenizer profile and digest, maximum input tokens, vector dimensions, input role, metadata-template version, source, and verification date.

The initial `text-embedding-3-large` profile SHALL record:

- maximum input: 8,192 tokens;
- tokenization profile: `cl100k_base`;
- default vector length: 3,072 dimensions;
- optional requested dimensions as a profile field.

These are initial provider facts, not universal constants.

### FR-SS-005 — Final-payload token gate

The system SHALL render the complete payload, including selected title/heading metadata and separators, before the authoritative token count.

It SHALL refuse any payload for which:

\[
\operatorname{tokens}(\operatorname{payload}) >
\operatorname{max\_input\_tokens}.
\]

Overflow rate SHALL equal zero.

### FR-SS-006 — Semantic packing

The system SHALL pack adjacent micro-units under mandatory structure boundaries, a configurable target range, the hard model budget, and optional dense semantic-boundary evidence.

No boundary score SHALL use a language identity, TF-IDF, BM25, or bag-of-words weight.

### FR-SS-007 — Oversized-unit recovery

The system SHALL recursively split an oversized unit by child structure, Unicode-safe sentence/line hints, punctuation/clause hints, and finally tokenizer offsets. It SHALL never silently truncate source text.

### FR-SS-008 — Hierarchical context graph

The system SHALL create leaf, section, and document spans with parent and neighboring-unit relations. A retrieved leaf SHALL be able to reconstruct bounded parent and neighbor context without replacing source evidence with a summary.

### FR-SS-009 — Explicit degradation

If semantic-similarity or optional LLM refinement is unavailable, the system MAY continue using deterministic structure-only packing and SHALL record the degraded boundary mode.

Tokenizer-profile, source-offset, and model-profile failures SHALL fail closed.

### FR-SS-010 — Ecosystem ports

TEPP SHALL expose provider-neutral ports for token counting, embedding, optional semantic similarity, optional LLM refinement, vector persistence, and batch transport.

The owner boundaries are:

- TEPP: unit identity, hierarchy, budget, evaluation, provenance;
- contextual-orchestrator: optional LLM proposal/verification;
- pg-llm-batch: optional batch/token-count transport adapter;
- semantic-data-portal: downstream graph/vector consumer;
- EmbedRelay: embedding-space migration.

No service may directly read or write another service's application tables.

## 4. Non-functional requirements

### NFR-SS-001 — Determinism

With the same source bytes, parser version, policy, model profile, tokenizer artifact, and deterministic similarity fixture, span identities and payload hashes SHALL be reproducible.

### NFR-SS-002 — Auditability

Every span SHALL record source references, block types, boundary reasons, policy version, model profile, token count, payload hash, hierarchy, degradation status, and creation provenance.

### NFR-SS-003 — Security

Document text SHALL be treated as untrusted data. It cannot modify tool authority, network destinations, credentials, templates, profiles, or budgets. Logs SHALL not contain source text, secrets, or vector values.

### NFR-SS-004 — Modularity

The deterministic segmentation/budget core SHALL run standalone with fake or local ports. Provider SDKs, databases, and contextual-orchestrator SHALL remain optional adapters.

### NFR-SS-005 — Quality

New production logic requires 100% line and branch coverage, complete public/safety docstrings, property tests, fuzz tests, exact-head CI, independent review, SBOM/provenance updates, and CHANGELOG evidence.

## 5. Acceptance metrics

### Hard gates

- payload overflow: 0;
- silent truncation: 0;
- invalid source-offset acceptance: 0;
- language-code control branches in the base algorithm: 0;
- unversioned model-profile requests: 0;
- production line/branch coverage: 100%;
- public and safety-contract docstrings: 100%.

### Comparative metrics

Against fixed-window and paragraph-only baselines, report:

- Recall@1/5/10;
- nDCG@5/10;
- MRR;
- duplicate-hit rate;
- boundary precision/recall;
- exact-span agreement;
- parent/neighbor context-restoration success;
- indexing latency and throughput;
- provider input tokens and cost;
- vector and relation storage.

Results SHALL include uncertainty and shall be stratified by source structure, script/language profile, and mixed-language status. Stratification evaluates robustness; it does not select the base algorithm.

## 6. User stories

### Evidence engineer

As an evidence engineer, I can index a document whose language is absent or mixed and receive a complete manifest showing exact spans, token counts, boundary reasons, and hierarchy, so that I can audit the source without reverse-engineering a tokenizer pipeline.

### Researcher

As a researcher, I can compare fixed windows, paragraph-only units, semantic spans, and hierarchical retrieval on the same gold queries, so that a chunking claim is supported by retrieval evidence rather than intuition.

### Platform operator

As a platform operator, I can change a model profile without changing segmentation code, and the system refuses unverified limits or tokenizers, so that provider drift cannot create silent overflows.

### Downstream consumer

As a semantic-data-portal or LineageWeave consumer, I can retrieve a precise leaf and its parent/neighbor graph while preserving `embedding_space_id`, so that cross-model and cross-context comparisons remain valid.

## 7. Scope sequencing

### P0 — Contract and deterministic baseline

- exact typed source blocks;
- model profiles and token-count port;
- final-payload budget gate;
- deterministic micro-units and recursive fallback;
- hierarchy and provenance;
- fixed-window and paragraph-only baselines;
- hostile Unicode and mixed-script tests.

### P1 — Dense semantic refinement

- adjacent-unit similarity port;
- boundary-score calibration;
- deterministic caching;
- retrieval benchmark and threshold selection;
- structure-only fallback.

### P2 — Optional LLM and advanced context

- evidence-bounded LLM boundary proposals and verifier;
- versioned section/document summaries;
- direct-vs-orchestrated ablations;
- open-weight/token-level late-chunking experiments where the model contract permits them.

True late chunking is not claimed for `text-embedding-3-large` because the API does not expose the token-level contextual states and pooling control required by that technique.

## 8. Data requirements

Normalized two-or-more-word `snake_case` objects are proposed:

- `source_block`;
- `semantic_unit`;
- `semantic_unit_relation`;
- `embedding_model_profile`;
- `embedding_payload`;
- `embedding_vector_record`;
- `embedding_evaluation_run`.

The logical model remains in third normal form:

- source text belongs to the source artifact/document authority;
- units reference source offsets rather than duplicating text;
- model/profile metadata is stored once and referenced;
- vectors bind to an explicit embedding space and payload;
- evaluation runs reference immutable corpora, queries, policies, and profiles.

## 9. Release claim boundary

Acceptance of this PRD delta would authorize implementation planning only. It would not establish:

- support for every language;
- cross-language measurement equivalence;
- retrieval superiority;
- a production-ready OpenAI connector;
- a completed vector database;
- a released feature.

Those claims require the acceptance evidence in this document and ADR 0014 promotion.