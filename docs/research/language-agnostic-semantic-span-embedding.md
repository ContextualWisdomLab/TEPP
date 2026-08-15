# Language-agnostic semantic-span embedding research traceability

## Decision summary

TEPP should not ask “what language is this?” before it can preserve evidence, form candidate meaning units, or enforce an embedding-model context limit. Source structure, Unicode-safe offsets, final rendered-payload token counts, optional dense semantic changes, and explicit hierarchy provide the base contract.

Language metadata remains important for measurement invariance, fairness, lexical analysis, and result stratification. It is not the switch that decides whether core processing works.

## Evidence-to-decision map

| Evidence | Supported decision | Claim boundary |
|---|---|---|
| OpenAI model documentation and the January 2024 embedding-model announcement | `text-embedding-3-large` remains an embeddings model; its full output is up to 3,072 dimensions and the `dimensions` request parameter may shorten the vector | Provider documentation, not a universal vector length or evidence that every shortened dimension has equal retrieval quality |
| Current OpenAI Python SDK embeddings resource | The documented per-input maximum for embedding models is 8,192 tokens; the complete request also has a separate aggregate-token limit | A mutable SDK/API contract. TEPP stores the source URL, retrieval date, and artifact digest and may choose an operational hard limit below the provider maximum |
| OpenAI `tiktoken` model mapping and token-counting cookbook | `text-embedding-3-large` maps to `cl100k_base`; `encoding_for_model` is preferred over scattering the encoding name through product code | Tokenization is model/revision profile data. A provider or tokenizer revision requires a new verified profile rather than silently changing an existing one |
| Unicode Standard Annex #29, Revision 47 | Unicode-safe default grapheme/word/sentence boundary guidance | Default boundaries are not semantic truth and are not sufficient for every script or locale |
| Chen et al. (2024), Dense X Retrieval | Retrieval-unit granularity affects retrieval and QA; proposition-scale units are an empirical alternative to passages | Does not prove one granularity is best for every corpus, language, or embedding model |
| Günther et al. (2024), Late Chunking | Independent short chunks can lose global context; token-level long-context models can pool contextualized chunk representations | The OpenAI embeddings API does not expose contextual token states/pooling, so TEPP does not claim true late chunking for `text-embedding-3-large` |
| TEPP approved PRD v0.4 | Exact source spans, multilingual evidence, optional LLM semantic units, hierarchical/relational evidence, no TF-IDF/BM25 inferential weighting | The approved baseline does not by itself prove the new segmentation implementation |

## Provider-profile verification rule

The model profile distinguishes the provider-documented ceiling from TEPP's operational hard limit. The latter must be less than or equal to the documented ceiling and may reserve a safety margin. Unit and property tests use a deterministic token-counter port to prove exact boundary behavior. A scheduled live provider probe may corroborate the external contract, but a transient provider call is not the only authority for offline segmentation correctness. If the provider rejects a payload that the current profile permits, the adapter fails closed, records the divergence, and requires a new verified profile or a lower operational limit; it does not silently truncate or mutate existing span identity.

## Design implications

1. **No language gate.** Missing, mixed, or wrong language metadata cannot cause a different base algorithm.
2. **No character-count fallback.** Token limits are model/tokenizer facts.
3. **Count the final payload.** Metadata and separators consume tokens.
4. **Use hierarchy, not only overlap.** Leaves retrieve precisely; parent and neighbor relations restore context.
5. **Treat granularity as an empirical parameter.** Compare fixed, paragraph, structural, semantic, and hierarchical alternatives.
6. **Keep late chunking experimental by model capability.** It requires token-level contextual states and pooling control.
7. **Do not use TF-IDF or BM25 in boundary scoring.** Dense similarity is optional; structure-only remains deterministic.
8. **Preserve source evidence.** Summaries and embeddings are derived artifacts.
9. **Version external facts.** Provider limit, tokenizer mapping, dimensions, source URL, retrieval date, and artifact digest belong to an immutable profile.

## APA 7 references

Chen, T., Wang, H., Chen, S., Yu, W., Ma, K., Zhao, X., Zhang, H., & Yu, D. (2024). Dense X retrieval: What retrieval granularity should we use? In *Proceedings of the 2024 Conference on Empirical Methods in Natural Language Processing* (pp. 15159–15177). Association for Computational Linguistics. https://doi.org/10.18653/v1/2024.emnlp-main.845

Günther, M., Mohr, I., Williams, D. J., Wang, B., & Xiao, H. (2024). *Late chunking: Contextual chunk embeddings using long-context embedding models* (arXiv:2409.04701). arXiv. https://doi.org/10.48550/arXiv.2409.04701

OpenAI. (2024, January 25). *New embedding models and API updates*. https://openai.com/index/new-embedding-models-and-api-updates/

OpenAI. (2026). *Embeddings resource* [Source code]. GitHub. Retrieved August 15, 2026, from https://github.com/openai/openai-python/blob/main/src/openai/resources/embeddings.py

OpenAI. (2026). *How to count tokens with tiktoken* [Jupyter Notebook]. OpenAI Cookbook. Retrieved August 15, 2026, from https://github.com/openai/openai-cookbook/blob/main/examples/How_to_count_tokens_with_tiktoken.ipynb

OpenAI. (2026). *Model-to-encoding mapping* [Source code]. GitHub. Retrieved August 15, 2026, from https://github.com/openai/tiktoken/blob/main/tiktoken/model.py

OpenAI. (2026). *text-embedding-3-large model*. OpenAI API documentation. Retrieved August 15, 2026, from https://developers.openai.com/api/docs/models/text-embedding-3-large

Unicode Consortium. (2025). *Unicode Standard Annex #29: Unicode text segmentation* (Revision 47, Unicode 17.0.0). https://www.unicode.org/reports/tr29/tr29-47.html
