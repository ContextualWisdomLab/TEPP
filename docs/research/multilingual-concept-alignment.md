# Shared multilingual concept alignment gates

## Scope

This slice delivers the first executable ADR 0004 contract in `concept_dictionary`:

1. label each language profile `validated`, `calibrated`, `provisional`, or `unresolved`;
2. refuse comparative interpretation from architecture support alone;
3. share one concept identity across native lexical channels;
4. refuse machine translation as measurement equivalence;
5. require exact source spans for semantic units and keep unknown meaning unresolved;
6. refuse default stopword deletion and TF-IDF/BM25 inferential weights;
7. permit cross-language mean comparison only with scalar or partial invariance evidence;
8. recover shared concept coordinates with a CPU `f64` RMSE.

Language-tailored segmentation/morphology, concept-dictionary persistence, and full fairness/invariance validation packages remain accepted-target. Topic estimation remains ADR 0012.

## Authoritative sources

Phillips, A., & Davis, M. (2009). *Tags for identifying languages* (RFC 5646). Internet Engineering Task Force. https://doi.org/10.17487/RFC5646

Meredith, W. (1993). Measurement invariance, factor analysis and factorial invariance. *Psychometrika, 58*(4), 525–543. https://doi.org/10.1007/BF02294825

Vandenberg, R. J., & Lance, C. E. (2000). A review and synthesis of the measurement invariance literature: Suggestions, practices, and recommendations for organizational research. *Organizational Research Methods, 3*(1), 4–70. https://doi.org/10.1177/109442810031002

Bender, E. M. (2011). On achieving and evaluating language-independence in NLP. *Linguistic Issues in Language Technology, 6*. https://doi.org/10.33011/lilt.v6i.1239

Mimno, D., Wallach, H. M., Naradowsky, J., Smith, D. A., & McCallum, A. (2009). Polylingual topic models. In *Proceedings of the 2009 Conference on Empirical Methods in Natural Language Processing* (pp. 880–889). Association for Computational Linguistics.

Robertson, S., & Zaragoza, H. (2009). The probabilistic relevance framework: BM25 and beyond. *Foundations and Trends in Information Retrieval, 3*(4), 333–389. https://doi.org/10.1561/1500000019

## Formula notes

- **Concept-coordinate RMSE** \(\mathrm{RMSE}=\sqrt{\frac{1}{n}\sum_i (x_i-y_i)^2}\) on already-aligned shared-space coordinates.
- RMSE is computed from recovered versus known true coordinates; tests do not hard-code expected recovery numbers as the scientific target.
- Scalar or partial invariance is required before a cross-language mean comparison (Meredith, 1993; Vandenberg & Lance, 2000). Configural or metric evidence is not sufficient.
- Architecture support is not a validated language profile (Bender, 2011).
- TF-IDF and BM25 remain lexical retrieval weights, not inferential weights for the statistical estimator (Robertson & Zaragoza, 2009).

## Verification

- noiseless shared Korean/English coordinates recover with machine-scale computed RMSE;
- a translation offset produces a larger computed RMSE than the identity mapping;
- unresolved/provisional profiles, missing/reversed spans, forced unknown concepts, default stopword deletion, TF-IDF/BM25 weights, and insufficient invariance fail closed.
