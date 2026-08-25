# Inferential retrieval-weight refusal

## Scope

This note doctors the `corpus_split` gate that keeps TEPP from treating information-retrieval scores as statistical estimator weights:

1. only group-normalized ESS and uniform observation weights may enter an estimator as inferential weights;
2. TF-IDF and BM25 fail closed;
3. global stopword-list deletion is not the default preprocessing rule.

No database migration is allocated. A later topic backend may consume retrieval scores as *non-inferential* diagnostics only.

## Authoritative sources

Salton, G., & Buckley, C. (1988). Term-weighting approaches in automatic text retrieval. *Information Processing & Management, 24*(5), 513–523. https://doi.org/10.1016/0306-4573(88)90021-0

Robertson, S., & Zaragoza, H. (2009). The probabilistic relevance framework: BM25 and beyond. *Foundations and Trends® in Information Retrieval, 3*(4), 333–389. https://doi.org/10.1561/1500000019

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for structural topic models. *Journal of Statistical Software, 91*(2), 1–40. https://doi.org/10.18637/jss.v091.i02

## Application

Salton and Buckley (1988) and Robertson and Zaragoza (2009) describe TF-IDF and BM25 as *retrieval ranking* functions. Based on that distinction, TEPP implements an explicit policy that refuses `tf_idf` and `bm25` as estimator inputs. Independently, TEPP refuses global stopword deletion as the default token rule so token and background effects remain available for modeling; both policies are enforced by the `corpus_split` contract.

## Verification

- `refuse_inferential_retrieval_weight` admits `group_normalized_ess` and `uniform`;
- `tf_idf` and `bm25` return `InferentialRetrievalWeight`;
- `refuse_default_stopword_deletion` admits `preserve_and_model_background` and refuses `global_stopword_list`;
- computed RMSE of known membership shares is lower under `group_normalized_ess` than under an L1-normalized TF-IDF surrogate.
