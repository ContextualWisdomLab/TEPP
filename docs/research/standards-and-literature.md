# Standards and Research Foundations

This register traces TEPP's methodological and engineering contracts to authoritative standards and primary research. References use APA 7th style. Implementations must link claims, equations, tests, and ADRs to the most specific applicable source.

Implementation-specific doctoring supplements this register. The executable Task 3 timestamp, interval, wire, schema, security, and claim-boundary mapping is documented in [`task-3-temporal-wire-foundations.md`](task-3-temporal-wire-foundations.md).

## Psychometrics and latent-variable modeling

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural equation models. *Structural Equation Modeling: A Multidisciplinary Journal, 25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803

Marsh, H. W., Morin, A. J. S., Parker, P. D., & Kaur, G. (2014). Exploratory structural equation modeling: An integration of the best features of exploratory and confirmatory factor analysis. *Annual Review of Clinical Psychology, 10*, 85–110. https://doi.org/10.1146/annurev-clinpsy-032813-153700

TEPP applies these sources to construct definition, score interpretation, reliability, validity evidence, uncertainty, consequences, longitudinal invariance, ESEM cross-loadings, and DSEM. Topic outputs are treated as fallible indicators or components only after their construct role is evaluated.

## Structural, correlated, dynamic, relational, and multilingual topic models

Blei, D. M., & Lafferty, J. D. (2006). Dynamic topic models. In *Proceedings of the 23rd International Conference on Machine Learning* (pp. 113–120). Association for Computing Machinery. https://doi.org/10.1145/1143844.1143859

Blei, D. M., & Lafferty, J. D. (2007). A correlated topic model of Science. *The Annals of Applied Statistics, 1*(1), 17–35. https://doi.org/10.1214/07-AOAS114

Chang, J., & Blei, D. M. (2009). Relational topic models for document networks. In *Proceedings of the 12th International Conference on Artificial Intelligence and Statistics* (pp. 81–88). PMLR.

Mimno, D., Wallach, H. M., Naradowsky, J., Smith, D. A., & McCallum, A. (2009). Polylingual topic models. In *Proceedings of the 2009 Conference on Empirical Methods in Natural Language Processing* (pp. 880–889). Association for Computational Linguistics.

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for structural topic models. *Journal of Statistical Software, 91*(2), 1–40. https://doi.org/10.18637/jss.v091.i02

Roberts, M. E., Stewart, B. M., Tingley, D., Lucas, C., Leder-Luis, J., Gadarian, S. K., Albertson, B., & Rand, D. G. (2014). Structural topic models for open-ended survey responses. *American Journal of Political Science, 58*(4), 1064–1082. https://doi.org/10.1111/ajps.12103

Bianchi, F., Terragni, S., Hovy, D., Nozza, D., & Fersini, E. (2021). Cross-lingual contextualized topic models with zero-shot learning. In *Proceedings of the 16th Conference of the European Chapter of the Association for Computational Linguistics* (pp. 1676–1683). Association for Computational Linguistics. https://doi.org/10.18653/v1/2021.eacl-main.143

Nguyen, T. P., Minh, N. V., Nguyen, T., Van, L. N., Nguyen, D. A., Sang, D. V., & Le, T. (2025). XTRA: Cross-lingual topic modeling with topic and representation alignments. In *Findings of the Association for Computational Linguistics: EMNLP 2025*. Association for Computational Linguistics.

TEPP retains a logistic-normal CPU reference while allowing adapter backends that satisfy shared-latent, posterior, temporal, relational, and measurement-invariance contracts.

## Topic-model evaluation and LLM judges

Chang, J., Gerrish, S., Wang, C., Boyd-Graber, J. L., & Blei, D. M. (2009). Reading tea leaves: How humans interpret topic models. In *Advances in Neural Information Processing Systems 22*.

Mimno, D., Wallach, H. M., Talley, E., Leenders, M., & McCallum, A. (2011). Optimizing semantic coherence in topic models. In *Proceedings of the 2011 Conference on Empirical Methods in Natural Language Processing* (pp. 262–272). Association for Computational Linguistics.

Stammbach, D., Zouhar, V., Hoyle, A., Sachan, M., & Ash, E. (2023). Revisiting automated topic model evaluation with large language models. In *Proceedings of the 2023 Conference on Empirical Methods in Natural Language Processing* (pp. 9348–9357). Association for Computational Linguistics. https://doi.org/10.18653/v1/2023.emnlp-main.581

Yang, X., Zhao, H., Phung, D., Buntine, W., & Du, L. (2025). LLM reading tea leaves: Automatically evaluating topic models with large language models. *Transactions of the Association for Computational Linguistics, 13*.

LLM evaluation complements but never replaces predictive, posterior, stability, alignment, fairness, recovery, and human-validation evidence. Candidates are blinded and statistically gated before LLM review.

## Compositional data, correlation, and clusters

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Friedman, J., Hastie, T., & Tibshirani, R. (2008). Sparse inverse covariance estimation with the graphical lasso. *Biostatistics, 9*(3), 432–441. https://doi.org/10.1093/biostatistics/kxm045

Traag, V. A., Waltman, L., & van Eck, N. J. (2019). From Louvain to Leiden: Guaranteeing well-connected communities. *Scientific Reports, 9*, Article 5233. https://doi.org/10.1038/s41598-019-41695-z

Raw topic proportions are not ordinary Euclidean measurements. TEPP uses logistic-normal or orthonormal log-ratio coordinates and reports posterior and resampling uncertainty for every network edge and cluster.

## Time, events, and topic detection and tracking

International Organization for Standardization. (2012). *Language resource management—Semantic annotation framework (SemAF)—Part 1: Time and events (SemAF-Time, ISO-TimeML)* (ISO Standard No. 24617-1:2012).

International Organization for Standardization. (2019a). *Date and time—Representations for information interchange—Part 1: Basic rules* (ISO Standard No. 8601-1:2019). https://www.iso.org/standard/70907.html

International Organization for Standardization. (2019b). *Date and time—Representations for information interchange—Part 2: Extensions* (ISO Standard No. 8601-2:2019). https://www.iso.org/standard/70908.html

Klyne, G., & Newman, C. (2002). *Date and time on the Internet: Timestamps* (RFC 3339). RFC Editor. https://doi.org/10.17487/RFC3339

Sharma, U., & Bormann, C. (2024). *Date and time on the Internet: Timestamps with additional information* (RFC 9557). RFC Editor. https://doi.org/10.17487/RFC9557

Hobbs, J. R., & Pan, F. (2017). *Time ontology in OWL* (W3C Recommendation). World Wide Web Consortium. https://www.w3.org/TR/owl-time/

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Papadakis, N., Stravoskoufos, K., Baratis, E., & Plexousakis, D. (2013). CHRONOS: A reasoner for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 502–511. https://doi.org/10.1016/j.procs.2013.09.130

TEPP Task 3 uses a deliberately narrow, versioned RFC 3339 profile for validated instants and typed uncertain intervals. RFC 9557 and ISO 8601-2 extensions are tracked but are not silently accepted in wire version `1`. Interval and partial-order reasoning, bitemporal availability, leakage-safe cutoffs, TDT segmentation/link/detection/first-story/tracking tasks, and separate neural/symbolic event-schema and temporal-consistency layers remain explicit later contracts.

## Unicode, language tags, and multilingual structure

Davis, M., Iancu, L., & Whistler, K. (Eds.). (2024). *Unicode Standard Annex #15: Unicode normalization forms*. Unicode Consortium.

Davis, M., Iancu, L., & Whistler, K. (Eds.). (2024). *Unicode Standard Annex #29: Unicode text segmentation*. Unicode Consortium.

Phillips, A., & Davis, M. (2009). *Tags for identifying languages* (RFC 5646). Internet Engineering Task Force. https://doi.org/10.17487/RFC5646

Nivre, J., de Marneffe, M.-C., Ginter, F., Hajič, J., Manning, C. D., Pyysalo, S., Schuster, S., Tyers, F., & Zeman, D. (2020). Universal Dependencies v2: An evergrowing multilingual treebank collection. In *Proceedings of the 12th Language Resources and Evaluation Conference* (pp. 4034–4043). European Language Resources Association.

The original source is preserved. NFC is used for canonical analysis views; compatibility normalization is limited to explicit auxiliary keys. Segmentation and morphology are language-tailored. Universal POS informs source priors rather than irreversible deletion.

## Evidence identity, hashing, and interchange

Bray, T. (Ed.). (2017). *The JavaScript Object Notation (JSON) data interchange format* (RFC 8259). RFC Editor. https://doi.org/10.17487/RFC8259

Hutton, B., Andrews, H., Wright, A., & Dennis, G. (2022). *JSON Schema: A media type for describing JSON documents* (Draft 2020-12). JSON Schema. https://json-schema.org/draft/2020-12/json-schema-core.html

Wright, A., Hutton, B., Andrews, H., & Dennis, G. (2022). *JSON Schema validation: A vocabulary for structural validation of JSON* (Draft 2020-12). JSON Schema. https://json-schema.org/draft/2020-12/json-schema-validation.html

Davis, K., Peabody, B., & Leach, P. (2024). *Universally unique identifier (UUID)* (RFC 9562). RFC Editor. https://doi.org/10.17487/RFC9562

National Institute of Standards and Technology. (2015). *Secure Hash Standard (SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4

Yergeau, F. (2003). *UTF-8, a transformation format of ISO 10646* (RFC 3629). RFC Editor. https://doi.org/10.17487/RFC3629

Lebo, T., Sahoo, S., & McGuinness, D. (Eds.). (2013). *PROV-O: The PROV ontology*. World Wide Web Consortium. https://www.w3.org/TR/prov-o/

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data model*. World Wide Web Consortium. https://www.w3.org/TR/prov-dm/

TEPP separates stable record identity, content equality, exact text location, wire representation, authorization, and provenance. JSON wire records are explicit versioned DTOs with unknown-field rejection and reconstruct through domain validation. JSON Schema records structural and lexical constraints but never substitutes for Rust semantic validation. `SHA-256` detects content substitution but is not treated as proof of origin, authority, or chain of custody.

## AI risk, security, accessibility, and software supply chain

National Institute of Standards and Technology. (2023). *Artificial intelligence risk management framework (AI RMF 1.0)* (NIST AI 100-1). https://doi.org/10.6028/NIST.AI.100-1

National Institute of Standards and Technology. (2024). *Artificial intelligence risk management framework: Generative artificial intelligence profile* (NIST AI 600-1). https://doi.org/10.6028/NIST.AI.600-1

World Wide Web Consortium. (2023). *Web content accessibility guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/

OpenSSF. (2023). *Supply-chain Levels for Software Artifacts (SLSA) specification, version 1.0*. https://slsa.dev/spec/v1.0/

TEPP treats documents and model output as untrusted, requires exact evidence and fail-closed validation, supplies accessible exact-value alternatives to graphics, and emits SBOM and provenance evidence for releases.

## LLM orchestration research register

Fugu-, Conductor-, and TRINITY-style orchestration claims require a dedicated literature-review ADR before production implementation. The implementation study must identify the exact primary papers and versions, compare direct routing with deeper role-based orchestration, vary reasoning effort, decomposition, recursion, workflow stages, and access lists, and record accuracy, calibration, disagreement, token use, cost, and failure modes. No ambiguous project name is treated as a verified citation.
