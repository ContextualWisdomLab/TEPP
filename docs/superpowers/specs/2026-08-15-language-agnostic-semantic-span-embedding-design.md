# Language-Agnostic Semantic Span Embedding Design

**Status:** Proposed design  
**Repository owner:** `ContextualWisdomLab/TEPP`  
**Date:** 2026-08-15  
**Architecture authority:** ADR 0017  
**Product authority:** proposed PRD v0.5 delta

## 1. Design decision

TEPP is the canonical owner because semantic units are fallible, exact-span observations feeding its multilingual evidence, topic, event, and psychometric layers. This is not primarily an embedding-space migration concern, an LLM routing concern, a batch-provider concern, or a vector-catalog concern.

The implementation is split so that TEPP can also operate as a reusable module:

```text
source adapters
  -> evidence_core
  -> semantic_preprocessor
  -> embedding ports
  -> vector/graph consumers
  -> TEPP estimators
```

Adjacent CWL systems connect through ports:

```text
contextual-orchestrator -- optional LLM proposals / summaries / verification
pg-llm-batch          -- optional batch transport and pg_tiktoken adapter
semantic-data-portal  -- optional vector + relation persistence/search
EmbedRelay            -- later embedding-space migration
```

## 2. Why PR #56 is not the target design

Draft PR #56 equates semantic units with `text.split("\n\n")` and verifies two English paragraphs. That implementation preserves offsets but misses the governing requirements:

- language-independent correctness;
- final payload token counting;
- a versioned embedding-model profile;
- structural units beyond paragraphs;
- recursive recovery for one oversized paragraph;
- semantic-boundary evidence;
- hierarchy and context restoration;
- mixed-script and hostile Unicode tests;
- provider and ecosystem ownership boundaries.

The PR therefore must be superseded rather than expanded by small patches around the same public API.

## 3. Architecture

### 3.1 `evidence_core`

Owns immutable source evidence only.

Proposed interfaces:

```rust
pub enum SourceBlockKind {
    DocumentTitle,
    SectionHeading,
    ParagraphBlock,
    ListItem,
    TableRowGroup,
    CodeBlock,
    CaptionBlock,
    DialogueTurn,
    DomRegion,
    UnknownBlock,
}

pub struct SourceBlock {
    block_id: EvidenceId,
    document_id: EvidenceId,
    block_kind: SourceBlockKind,
    source_span: SourceSpan,
    parent_block_id: Option<EvidenceId>,
    ordinal_index: u32,
}
```

Invariants:

- exact byte and Unicode-scalar offsets;
- source reconstruction;
- no text duplication as authority;
- parent belongs to the same document;
- ordinal ordering is total within a parent;
- unknown blocks are retained, not discarded.

### 3.2 `semantic_preprocessor`

Owns model-independent span construction and budget-aware packing.

Proposed interfaces:

```rust
pub struct EmbeddingModelProfile {
    profile_id: EvidenceId,
    provider_code: String,
    model_identifier: String,
    observed_revision: String,
    tokenizer_profile: String,
    tokenizer_digest: ContentDigest,
    max_input_tokens: u32,
    default_dimensions: u32,
    requested_dimensions: Option<u32>,
    input_role: EmbeddingInputRole,
    metadata_template_version: String,
    verified_at: Timestamp,
}

pub trait TokenCounter {
    type Error;

    fn count_tokens(
        &self,
        profile: &EmbeddingModelProfile,
        payload: &str,
    ) -> Result<u32, Self::Error>;

    fn token_offsets(
        &self,
        profile: &EmbeddingModelProfile,
        payload: &str,
    ) -> Result<Vec<TokenOffset>, Self::Error>;
}

pub trait AdjacentSimilarity {
    type Error;

    fn similarities(
        &self,
        units: &[MicroUnit],
    ) -> Result<Vec<f64>, Self::Error>;
}

pub struct SemanticSpanPolicy {
    target_input_tokens: u32,
    preferred_max_input_tokens: u32,
    metadata_reserve_tokens: u32,
    minimum_unit_tokens: u32,
    maximum_overlap_tokens: u32,
    semantic_drop_threshold: f64,
}

pub enum BoundaryMode {
    StructureOnly,
    StructureAndDenseSimilarity,
    StructureDenseAndLlmVerified,
}

pub struct SemanticUnit {
    unit_id: EvidenceId,
    unit_level: SemanticUnitLevel,
    source_spans: Vec<SourceSpan>,
    parent_unit_id: Option<EvidenceId>,
    previous_unit_id: Option<EvidenceId>,
    next_unit_id: Option<EvidenceId>,
    token_count: u32,
    payload_digest: ContentDigest,
    boundary_mode: BoundaryMode,
    boundary_reasons: Vec<BoundaryReason>,
}
```

The exact domain types can be adjusted to repository conventions, but the responsibilities and invariants are fixed.

### 3.3 `tepp_api`

Owns versioned DTOs and ports, not provider credentials.

Data-plane operations:

```text
POST /v1/semantic-span-runs
GET  /v1/semantic-span-runs/{run_id}
GET  /v1/semantic-span-runs/{run_id}/manifest
POST /v1/embedding-model-profiles/validate
POST /v1/semantic-span-evaluations
```

The API does not expose raw provider secrets or direct database handles. Bulk unit/vector interchange uses Arrow IPC or Parquet only after a schema version is fixed.

## 4. Data flow

```text
1. Ingest immutable source
2. Parse typed source blocks
3. Build Unicode-safe micro-units
4. Load verified embedding-model profile
5. Render candidate payload with title/heading metadata
6. Count final payload tokens
7. Merge adjacent units under structure + budget + optional similarity
8. Recursively split any oversized unit
9. Validate exact offsets and final token count
10. Build leaf / section / document hierarchy
11. Produce immutable span manifest
12. Call embedding provider through a port
13. Persist vector with embedding_space_id and payload hash
14. Evaluate retrieval and context restoration
```

No language detection step is required. A language/profile classifier may run in parallel and attach metadata for evaluation or psychometric invariance.

## 5. Payload template

A payload is deterministic and versioned. The P0 template is intentionally small:

```text
[document_title]
{title}

[heading_path]
{heading_1} > {heading_2}

[content]
{source_text}
```

Rules:

- omit an absent field rather than render `null`;
- normalize template line endings without rewriting source content;
- source text remains exact inside the content slot;
- metadata values are themselves exact source or approved metadata references;
- count the complete rendered payload;
- bind `metadata_template_version` and payload digest to the vector record.

## 6. Packing algorithm

Pseudocode:

```text
micro_units = build_micro_units(source_blocks)

for unit in micro_units:
    if final_payload_tokens(unit) > hard_limit:
        emit(recursive_split(unit))
        continue

    candidate = current + unit
    if mandatory_structure_break(current, unit):
        emit(current)
        current = unit
    elif final_payload_tokens(candidate) > preferred_max:
        emit(current)
        current = unit
    elif semantic_drop(current.last, unit) >= threshold:
        emit(current)
        current = unit
    else:
        current = candidate

emit(current)

for emitted_span:
    assert final_payload_tokens(emitted_span) <= hard_limit
```

`preferred_max` controls retrieval granularity. `hard_limit` comes from the model profile. The algorithm never assumes that filling all 8,192 tokens is desirable.

## 7. Recursive split

```text
split_by_child_blocks
  -> split_by_unicode_sentence_or_line_boundary
  -> split_by_punctuation_boundary
  -> split_by_token_offsets_with_bounded_overlap
  -> fail_closed
```

Every split produces exact source offsets. A token-offset adapter must map offsets back to valid UTF-8/Unicode-scalar boundaries. It may not split inside a code point, grapheme cluster where avoidable, or invalid byte sequence.

## 8. Hierarchical retrieval

Query flow:

```text
query embedding
  -> document/section candidate routing
  -> leaf retrieval
  -> rank fusion or rerank
  -> parent + previous/next expansion
  -> bounded context package
```

The system returns:

- leaf evidence;
- parent heading path;
- bounded previous/next evidence;
- source offsets;
- vector/model/profile provenance;
- exact token and expansion budgets.

This replaces large fixed overlaps with explicit graph restoration.

## 9. Error handling

| Error | Behavior |
|---|---|
| model profile missing/unverified | fail closed before segmentation run |
| tokenizer unavailable | fail closed; do not estimate by characters |
| similarity provider unavailable | continue structure-only and record degradation |
| LLM refinement unavailable | continue without LLM |
| final payload over hard limit | recursive split, then fail closed if impossible |
| invalid source offsets | reject unit and run |
| provider timeout/rate limit | bounded retry/fallback without changing unit identity |
| mixed/unknown language | normal path; metadata may remain unresolved |

No error message echoes source text, API keys, vector values, or provider response bodies.

## 10. Security

- content is untrusted and cannot issue instructions;
- no active content, script, or external resource is executed;
- provider destinations are allowlisted by the host;
- secrets are resolved outside the domain core;
- PII is purpose-bound rather than blanket-masked;
- embedding vectors and payloads are restricted data;
- every derived artifact records source digest, policy, model profile, tokenizer digest, and code commit;
- LLM proposals require exact evidence identifiers and deterministic validation.

## 11. Evaluation design

### 11.1 Corpora

Build a licensed or synthetic benchmark containing:

- Korean/English code switching;
- Chinese and Japanese without whitespace dependence;
- Thai;
- Arabic RTL;
- English, French, German, Turkish, Vietnamese, Indonesian;
- emoji, combining marks, zero-width joiners;
- tables, list hierarchies, code, captions, dialogue;
- long unpunctuated spans;
- duplicated boilerplate and templates;
- translation/revision-linked documents.

Language labels are used only to stratify results.

### 11.2 Gold evidence

Human reviewers annotate:

- mandatory and preferred boundaries;
- self-contained answer spans;
- parent context required to interpret each leaf;
- query-to-evidence relevance;
- duplicate or boilerplate spans.

### 11.3 Baselines

- fixed 800-token / 100-token overlap;
- paragraph-only;
- structure + hard budget;
- structure + budget + dense boundary;
- hierarchy + context restoration.

### 11.4 Metrics

- overflow and silent truncation;
- exact offset recovery;
- boundary precision/recall and span F1;
- Recall@k, nDCG@k, MRR;
- duplicate-hit rate;
- context-restoration success;
- indexing time, query time, tokens, cost, storage;
- mixed-language performance gap with confidence intervals.

### 11.5 Scientific claim gate

A strategy is not promoted because one language or one dataset improves. The report includes uncertainty, per-profile results, failure examples, and practical effect size. A model-specific threshold is versioned and revalidated on model revision.

## 12. Research position

Dense X Retrieval shows that retrieval-unit granularity materially affects retrieval and downstream QA, and that proposition-sized units can outperform passage indexing. Late Chunking shows that short units can lose surrounding context and that models exposing contextual token states can pool chunks after encoding. Unicode Standard Annex #29 defines default grapheme, word, and sentence boundary guidance but does not make semantic or language validity claims.

TEPP therefore:

- treats retrieval granularity as an empirical choice;
- preserves explicit hierarchy for context restoration in P0;
- keeps true late chunking as an adapter-specific experiment;
- does not claim the closed OpenAI embedding API exposes late-chunking internals;
- validates semantic-unit quality with retrieval and human-gold evidence.

## 13. APA 7 references

Chen, T., Wang, H., Chen, S., Yu, W., Ma, K., Zhao, X., Zhang, H., & Yu, D. (2024). Dense X retrieval: What retrieval granularity should we use? In *Proceedings of the 2024 Conference on Empirical Methods in Natural Language Processing* (pp. 15159–15177). Association for Computational Linguistics. https://doi.org/10.18653/v1/2024.emnlp-main.845

Günther, M., Mohr, I., Williams, D. J., Wang, B., & Xiao, H. (2024). *Late chunking: Contextual chunk embeddings using long-context embedding models* (arXiv:2409.04701). arXiv. https://doi.org/10.48550/arXiv.2409.04701

OpenAI. (2026). *Vector embeddings*. OpenAI API documentation. Retrieved August 15, 2026, from https://developers.openai.com/api/docs/guides/embeddings

Unicode Consortium. (2025). *Unicode Standard Annex #29: Unicode text segmentation* (Revision 47, Unicode 17.0.0). https://www.unicode.org/reports/tr29/tr29-47.html