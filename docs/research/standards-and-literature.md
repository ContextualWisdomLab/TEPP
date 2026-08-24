# Standards and Research Foundations

This register traces TEPP's methodological, engineering, governance, privacy, and orchestration contracts to authoritative standards and primary research. References use APA 7th style. Implementations must link claims, equations, tests, and ADRs to the most specific applicable source.

## Psychometrics and latent-variable modeling

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural equation models. *Structural Equation Modeling: A Multidisciplinary Journal, 25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling. *Structural Equation Modeling: A Multidisciplinary Journal, 16*(3), 397–438. https://doi.org/10.1080/10705510903008204

Fox, J.-P., & Glas, C. A. W. (2001). Bayesian estimation of a multilevel IRT model using Gibbs sampling. *Psychometrika, 66*(2), 271–288. https://doi.org/10.1007/BF02294839

Marsh, H. W., Morin, A. J. S., Parker, P. D., & Kaur, G. (2014). Exploratory structural equation modeling: An integration of the best features of exploratory and confirmatory factor analysis. *Annual Review of Clinical Psychology, 10*, 85–110. https://doi.org/10.1146/annurev-clinpsy-032813-153700

TEPP applies these sources to construct definition, score interpretation, reliability, validity evidence, uncertainty, consequences, longitudinal invariance, ESEM cross-loadings, DSEM, and multilevel/non-independence measurement. Fox and Glas (2001) support Bayesian multilevel IRT estimation; they are not evidence for every cross-classified or multiple-membership estimator TEPP may later implement. Topic outputs are treated as fallible indicators or components only after their construct role is evaluated. Clustered or cross-classified observations are not flattened into independent rows (Fox & Glas, 2001; American Educational Research Association et al., 2014).

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

Akaike, H. (1974). A new look at the statistical model identification. *IEEE Transactions on Automatic Control, 19*(6), 716–723. https://doi.org/10.1109/TAC.1974.1100705

Burnham, K. P., & Anderson, D. R. (2002). *Model selection and multimodel inference: A practical information-theoretic approach* (2nd ed.). Springer.

Deb, K., Pratap, A., Agarwal, S., & Meyarivan, T. (2002). A fast and elitist multiobjective genetic algorithm: NSGA-II. *IEEE Transactions on Evolutionary Computation, 6*(2), 182–197. https://doi.org/10.1109/4235.996017

LLM evaluation complements but never replaces predictive, posterior, stability,
alignment, fairness, recovery, and human-validation evidence. The current
`model_selection` crate performs statistical/Pareto gating; candidate blinding
and blinded LLM review remain accepted-target extensions and are not executed
by this crate. Pareto-filtered held-out log-likelihood and complexity admit a
candidate `K`; an LLM vote cannot define the numerical optimum.

## Compositional data, correlation, and clusters

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Friedman, J., Hastie, T., & Tibshirani, R. (2008). Sparse inverse covariance estimation with the graphical lasso. *Biostatistics, 9*(3), 432–441. https://doi.org/10.1093/biostatistics/kxm045

Traag, V. A., Waltman, L., & van Eck, N. J. (2019). From Louvain to Leiden: Guaranteeing well-connected communities. *Scientific Reports, 9*, Article 5233. https://doi.org/10.1038/s41598-019-41695-z

Raw topic proportions are not ordinary Euclidean measurements. TEPP uses logistic-normal or orthonormal log-ratio coordinates and reports posterior and resampling uncertainty for every network edge and cluster.

## Time, events, and topic detection and tracking

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434

International Organization for Standardization. (2012). *Language resource management—Semantic annotation framework (SemAF)—Part 1: Time and events (SemAF-Time, ISO-TimeML)* (ISO Standard No. 24617-1:2012).

Hobbs, J. R., & Pan, F. (2017). *Time ontology in OWL* (W3C Recommendation). World Wide Web Consortium. https://www.w3.org/TR/owl-time/

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Li, M., Li, S., Wang, Z., Huang, L., Cho, K., Ji, H., Han, J., & Voss, C. (2021). The future is not one-dimensional: Complex event schema induction by graph modeling for event prediction. In *Proceedings of the 2021 Conference on Empirical Methods in Natural Language Processing* (pp. 5203–5215). Association for Computational Linguistics. https://doi.org/10.18653/v1/2021.emnlp-main.422

TEPP uses Allen interval algebra and partial-order reasoning, ISO-TimeML/OWL-Time vocabulary, bitemporal availability, leakage-safe cutoffs, TDT segmentation/link/detection/first-story/tracking tasks, neural event-schema induction/prediction, and separate symbolic temporal-consistency reasoning (Allen, 1983; International Organization for Standardization, 2012; Hobbs & Pan, 2017; Allan, 2002; Li et al., 2021; Anagnostopoulos et al., 2013).

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434. Interval relations constrain temporal reasoning; they do not make support, contradiction, summary, or `outcome_of` a state transition.

Raudenbush, S. W., & Bryk, A. S. (2002). *Hierarchical linear models: Applications and data analysis methods* (2nd ed.). Sage.

Snijders, T. A. B., & Bosker, R. J. (2012). *Multilevel analysis: An introduction to basic and advanced multilevel modeling* (2nd ed.). SAGE.

Browne, W. J., Goldstein, H., & Rasbash, J. (2001). Multiple membership multiple classification (MMMC) models. *Statistical Modelling, 1*(2), 103–124. https://doi.org/10.1177/1471082X0100100202

Beretvas, S. N. (2011). Cross-classified and multiple-membership models. In J. J. Hox & J. K. Roberts (Eds.), *Handbook of advanced multilevel analysis* (pp. 313–334). Routledge.

Nested ICC recovery uses the unbalanced ANOVA variance-component estimator and refuses to treat cross-classified or multiple-membership designs as one hierarchy (Raudenbush & Bryk, 2002; Snijders & Bosker, 2012; Browne et al., 2001; Beretvas, 2011).

## Unicode, language tags, and multilingual structure

Davis, M., Iancu, L., & Whistler, K. (Eds.). (2024). *Unicode Standard Annex #15: Unicode normalization forms*. Unicode Consortium.

Davis, M., Iancu, L., & Whistler, K. (Eds.). (2024). *Unicode Standard Annex #29: Unicode text segmentation*. Unicode Consortium.

Bird, S., & Liberman, M. (2001). A formal framework for linguistic annotation. *Speech Communication, 33*(1–2), 23–60. https://doi.org/10.1016/S0167-6393(00)00068-6

Wilde, E., & Duerst, M. (2008). *URI fragment identifiers for the text/plain media type* (RFC 5147). Internet Engineering Task Force. https://doi.org/10.17487/RFC5147

Phillips, A., & Davis, M. (2009). *Tags for identifying languages* (RFC 5646). Internet Engineering Task Force. https://doi.org/10.17487/RFC5646

Nivre, J., de Marneffe, M.-C., Ginter, F., Hajič, J., Manning, C. D., Pyysalo, S., Schuster, S., Tyers, F., & Zeman, D. (2020). Universal Dependencies v2: An evergrowing multilingual treebank collection. In *Proceedings of the 12th Language Resources and Evaluation Conference* (pp. 4034–4043). European Language Resources Association.

The original source is preserved. NFC is used for canonical analysis views; compatibility normalization is limited to explicit auxiliary keys. Segmentation and morphology are language-tailored. Universal POS informs source priors rather than irreversible deletion. Persist an exact UTF-8 byte span through `text_segment` SQL when a membership or mention must point at a unit without copying source text (Bird & Liberman, 2001; Wilde & Duerst, 2008; Davis et al., 2024).

## Evidence identity, hashing, and interchange

Bray, T. (Ed.). (2017). *The JavaScript Object Notation (JSON) data interchange format* (RFC 8259). RFC Editor. https://doi.org/10.17487/RFC8259

Davis, K., Peabody, B., & Leach, P. (2024). *Universally unique identifier (UUID)* (RFC 9562). RFC Editor. https://doi.org/10.17487/RFC9562

National Institute of Standards and Technology. (2015). *Secure Hash Standard (SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4

Yergeau, F. (2003). *UTF-8, a transformation format of ISO 10646* (RFC 3629). RFC Editor. https://doi.org/10.17487/RFC3629

Lebo, T., Sahoo, S., & McGuinness, D. (Eds.). (2013). *PROV-O: The PROV ontology*. World Wide Web Consortium. https://www.w3.org/TR/prov-o/

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data model*. World Wide Web Consortium. https://www.w3.org/TR/prov-dm/

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Peng, R. D. (2011). Reproducible research in computational science. *Science, 334*(6060), 1226–1227. https://doi.org/10.1126/science.1213847

TEPP separates stable record identity, content equality, exact text location, wire representation, authorization, and provenance. JSON wire records are explicit versioned DTOs with unknown-field rejection and reconstruct through domain validation. `SHA-256` detects content substitution but is not treated as proof of origin, authority, or chain of custody. A model checkpoint is a derived run artifact whose digest verifies bytes (National Institute of Standards and Technology, 2015); it does not become the CPU `f64` estimator or a scientific claim (Peng, 2011; National Academies of Sciences, Engineering, and Medicine, 2019).

## Privacy lifecycle, retention, and legal hold

European Union. (2016). *Regulation (EU) 2016/679 of the European Parliament and of the Council of 27 April 2016 on the protection of natural persons with regard to the processing of personal data and on the free movement of such data (General Data Protection Regulation)*. Official Journal of the European Union, L 119, 1–88. https://eur-lex.europa.eu/eli/reg/2016/679/oj

National Institute of Standards and Technology. (2020). *NIST privacy framework: A tool for improving privacy through enterprise risk management, version 1.0*. https://doi.org/10.6028/NIST.CSWP.01162020

TEPP uses these sources, together with the AICPA Trust Services Criteria cited below, as readiness inputs for purpose-bound retention, deletion, and legal hold. They are not self-certification authority. Persistence migration `0007` records policy, hold, deletion requests, and evidence tombstones; it does not assert that a deployment is lawful under GDPR Article 17 or attested under SOC 2.

## AI risk, management systems, and assurance readiness

International Organization for Standardization. (2023a). *Information technology—Artificial intelligence—Guidance on risk management* (ISO/IEC Standard No. 23894:2023). https://www.iso.org/standard/77304.html

International Organization for Standardization. (2023b). *Information technology—Artificial intelligence—Management system* (ISO/IEC Standard No. 42001:2023). https://www.iso.org/standard/81230.html

Tabassi, E. (2023). *Artificial intelligence risk management framework (AI RMF 1.0)* (NIST AI 100-1). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.AI.100-1

Autio, C., Schwartz, R., Dunietz, J., Jain, S., Stanley, M., Tabassi, E., Hall, P., & Roberts, K. (2024). *Artificial intelligence risk management framework: Generative artificial intelligence profile* (NIST AI 600-1). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.AI.600-1

American Institute of Certified Public Accountants. (2023). *2017 Trust services criteria for security, availability, processing integrity, confidentiality, and privacy (with revised points of focus—2022)*. AICPA & CIMA.

한국인터넷진흥원. (2025). *2025년 클라우드 서비스 보안인증제 안내서 (2025.02)*. KISA.

National Institute of Standards and Technology. (n.d.). *AI risk management framework*. Retrieved August 11, 2026, from https://www.nist.gov/itl/ai-risk-management-framework

한국인터넷진흥원. (n.d.). *클라우드서비스 보안인증제 제도소개*. Retrieved August 11, 2026, from https://isms.kisa.or.kr/main/csap/intro/index.jsp

## Privacy purpose limitation and provider minimization

ISO/IEC. (2025). *ISO/IEC 27701:2025 Information security, cybersecurity and privacy protection — Privacy information management systems — Requirements and guidance*. International Organization for Standardization.

ISO/IEC. (2019). *ISO/IEC 27701:2019 Security techniques — Extension to ISO/IEC 27001 and ISO/IEC 27002 for privacy information management — Requirements and guidelines*. International Organization for Standardization.

National Institute of Standards and Technology. (2020). *NIST Privacy Framework: A tool for improving privacy through enterprise risk management* (Version 1.0). U.S. Department of Commerce. https://doi.org/10.6028/NIST.CSWP.01162020

TEPP applies ISO/IEC 27701:2025 purpose limitation and disclosure minimization, and the NIST Privacy Framework Control-P / Communicate-P functions, to provider payloads and separately authorized re-identification (ISO/IEC, 2025; National Institute of Standards and Technology, 2020). The 2019 edition is retained for earlier doctoring that treated PIMS as an ISO/IEC 27001 extension (ISO/IEC, 2019). Re-identification audit evidence is bound with FIPS 180-4 SHA-256 over a length-delimited canonical encoding (National Institute of Standards and Technology, 2015). These sources are readiness mappings, not certification.

TEPP uses these sources as management/risk/readiness inputs, not as self-certification authority. ISO/IEC 42001:2023 and ISO/IEC 23894:2023 are published international standards (International Organization for Standardization, 2023a, 2023b). NIST AI RMF 1.0 remains the published framework while NIST is preparing a revision (Tabassi, 2023; National Institute of Standards and Technology, n.d.); the repository tracks the revision but does not silently treat an unpublished successor as normative. AICPA Trust Services Criteria are readiness inputs rather than self-issued attestation (American Institute of Certified Public Accountants, 2023). KISA currently describes CSAP service types as IaaS, SaaS, and DaaS and grades as high, medium, and low, while noting that the high and medium grades await later implementation (한국인터넷진흥원, n.d.). CSAP and SOC 2 evidence depend on actual deployment/organization controls and independent assessment.

## HTTP interchange and timestamp authority

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics* (RFC 9110). IETF. https://doi.org/10.17487/RFC9110

Klyne, G., & Newman, C. (2002). *Date and time on the Internet: Timestamps* (RFC 3339). IETF. https://doi.org/10.17487/RFC3339

TEPP uses RFC 9110 for live `Host` and `Transfer-Encoding` refusal on the naruon loopback listener, and RFC 3339 via `temporal_core::KnowledgeCutoff` so a buyer cannot submit `"k"` or a future-dated cutoff as an analysis-run clock.

## Security, accessibility, and software supply chain

World Wide Web Consortium. (2023). *Web content accessibility guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/

OpenSSF. (2023). *Supply-chain Levels for Software Artifacts (SLSA) specification, version 1.0*. https://slsa.dev/spec/v1.0/

GitHub. (n.d.). *REST API endpoints for workflows*. GitHub Docs. Retrieved August 13, 2026, from https://docs.github.com/en/rest/actions/workflows

OWASP Foundation. (2023). *OWASP Top 10 CI/CD Security Risks*. https://owasp.org/www-project-top-10-ci-cd-security-risks/

TEPP treats documents and model output as untrusted, requires exact evidence and fail-closed validation, supplies accessible exact-value alternatives to graphics, and emits SBOM and provenance evidence for releases. Actions registry identities are inventoried against the protected-main tree rather than trusted because a YAML path once existed (GitHub, n.d.; OpenSSF, 2023; OWASP Foundation, 2023).

## LLM orchestration and test-time compute

### ORCH-TRINITY-2026

Xu, J., Sun, Q., Schwendeman, P., Nielsen, S., Cetin, E., & Tang, Y. (2026). TRINITY: An evolved LLM coordinator. In *International Conference on Learning Representations (ICLR 2026)*. https://arxiv.org/abs/2512.04695

### ORCH-CONDUCTOR-2026

Nielsen, S., Cetin, E., Schwendeman, P., Sun, Q., Xu, J., & Tang, Y. (2026). Learning to orchestrate agents in natural language with the Conductor. In *International Conference on Learning Representations (ICLR 2026)*. https://arxiv.org/abs/2512.04388

### ORCH-FUGU-2026

Tang, Y., Cetin, E., Xu, J., Sun, Q., Nielsen, S., Richard, V., Goda, H., Tymchenko, I., Nguyen, N., Lee, H., Ashiga, M., Kotyan, S., Kuroki, S., & Clanuwat, T. (2026). *Sakana Fugu technical report* [Preprint]. arXiv. https://arxiv.org/abs/2606.21228

TRINITY motivates lightweight learned model/role delegation over multiple turns; Conductor motivates query-adaptive natural-language workflow/topology/instruction generation and recursive test-time scaling; Fugu demonstrates a production-oriented family of query-adaptive agentic scaffolds building on these research lines (Xu et al., 2026; Nielsen et al., 2026; Tang et al., 2026). TEPP therefore treats direct routing, verification, fixed multi-agent workflows, adaptive orchestration, stage count, decomposition, recursion, access lists, role-specific reasoning effort, and total test-time budget as explicit experimental variables. `tepp_api::route_orchestration` is the deterministic selector for those variables; live provider execution and production-quality claims remain later work. Deeper/more-agent orchestration is never assumed better by default. Comparable-budget ablation, evidence support, calibration, disagreement, safety, cost, and failure behavior are required before a production claim. See `docs/research/adaptive-orchestration-router.md`.
