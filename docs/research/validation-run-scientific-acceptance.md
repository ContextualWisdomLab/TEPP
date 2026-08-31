# Validation-run scientific acceptance evidence

## Scope

This note doctors the first GAP-003A slice in `analysis_engine`:

1. immutable evidence, knowledge cutoff, model, seed, backend, and precision bind to one hash-stable validation run;
2. the accepted receipt carries no scientific metrics;
3. completion emits `tepp.scientific_acceptance.v1` with RMSE, bias, interval coverage (Wilson bounds), temporal-order accuracy, and an SE-aware gate;
4. LLM-authored recovery, non-finite inputs, empty or duplicate evidence, snapshot mismatch, and cutoff-empty corpora fail closed.

Postgres persistence, restart/recovery, and Compose execution remain GAP-003B. This slice is not implemented-main.

## Authoritative sources

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values: Context, process, and purpose. *The American Statistician, 70*(2), 129–133. https://doi.org/10.1080/00031305.2016.1154108

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

## Application

The National Academies (2019) separate computational reproducibility (same binding, same digest) from a scientific claim that recovery is correct. Wasserstein and Lazar (2016) refuse to treat a passing threshold as automatic scientific authority, so a failed SE-aware gate still reports metrics with `se_gate_accepted = false` rather than inventing a passing claim. Wilson (1927) supplies the coverage interval bounds already implemented in `validation_core`. TEPP therefore binds cutoff-safe evidence before any metric is computed, refuses LLM-authored recovery, and reports RMSE, bias, coverage, temporal order, and the SE-aware gate as operator-usable evidence (National Academies of Sciences, Engineering, and Medicine, 2019; Wasserstein & Lazar, 2016; Wilson, 1927).

## Verification

- identical eligible evidence in any order and with extra post-cutoff units yields the same `tepp-validation-{32 hex}` identity;
- receipts serialize without RMSE, bias, or coverage fields;
- known-truth recovery emits `tepp.scientific_acceptance.v1` with a digest-stable JSON body;
- a large residual vector remains operator-readable with `se_gate_accepted = false`;
- LLM authorship, NaN recovery, empty corpora, duplicates, snapshot mismatch, wrong profile/model, and cutoff-empty eligibility return dedicated fail-closed errors.
