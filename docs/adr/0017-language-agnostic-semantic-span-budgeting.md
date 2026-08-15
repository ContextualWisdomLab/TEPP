# ADR 0017 — Language-agnostic semantic-span and embedding-budget authority

**Decision status:** Proposed  
**Implementation maturity:** research-only  
**Date:** 2026-08-15  
**Supersedes:** None; narrows the implementation of ADR 0004 multilingual alignment and complements ADR 0008 exact evidence spans, ADR 0010 LLM orchestration, ADR 0011 modular MSA authority, and ADR 0012 topic-measurement inputs.

## Context

TEPP must turn documentary evidence into meaning-bearing embedding inputs without assuming that a document uses one known language, whitespace-delimited words, one script, or a language-specific morphology pipeline. A document may mix Korean, English, Chinese, Japanese, Thai, Arabic, identifiers, code, formulas, emoji, tables, quotations, and translated passages within one source span.

The current draft PR #56 uses blank-line splitting as the semantic-unit implementation. That is a useful source-structure hint, but it is not the required architecture. Blank lines do not establish semantic coherence, do not protect the final rendered embedding payload from a model context overflow, do not represent headings, lists, tables, code, captions, dialogue turns, or DOM regions, and do not restore parent or neighboring context at retrieval time.

The target OpenAI profile currently relevant to this decision, `text-embedding-3-large`, has a provider-documented per-input ceiling of 8,192 tokens, maps to `cl100k_base` in OpenAI's current tokenizer registry, and emits a full vector of up to 3,072 dimensions with an optional `dimensions` parameter. These are mutable provider facts, not universal TEPP constants. TEPP therefore records both the provider-documented ceiling and its own operational hard limit, together with exact source, retrieval date, tokenizer artifact digest, and verification method.

## Decision

TEPP owns a provider-neutral **language-agnostic semantic-span pipeline**. Language identification may be recorded for evaluation, governance, or later measurement-invariance analysis, but it MUST NOT select the core segmentation, packing, overflow, or fallback algorithm.

The pipeline has seven ordered contracts.

### 1. Immutable evidence and source structure

`evidence_core` preserves immutable source bytes, decoded text, exact byte and Unicode-scalar offsets, and available structural evidence. Parsers emit typed source blocks such as:

- `document_title`;
- `section_heading`;
- `paragraph_block`;
- `list_item`;
- `table_row_group`;
- `code_block`;
- `caption_block`;
- `dialogue_turn`;
- `dom_region`;
- `unknown_block`.

A paragraph is one candidate block, not automatically one final embedding unit.

### 2. Language-agnostic micro-units

A future `semantic_preprocessor` crate converts source blocks into exact-span micro-units. It uses source structure, Unicode-safe grapheme and sentence boundary hints, line boundaries, punctuation categories, and bounded token windows. It does not require a language code, dictionary, word count, stopword list, stemming, morphology, TF-IDF, BM25, or translation into a pivot language.

Language-tailored analyzers may later supply optional annotations behind an adapter, but failure or absence of such an analyzer cannot change the correctness of the base pipeline.

### 3. Versioned embedding-model profiles

Every embedding request is governed by an immutable `embedding_model_profile` containing at least:

- provider and endpoint family;
- model identifier and observed revision;
- tokenizer profile and tokenizer artifact digest;
- provider-documented maximum input tokens;
- TEPP operational hard-limit tokens and explicit safety margin;
- full/default and requested vector dimensions;
- input role;
- metadata-template version;
- profile source URL, retrieval date, source artifact digest, and verification method.

The following invariant is database- and domain-enforced:

\[
\operatorname{operational\_hard\_limit}
\leq
\operatorname{provider\_documented\_maximum}.
\]

For `text-embedding-3-large`, the initial profile records `provider_documented_max_input_tokens = 8192`, `operational_hard_limit_tokens <= 8192`, `tokenizer_profile = cl100k_base`, and `full_dimensions = 3072`. TEPP code does not scatter these values as literals. Reducing the operational limit or changing a provider/tokenizer fact creates a new profile revision rather than mutating an existing one.

### 4. Budget-aware semantic-span packing

Micro-units are packed in document order. A candidate span may be merged only when all of the following hold:

1. no mandatory structural boundary is crossed;
2. the final rendered payload remains within the active profile's operational hard limit;
3. the configurable target-size policy allows the merge;
4. semantic evidence does not indicate a strong boundary.

The boundary score may combine structural break strength, block-type transition, dense adjacent-unit similarity drop, and length pressure. It MUST NOT contain language identity or TF-IDF/BM25 features.

Pre-packing may reserve tokens for headings and other metadata, but the authoritative gate tokenizes the **final rendered payload**, not only its source text. No request may be sent when:

\[
\operatorname{tokens}(\operatorname{rendered\_payload})
>
\operatorname{operational\_hard\_limit\_tokens}.
\]

The hard overflow rate is therefore required to be zero.

### 5. Recursive oversized-unit recovery

When one candidate unit exceeds the budget, TEPP applies the following order:

1. split by known child blocks;
2. split at Unicode-safe sentence or line boundaries;
3. split at punctuation or clause-like boundaries that preserve exact offsets;
4. split by tokenizer offsets as the final fallback.

The final fallback may use a bounded overlap, but overlap is not the primary context strategy. An unsplittable unit that cannot be represented with valid offsets fails closed as `unit_unsplittable_under_budget`; it is never silently truncated.

### 6. Hierarchical context graph

TEPP indexes three coordinated levels:

- `leaf_span` — precise retrieval unit;
- `section_span` — parent context and section summary;
- `document_span` — coarse document routing and document summary.

Each leaf retains parent, previous-sibling, next-sibling, source-block, and exact-source references. Retrieval first identifies document or section candidates, retrieves leaf spans, and restores bounded parent and neighbor context. Summaries are separate, versioned derived artifacts and never replace source evidence.

### 7. Provider and ecosystem boundaries

- TEPP owns exact semantic-span identity, hierarchy, budgeting policy, evaluation, and evidence provenance.
- `contextual-orchestrator` may provide optional LLM boundary proposals, summaries, or verification through a versioned API; it does not own TEPP evidence or token-budget authority.
- `pg-llm-batch` may implement a batch transport and a Postgres `pg_tiktoken` token-count adapter; it does not own segmentation policy.
- `semantic-data-portal` may persist and retrieve TEPP vectors and graph relations as a downstream consumer.
- `EmbedRelay` governs later embedding-space migration; it does not translate source segmentation into a different semantic claim.
- Direct cross-repository application-table access remains prohibited under ADR 0011.

## Non-goals

This ADR does not:

- assert that every language has been psychometrically validated;
- make language detection a prerequisite;
- translate all text to English or another pivot language;
- use TF-IDF, BM25, or bag-of-words weights for semantic-span construction;
- claim that a blank line, sentence, or paragraph is always a complete semantic unit;
- implement true late chunking for a closed embedding API that does not expose contextual token embeddings and pooling control;
- let an LLM directly write authoritative spans without deterministic offset and budget validation;
- treat one live provider probe as permanent evidence of an immutable external contract;
- establish a stable public release.

## Alternatives considered

1. **Blank-line paragraph splitting only** — rejected as the final architecture because formatting is not semantic coherence and the final payload is not budgeted.
2. **Fixed token windows with fixed overlap** — retained only as the final recovery mechanism because it has predictable size but often cuts concepts and creates duplicate retrieval.
3. **Language detection followed by language-specific tokenizers** — rejected as the base control flow because code switching, unknown languages, and long-tail scripts become correctness failures.
4. **Translation-first embedding** — rejected because translation can remove distinctions, introduces an additional model and measurement error, and breaks exact-source correspondence.
5. **LLM-only segmentation** — rejected because it is nondeterministic, expensive, vulnerable to prompt injection, and cannot be trusted for exact offsets or hard token limits.
6. **Structure + model budget + optional dense boundary + hierarchical context** — selected.

## Consequences

- exact source spans remain the canonical evidence identity;
- paragraph-only logic becomes one structural adapter rather than the product contract;
- provider limits and TEPP operational limits are distinct versioned profile facts;
- the same algorithm works when language metadata is absent, mixed, wrong, or unresolved;
- provider calls are impossible until the final payload is counted;
- retrieval can use small precise leaves without abandoning larger context;
- a deterministic structure-only mode remains available when dense or LLM services fail;
- model-specific and provider-specific behavior stays behind ports, preserving standalone and MSA use.

The approach costs more metadata, hierarchy storage, and evaluation work than fixed windows. Dense boundary refinement also adds embedding calls. Those costs are measured rather than hidden.

## Failure and recovery

TEPP returns typed, content-redacting failures:

- `tokenizer_profile_unavailable`;
- `embedding_profile_unverified`;
- `embedding_payload_too_large`;
- `unit_unsplittable_under_budget`;
- `invalid_source_offset`;
- `semantic_similarity_unavailable`;
- `embedding_provider_unavailable`;
- `embedding_provider_contract_diverged`.

`semantic_similarity_unavailable` may degrade to deterministic structure-only packing and records `boundary_mode = structure_only`. Tokenizer/profile/offset failures cannot degrade because they protect correctness. Provider retries and fallback are bounded and recorded; they never change span identity silently.

If the provider rejects a payload that the verified profile permits, the adapter returns `embedding_provider_contract_diverged`, records the response class without source text, and fails closed. Operations may publish a new profile revision with a lower operational hard limit after verification; no existing profile, payload digest, span identity, or vector provenance is rewritten.

## Security, privacy, and governance impact

Document content is untrusted data. It cannot alter the model profile, metadata template, access list, network destination, tool authority, or token budget. LLM boundary proposals require exact source references and deterministic validation. No external resource is fetched during segmentation.

TEPP preserves PII required for authorized work under ADR 0009 rather than blanket masking it. Provider disclosure is purpose-bound and recorded. Logs and metrics contain identifiers, counts, policy versions, hashes, and outcomes—not source text, secrets, or embedding vectors.

Embedding vectors, tokenizer artifacts, summaries, and model-profile artifacts are versioned and integrity checked. An embedding vector is always bound to `embedding_space_id`, model profile, input role, payload hash, and source-span set.

## Compatibility and migration

The initial implementation is additive. Existing document and source-span contracts remain valid. Paragraph-only outputs from draft PR #56 are not migrated as authoritative semantic units.

A future persistence migration uses normalized two-or-more-word `snake_case` objects:

- `source_block`;
- `semantic_unit`;
- `semantic_unit_relation`;
- `embedding_model_profile`;
- `embedding_payload`;
- `embedding_vector_record`;
- `embedding_evaluation_run`.

`semantic_unit_relation` stores parent/previous/next/derived-from relations without duplicating document text. `embedding_vector_record` references an `embedding_space_id`; vectors from different spaces are never compared directly.

## Verification

Acceptance requires falsifiable evidence.

### Correctness

- final rendered-payload overflow rate is exactly zero relative to the profile's operational hard limit;
- source reconstruction from every leaf span is exact;
- byte and Unicode-scalar offsets remain valid under mixed scripts, combining marks, emoji ZWJ sequences, RTL text, and malformed-input rejection;
- no control-flow branch depends on a language code;
- unsplittable oversized inputs fail closed without truncation;
- every operational limit is less than or equal to its provider-documented ceiling and tied to source/retrieval/digest evidence.

### Retrieval quality

Compare at least:

1. fixed token windows with overlap;
2. paragraph-only indexing;
3. structure + budget;
4. structure + budget + dense boundary;
5. hierarchical retrieval with parent/neighbor restoration.

Report Recall@k, nDCG@k, MRR, duplicate-hit rate, context-restoration success, latency, input tokens, storage, and cost. Human-gold boundaries report precision, recall, and span agreement. Results are stratified by script/language profile and mixed-language status for evaluation only.

### Robustness fixtures

Tests include whitespace-delimited and non-whitespace-delimited scripts, Korean/English code switching, Chinese, Japanese, Thai, Arabic RTL, emoji and combining marks, tables, lists, code, long unpunctuated text, one token-dense source block, repeated boilerplate, and adversarial prompt-like document text.

### Model profile

Deterministic unit/property tests accept a final rendered payload exactly at the profile's operational hard limit and refuse or recursively split one token above it. The initial `text-embedding-3-large` profile additionally proves that its configured operational hard limit does not exceed the provider-documented 8,192-token ceiling, uses the pinned `cl100k_base` artifact, and binds the full 3,072-dimension output or an explicit requested dimension.

A scheduled live provider contract probe may corroborate the external boundary using `NVIDIA_NIM_API_KEY` only when the tested provider is reached through the reviewed orchestration/gateway policy; direct OpenAI live verification uses an explicitly authorized provider credential and is not a required unit-test dependency. Live rejection at or below the operational limit is treated as contract divergence, not as permission to truncate.

## Rollback and supersession

Rollback disables dense/LLM refinement and uses deterministic structure-only packing with the last validated model profile. It does not restore blank-line-only segmentation as an authoritative semantic algorithm.

Supersession requires an ADR that preserves exact evidence offsets, versioned provider and operational model limits, final-payload token enforcement, language-independent base correctness, explicit degradation, provider-divergence handling, and retrieval-quality evidence. A later open-weight model may add true late chunking, but that capability must remain separate from the `text-embedding-3-large` API profile unless the provider exposes the necessary token-level contract.
