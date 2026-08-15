# Language-agnostic semantic-span embedding research traceability

## Decision summary

TEPP should not ask “what language is this?” before it can preserve evidence, form candidate meaning units, or enforce an embedding-model context limit. Source structure, Unicode-safe offsets, final rendered-payload token counts, optional dense semantic changes, and explicit hierarchy provide the base contract.

Language metadata remains important for measurement invariance, fairness, lexical analysis, and result stratification. It is not the switch that decides whether core processing works.

## Evidence-to-decision map

| Evidence | Supported decision | Claim boundary |
|---|---|---|
| OpenAI embeddings documentation | `text-embedding-3-large` initial profile: 8,192-token maximum, 3,072 default dimensions, `dimensions` support; third-generation token estimation uses `cl100k_base` | Provider documentation, not a universal model limit or a guarantee that the model never changes |
| Unicode Standard Annex #29, Revision 47 | Unicode-safe default grapheme/word/sentence boundary guidance | Default boundaries are not semantic truth and are not sufficient for every script or locale |
| Chen et al. (2024), Dense X Retrieval | Retrieval-unit granularity affects retrieval and QA; proposition-scale units are an empirical alternative to passages | Does not prove one granularity is best for every corpus, language, or embedding model |
| Günther et al. (2024), Late Chunking | Independent short chunks can lose global context; token-level long-context models can pool contextualized chunk representations | The OpenAI embeddings API does not expose contextual token states/pooling, so TEPP does not claim true late chunking for `text-embedding-3-large` |
| TEPP approved PRD v0.4 | Exact source spans, multilingual evidence, optional LLM semantic units, hierarchical/relational evidence, no TF-IDF/BM25 inferential weighting | The approved baseline does not by itself prove the new segmentation implementation |

## Design implications

1. **No language gate.** Missing, mixed, or wrong language metadata cannot cause a different base algorithm.
2. **No character-count fallback.** Token limits are model/tokenizer facts.
3. **Count the final payload.** Metadata and separators consume tokens.
4. **Use hierarchy, not only overlap.** Leaves retrieve precisely; parent and neighbor relations restore context.
5. **Treat granularity as an empirical parameter.** Compare fixed, paragraph, structural, semantic, and hierarchical alternatives.
6. **Keep late chunking experimental by model capability.** It requires token-level contextual states and pooling control.
7. **Do not use TF-IDF or BM25 in boundary scoring.** Dense similarity is optional; structure-only remains deterministic.
8. **Preserve source evidence.** Summaries and embeddings are derived artifacts.

## APA 7 references

Chen, T., Wang, H., Chen, S., Yu, W., Ma, K., Zhao, X., Zhang, H., & Yu, D. (2024). Dense X retrieval: What retrieval granularity should we use? In *Proceedings of the 2024 Conference on Empirical Methods in Natural Language Processing* (pp. 15159–15177). Association for Computational Linguistics. https://doi.org/10.18653/v1/2024.emnlp-main.845

Günther, M., Mohr, I., Williams, D. J., Wang, B., & Xiao, H. (2024). *Late chunking: Contextual chunk embeddings using long-context embedding models* (arXiv:2409.04701). arXiv. https://doi.org/10.48550/arXiv.2409.04701

OpenAI. (2026). *How can I tell how many tokens a string will have before I try to embed it?* OpenAI Help Center. Retrieved August 15, 2026, from https://help.openai.com/en/articles/8984337-how-can-i-tell-how-many-tokens-a-string-will-have-before-i-try-to-embed-it

OpenAI. (2026). *Vector embeddings*. OpenAI API documentation. Retrieved August 15, 2026, from https://developers.openai.com/api/docs/guides/embeddings

Unicode Consortium. (2025). *Unicode Standard Annex #29: Unicode text segmentation* (Revision 47, Unicode 17.0.0). https://www.unicode.org/reports/tr29/tr29-47.html