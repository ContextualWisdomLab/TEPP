# Language-Agnostic Semantic Span Embedding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a language-independent, exact-span, model-budgeted, hierarchical semantic-unit pipeline whose final `text-embedding-3-large` payload never exceeds 8,192 `cl100k_base` tokens and whose provider-specific behavior remains behind ports.

**Architecture:** `evidence_core` owns immutable typed source blocks; a new `semantic_preprocessor` crate owns micro-units, model profiles, token budgeting, recursive splitting, span packing, and hierarchy. `tepp_api` owns versioned interchange. Provider, batch, vector-store, and optional LLM behavior enters through narrow adapters.

**Tech Stack:** Rust 1.97.1 workspace, CPU `f64` reference behavior where numeric similarity is computed, `serde` wire DTOs, property/fuzz testing, pinned tokenizer adapter, optional contextual-orchestrator/pg-llm-batch/semantic-data-portal ports.

## Global Constraints

- Language identity MUST NOT select the base segmentation, packing, overflow, or fallback algorithm.
- TF-IDF, BM25, stopword deletion, stemming, morphology, and translation MUST NOT be required by the base pipeline.
- The complete rendered payload MUST be token-counted and MUST NOT exceed the model profile.
- The initial `text-embedding-3-large` profile uses `max_input_tokens = 8192`, `tokenizer_profile = cl100k_base`, and `default_dimensions = 3072`.
- Silent truncation is prohibited.
- Exact byte and Unicode-scalar source offsets are preserved.
- Database objects use two-or-more-word `snake_case` names and remain in third normal form.
- Production statement and branch coverage and public/safety docstrings remain 100%.
- LLM live tests use `NVIDIA_NIM_API_KEY`; `COPILOT_GITHUB_TOKEN` is prohibited.
- No task may claim production readiness, cross-language equivalence, or retrieval superiority without the defined evidence.

---

### Task 1: Replace paragraph-only scope with accepted contracts

**Files:**
- Create: `docs/adr/0017-language-agnostic-semantic-span-budgeting.md`
- Create: `docs/product/prd-v0.5-proposed.md`
- Create: `docs/superpowers/specs/2026-08-15-language-agnostic-semantic-span-embedding-design.md`
- Modify: `docs/adr/README.md`
- Modify: `DOCUMENTATION.md`
- Modify: `docs/TRACEABILITY.md`
- Modify: `CHANGELOG.md`
- Delete from superseded branch only: `crates/evidence_core/src/semantic.rs`
- Delete from superseded branch only: `crates/evidence_core/tests/semantic_unit_contract.rs`

**Interfaces:**
- Consumes: approved PRD v0.4, ADR 0004, ADR 0008, ADR 0011, ADR 0012.
- Produces: ADR 0017 and proposed PRD v0.5 requirements for all later tasks.

- [ ] **Step 1: Add the documentation-contract test cases**

Extend the documentation validator fixture so ADR 0017 must appear exactly once and every required section is present.

- [ ] **Step 2: Run the documentation validator and confirm RED**

Run:

```bash
python3 scripts/validate_documentation.py
```

Expected: failure because ADR 0017 and the index row are absent.

- [ ] **Step 3: Add ADR, PRD delta, design, index, traceability, and changelog**

Use the exact documents approved in the design PR. Mark decision status and implementation maturity independently.

- [ ] **Step 4: Remove paragraph-only production claims**

Remove the `semantic_paragraph_units` / `refuse_document_bag_of_words` implementation and its claim rows from the superseded PR branch. Paragraph parsing returns later as one `SourceBlockKind`, not the semantic-unit authority.

- [ ] **Step 5: Verify documentation contracts**

Run:

```bash
python3 scripts/validate_documentation.py
python3 scripts/check_docstrings.py
python3 -m coverage run --branch -m unittest discover -s tests/quality -p 'test_*.py'
python3 -m coverage report --fail-under=100 --show-missing
```

Expected: all pass, including 100% statement and branch coverage for quality scripts.

- [ ] **Step 6: Commit**

```bash
git add docs DOCUMENTATION.md CHANGELOG.md crates/evidence_core
git commit -m "docs(evidence): define language-agnostic semantic spans"
```

### Task 2: Add typed source blocks to `evidence_core`

**Files:**
- Create: `crates/evidence_core/src/block.rs`
- Modify: `crates/evidence_core/src/lib.rs`
- Modify: `crates/evidence_core/src/error.rs`
- Test: `crates/evidence_core/tests/source_block_contract.rs`

**Interfaces:**
- Consumes: `DocumentRecord`, `SourceSpan`, `EvidenceId`.
- Produces:
  - `SourceBlockKind`;
  - `SourceBlock::new(document, kind, span, parent, ordinal)`;
  - accessors for block identity, kind, span, parent, ordinal.

- [ ] **Step 1: Write failing exact-block tests**

Cover headings, paragraph, list, table-row group, code, caption, dialogue, DOM, and unknown blocks. Assert cross-document parents, duplicate ordinals, zero-length spans, and invalid ownership fail closed.

- [ ] **Step 2: Run focused tests and confirm RED**

```bash
cargo test -p evidence_core --test source_block_contract --offline
```

Expected: compile failure for missing `SourceBlock` APIs.

- [ ] **Step 3: Implement `SourceBlockKind` and `SourceBlock`**

Store validated domain fields privately. Do not store a second authoritative text copy.

- [ ] **Step 4: Add stable typed errors and docstrings**

Add content-redacting errors for cross-document parentage and invalid source-block order.

- [ ] **Step 5: Run focused and full crate checks**

```bash
cargo fmt --all -- --check
cargo clippy -p evidence_core --all-targets --offline -- -D warnings
cargo test -p evidence_core --offline
cargo llvm-cov -p evidence_core --offline --lib --tests --fail-under-lines 100 --fail-under-regions 100
```

Expected: all pass at 100%.

- [ ] **Step 6: Commit**

```bash
git add crates/evidence_core
git commit -m "feat(evidence): add typed exact-span source blocks"
```

### Task 3: Create embedding model profiles and token-count port

**Files:**
- Create: `crates/semantic_preprocessor/Cargo.toml`
- Create: `crates/semantic_preprocessor/src/lib.rs`
- Create: `crates/semantic_preprocessor/src/profile.rs`
- Create: `crates/semantic_preprocessor/src/token.rs`
- Create: `crates/semantic_preprocessor/tests/model_profile_contract.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

**Interfaces:**
- Consumes: `EvidenceId`, `ContentDigest` from `evidence_core`.
- Produces:
  - `EmbeddingModelProfile`;
  - `EmbeddingInputRole`;
  - `TokenCounter` trait;
  - `TokenOffset`;
  - typed `SemanticPreprocessorError`.

- [ ] **Step 1: Write failing model-profile tests**

Assert:

```rust
assert_eq!(profile.max_input_tokens(), 8192);
assert_eq!(profile.tokenizer_profile(), "cl100k_base");
assert_eq!(profile.default_dimensions(), 3072);
```

Also reject zero limits, zero dimensions, empty provider/model/revision, noncanonical tokenizer digests, and requested dimensions above the default unless the profile explicitly allows them.

- [ ] **Step 2: Write failing fake token-counter tests**

The deterministic fake counter maps Unicode scalar boundaries to token offsets and refuses offsets that split UTF-8.

- [ ] **Step 3: Run focused tests and confirm RED**

```bash
cargo test -p semantic_preprocessor --test model_profile_contract --offline
```

Expected: crate or API missing.

- [ ] **Step 4: Implement profile and port with no provider SDK**

Keep model facts in constructors/fixtures and immutable DTOs. Domain code depends only on the trait.

- [ ] **Step 5: Run crate quality gates**

```bash
cargo fmt --all -- --check
cargo clippy -p semantic_preprocessor --all-targets --offline -- -D warnings
cargo test -p semantic_preprocessor --offline
cargo llvm-cov -p semantic_preprocessor --offline --lib --tests --fail-under-lines 100 --fail-under-regions 100
```

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock crates/semantic_preprocessor
git commit -m "feat(semantic): add versioned embedding model profiles"
```

### Task 4: Build language-independent micro-units

**Files:**
- Create: `crates/semantic_preprocessor/src/micro_unit.rs`
- Create: `crates/semantic_preprocessor/src/unicode_boundary.rs`
- Test: `crates/semantic_preprocessor/tests/micro_unit_contract.rs`
- Test: `crates/semantic_preprocessor/tests/fixtures/mixed_script_cases.json`

**Interfaces:**
- Consumes: ordered `SourceBlock` values.
- Produces:
  - `MicroUnit`;
  - `MicroUnitBuilder::build(&[SourceBlock])`;
  - boundary reasons with exact offsets.

- [ ] **Step 1: Write failing mixed-script tests**

Include Korean/English code switching, Chinese, Japanese, Thai, Arabic RTL, emoji ZWJ, combining marks, code, table cells, and long unpunctuated text. Call the same builder without a language argument.

- [ ] **Step 2: Add a static source check for language branching**

The quality test scans `semantic_preprocessor` production source and fails on a base API parameter or branch named `language_code`, `locale_code`, or equivalent selector in micro-unit/packing modules. Metadata DTOs are excluded from this narrow check.

- [ ] **Step 3: Run focused tests and confirm RED**

```bash
cargo test -p semantic_preprocessor --test micro_unit_contract --offline
```

- [ ] **Step 4: Implement structure and Unicode-safe boundary hints**

Use source-block children first, then Unicode-safe sentence/line hints. Preserve unknown content. Do not call morphology, stopword, TF-IDF, BM25, or translation logic.

- [ ] **Step 5: Add property tests**

Generate arbitrary valid Unicode strings and assert monotonic, nonoverlapping, reconstructable offsets and no panics.

- [ ] **Step 6: Run quality gates and commit**

```bash
cargo fmt --all -- --check
cargo clippy -p semantic_preprocessor --all-targets --offline -- -D warnings
cargo test -p semantic_preprocessor --offline
cargo llvm-cov -p semantic_preprocessor --offline --lib --tests --fail-under-lines 100 --fail-under-regions 100
git add crates/semantic_preprocessor
git commit -m "feat(semantic): build language-independent micro-units"
```

### Task 5: Enforce final-payload budgets and recursive recovery

**Files:**
- Create: `crates/semantic_preprocessor/src/payload.rs`
- Create: `crates/semantic_preprocessor/src/budget.rs`
- Create: `crates/semantic_preprocessor/src/split.rs`
- Test: `crates/semantic_preprocessor/tests/budget_contract.rs`

**Interfaces:**
- Consumes: `MicroUnit`, `EmbeddingModelProfile`, `TokenCounter`.
- Produces:
  - `PayloadTemplate`;
  - `RenderedEmbeddingPayload`;
  - `SemanticSpanPolicy`;
  - `BudgetedUnit`;
  - `RecursiveSplitter`.

- [ ] **Step 1: Write the 8,192/8,193 boundary tests**

Use the fake token counter to assert the complete metadata-rendered payload at 8,192 tokens is accepted and 8,193 is split or refused before a provider call.

- [ ] **Step 2: Write recursive recovery tests**

Cover child block, sentence/line, punctuation, token-offset fallback, bounded overlap, and an unsplittable invalid-offset case.

- [ ] **Step 3: Run focused tests and confirm RED**

```bash
cargo test -p semantic_preprocessor --test budget_contract --offline
```

- [ ] **Step 4: Implement deterministic payload rendering**

Version the template, omit absent metadata, preserve source content, and hash the exact rendered bytes.

- [ ] **Step 5: Implement recursive splitting and typed errors**

Never truncate. Token-offset fallback maps only to valid source boundaries.

- [ ] **Step 6: Run quality gates and commit**

```bash
cargo fmt --all -- --check
cargo clippy -p semantic_preprocessor --all-targets --offline -- -D warnings
cargo test -p semantic_preprocessor --offline
cargo llvm-cov -p semantic_preprocessor --offline --lib --tests --fail-under-lines 100 --fail-under-regions 100
git add crates/semantic_preprocessor
git commit -m "feat(semantic): enforce final embedding payload budgets"
```

### Task 6: Add semantic packing and explicit degradation

**Files:**
- Create: `crates/semantic_preprocessor/src/similarity.rs`
- Create: `crates/semantic_preprocessor/src/packer.rs`
- Test: `crates/semantic_preprocessor/tests/semantic_packer_contract.rs`

**Interfaces:**
- Consumes: `BudgetedUnit`, `AdjacentSimilarity`, `SemanticSpanPolicy`.
- Produces:
  - `SemanticSpanPacker`;
  - `BoundaryMode`;
  - `BoundaryReason`;
  - packed leaf units.

- [ ] **Step 1: Write failing structure-only tests**

Mandatory heading/table/code boundaries split deterministically. Similarity is not required.

- [ ] **Step 2: Write failing dense-boundary tests**

A fake similarity port returns known adjacent scores. Assert a sharp semantic drop creates a boundary without a language feature.

- [ ] **Step 3: Write failing degradation tests**

When similarity returns `Unavailable`, assert packing succeeds in `StructureOnly` mode and records the reason. Invalid scores (`NaN`, infinity, wrong length) fail closed.

- [ ] **Step 4: Run focused tests and confirm RED**

```bash
cargo test -p semantic_preprocessor --test semantic_packer_contract --offline
```

- [ ] **Step 5: Implement the packer**

Recount the final payload after every accepted merge and again before emit. Do not average embeddings from different model spaces.

- [ ] **Step 6: Run quality gates and commit**

```bash
cargo fmt --all -- --check
cargo clippy -p semantic_preprocessor --all-targets --offline -- -D warnings
cargo test -p semantic_preprocessor --offline
cargo llvm-cov -p semantic_preprocessor --offline --lib --tests --fail-under-lines 100 --fail-under-regions 100
git add crates/semantic_preprocessor
git commit -m "feat(semantic): pack spans with explicit fallback"
```

### Task 7: Build leaf/section/document hierarchy and wire DTOs

**Files:**
- Create: `crates/semantic_preprocessor/src/hierarchy.rs`
- Create: `crates/semantic_preprocessor/tests/hierarchy_contract.rs`
- Create: `crates/tepp_api/src/semantic_span.rs`
- Create: `schemas/semantic-span-manifest-v1.schema.json`
- Create: `examples/semantic-span-manifest-v1.json`
- Modify: `crates/tepp_api/src/lib.rs`
- Test: `crates/tepp_api/tests/semantic_span_wire_contract.rs`

**Interfaces:**
- Consumes: packed leaf units, source-block tree, model profile.
- Produces:
  - `SemanticUnitHierarchy`;
  - `SemanticSpanManifestV1`;
  - JSON schema and strict wire reconstruction.

- [ ] **Step 1: Write failing hierarchy tests**

Assert one parent per leaf, same-document ownership, acyclic parent links, exact previous/next symmetry, stable order, and bounded parent/neighbor expansion.

- [ ] **Step 2: Write failing wire tests**

Reject unknown fields, unsupported versions, cross-document relations, wrong payload digests, unknown model profiles, and vector records without `embedding_space_id`.

- [ ] **Step 3: Run focused tests and confirm RED**

```bash
cargo test -p semantic_preprocessor --test hierarchy_contract --offline
cargo test -p tepp_api --test semantic_span_wire_contract --offline
```

- [ ] **Step 4: Implement hierarchy and DTOs**

Summaries are optional derived artifacts. They cannot replace source spans or become source truth.

- [ ] **Step 5: Validate schema and run quality gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace --all-features
python3 scripts/validate_documentation.py
```

- [ ] **Step 6: Commit**

```bash
git add crates/semantic_preprocessor crates/tepp_api schemas examples
git commit -m "feat(api): add semantic span hierarchy contracts"
```

### Task 8: Add provider adapters without moving authority

**Files:**
- Create: `crates/tepp_api/src/embedding_port.rs`
- Create: `docs/connectors/pg-llm-batch-semantic-spans.md`
- Create: `docs/connectors/contextual-orchestrator-semantic-spans.md`
- Create: `docs/connectors/semantic-data-portal-semantic-spans.md`
- Create: `docs/connectors/embedrelay-semantic-spans.md`
- Test: `crates/tepp_api/tests/embedding_port_contract.rs`

**Interfaces:**
- Consumes: validated manifest and rendered payload.
- Produces:
  - provider-neutral `EmbeddingPort`;
  - token-count adapter contract;
  - optional LLM refinement contract;
  - vector-store handoff;
  - embedding-migration handoff.

- [ ] **Step 1: Write failing port tests**

Assert no API accepts provider credentials in DTOs, no adapter can change unit identity, and vector results require profile, payload hash, dimensions, and `embedding_space_id`.

- [ ] **Step 2: Run focused tests and confirm RED**

```bash
cargo test -p tepp_api --test embedding_port_contract --offline
```

- [ ] **Step 3: Implement ports and connector documents**

Keep all provider SDKs out of domain crates. Use HTTPS and host authorization boundaries in actual service adapters.

- [ ] **Step 4: Add deterministic mock integration**

Run source blocks through fake token count, fake similarity, fake embedding, and manifest export. Assert provider failure does not mutate span identity.

- [ ] **Step 5: Run quality gates and commit**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace --all-features
git add crates/tepp_api docs/connectors
git commit -m "feat(api): add semantic embedding ecosystem ports"
```

### Task 9: Build realistic retrieval and robustness evaluation

**Files:**
- Create: `crates/tepp_simulation/src/semantic_span_truth.rs`
- Create: `crates/validation_core/src/retrieval.rs`
- Create: `crates/validation_core/src/span_boundary.rs`
- Create: `crates/validation_core/tests/semantic_span_recovery.rs`
- Create: `docs/validation/semantic-span-benchmark.md`
- Create: `.github/workflows/semantic-span-study.yml`

**Interfaces:**
- Consumes: policies, gold source blocks, queries, relevant spans, hierarchy, retrieval results.
- Produces:
  - overflow/truncation report;
  - boundary precision/recall/span F1;
  - Recall@k, nDCG@k, MRR;
  - duplicate/context-restoration metrics;
  - stratified uncertainty report.

- [ ] **Step 1: Write deterministic truth-corpus recovery tests**

Generate known boundaries and query relevance under mixed scripts, duplicated boilerplate, translation/revision groups, and long-unit stress.

- [ ] **Step 2: Write metric tests with hand-calculated examples**

Cover ties, zero relevant documents, duplicate hits, malformed ranks, and confidence intervals.

- [ ] **Step 3: Run focused tests and confirm RED**

```bash
cargo test -p validation_core --test semantic_span_recovery --offline
```

- [ ] **Step 4: Implement metrics in Rust**

Use stable `f64` calculations and content-redacting reports. Do not infer quality from cosine score alone.

- [ ] **Step 5: Add scheduled live study**

The workflow uses `NVIDIA_NIM_API_KEY` only when an approved provider-backed evaluation is enabled. It never uses `COPILOT_GITHUB_TOKEN`, never skips a required live lane silently, and publishes immutable evaluation artifacts.

- [ ] **Step 6: Run full acceptance suite**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace --all-features
cargo llvm-cov --workspace --all-features --fail-under-lines 100 --fail-under-regions 100
python3 scripts/check_workspace_contract.py
python3 scripts/check_docstrings.py
python3 scripts/validate_documentation.py
cargo deny check
```

Expected: all local deterministic gates pass; live study remains a separate exact-head evidence lane.

- [ ] **Step 7: Commit**

```bash
git add crates/tepp_simulation crates/validation_core docs/validation .github/workflows
git commit -m "test(semantic): validate retrieval and boundary recovery"
```

### Task 10: Final evidence, review, and claim boundary

**Files:**
- Modify: `docs/TRACEABILITY.md`
- Modify: `docs/validation/temporal-event-foundation.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/research/standards-and-literature.md`
- Generate: release SBOM/provenance/checksum artifacts through existing tooling.

**Interfaces:**
- Consumes: exact-head deterministic and live evaluation results.
- Produces: reviewable implementation/scientific maturity decision.

- [ ] **Step 1: Update traceability with exact evidence**

Mark each capability `implemented-main`, `active-PR`, `partial`, or `accepted-target` truthfully.

- [ ] **Step 2: Run release evidence tools**

```bash
python3 scripts/release_evidence.py generate
python3 scripts/release_evidence.py validate
cargo deny check
```

- [ ] **Step 3: Run all exact-head checks**

Do not substitute earlier-head, local-only, queued, skipped, or cancelled evidence for required current-head checks.

- [ ] **Step 4: Obtain independent review**

Resolve actionable review threads, re-run affected tests, and require an independent approval under ADR 0015.

- [ ] **Step 5: Promote claims only when evidence permits**

A merged implementation may claim zero-overflow and exact-span behavior only after the hard gates pass. It may claim retrieval improvement or language-profile robustness only after the benchmark evidence passes under ADR 0014.

- [ ] **Step 6: Commit final evidence updates**

```bash
git add docs CHANGELOG.md
git commit -m "docs(semantic): record semantic span acceptance evidence"
```