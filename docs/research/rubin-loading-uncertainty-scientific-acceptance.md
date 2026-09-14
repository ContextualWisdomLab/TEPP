# Rubin loading uncertainty: repeated-sampling acceptance evidence

## Scope

This evidence belongs to the `rubin_loading_uncertainty_v1` Analysis Run profile. It exercises the public profile end to end while leaving reusable estimator arithmetic in protected-main `psychometric_core::{recover_loading_point_estimate_mean, combine_draw_level_ols_loadings}`.

It is not evidence for Mislevy person-level plausible values, ESEM/DSEM estimation, multilevel or cross-classified models, or a general missing-data imputation engine. The profile consumes already-mapped factor scores and complete-data indicator draws, so the acceptance design preserves that input structure rather than inventing a different estimand.

The executable authority is `crates/analysis_engine/tests/rubin_loading_scientific_acceptance.rs` on the exact PR head. The values below are deterministic rounded outputs of that test design. CI must reproduce them within the checked tolerance; this document alone is not a passing receipt. Bias, RMSE, empirical coverage, and Monte Carlo standard errors are acceptance-harness diagnostics only; they are not exported as product estimation APIs and do not duplicate the profile's loading or Rubin-combination estimator.

## Repeated-sampling design

Each replication generates factor scores `f_i ~ N(0, 1)` and a base indicator

`y_i = lambda * f_i + epsilon_i`, with `epsilon_i ~ N(0, sigma^2)`.

Every fourth observation also receives draw-specific zero-mean Gaussian uncertainty with standard deviation `0.5 * sigma`. Other observations are held fixed across draws. This deliberately makes `B > 0` without replacing the profile with a second estimator. The deterministic SplitMix64 seed and Box-Muller transform are implemented in the Rust test; no external RNG or generated fixture is required.

Eight predeclared scenarios vary two observation counts (`48`, `160`), two draw counts (`8`, `32`), and two paired loading/residual-scale settings (`0.4`/`0.5` and `1.2`/`1.2`). Each scenario uses 512 attempted replications. A replication is recovered only when the public Analysis Run executor succeeds; failures remain in the attempted/recovered/failed accounting.

Every generated population has its own immutable snapshot identity and its own accepted Analysis Run identity. This matters because a Monte Carlo replication changes the underlying population; reusing one `snapshot_id` or one run receipt across different generated datasets would contradict the same provenance semantics this acceptance test is intended to exercise. The rolling-origin comparison is different: the early-full, early-prefix, and later-cutoff executions inside one replication share that replication's snapshot because they are alternative historical views of one generated population, but they use distinct run/idempotency identities. RED `16e3c151d6cadd79c1ec8e32d74a8401333be42c` exposed the original snapshot aliasing; repair `ed2ac416ac42cb15909cefde0581af408de652e7` binds the corrected identities.

Bias is the mean signed recovery error. Bias MCSE is the sample standard deviation of recovery errors divided by `sqrt(R)`. RMSE is the square root of mean squared recovery error; its MCSE uses the delta method from the sample standard deviation of squared errors. Coverage MCSE is the sample standard error of the 0/1 coverage indicator. These numerical summaries are computed over recovered executions. The acceptance gate separately requires `recovered = attempted` and `failed = 0`, so a passing scenario cannot silently omit a refused execution from the metric denominator. All count-to-floating conversions are bounded and checked in the Rust harness rather than using unchecked integer casts.

The interval diagnostic is

`Qbar +/- 1.959963984540054 * sqrt(T)`.

Here `Qbar` is the Rubin-combination artifact field `mean_loading`, so the uncertainty interval is centered on the quantity to which `T` belongs. The separately reported `point_estimate_mean` remains the robust point-estimate recovery target. This is explicitly a large-sample normal diagnostic for these admitted designs, not a small-sample Rubin degrees-of-freedom implementation. A coverage claim is therefore limited to the predeclared repeated-sampling design below. The acceptance envelope is `|coverage - 0.95| <= 0.01 + 2 * MCSE`; it is not promoted to a universal interval guarantee.

## Checked-in evidence

| scenario | n | m | lambda | sigma | attempted / recovered / failed | bias | bias MCSE | RMSE | RMSE MCSE | coverage | coverage MCSE | mean Ubar | mean B | mean T |
| --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| small_m8_low | 48 | 8 | 0.4 | 0.5 | 512 / 512 / 0 | 0.002108 | 0.003402 | 0.076943 | 0.002491 | 0.951172 | 0.009534 | 0.005945 | 0.000344 | 0.006333 |
| small_m32_low | 48 | 32 | 0.4 | 0.5 | 512 / 512 / 0 | 0.000426 | 0.003284 | 0.074239 | 0.002370 | 0.964844 | 0.008147 | 0.005974 | 0.000343 | 0.006327 |
| small_m8_high | 48 | 8 | 1.2 | 1.2 | 512 / 512 / 0 | -0.015361 | 0.007998 | 0.181450 | 0.005577 | 0.953125 | 0.009351 | 0.034424 | 0.001921 | 0.036585 |
| small_m32_high | 48 | 32 | 1.2 | 1.2 | 512 / 512 / 0 | 0.003076 | 0.007818 | 0.176754 | 0.005550 | 0.957031 | 0.008971 | 0.034184 | 0.002005 | 0.036252 |
| large_m8_low | 160 | 8 | 0.4 | 0.5 | 512 / 512 / 0 | 0.001179 | 0.001769 | 0.040007 | 0.001279 | 0.957031 | 0.008971 | 0.001694 | 0.000100 | 0.001807 |
| large_m32_low | 160 | 32 | 0.4 | 0.5 | 512 / 512 / 0 | 0.000850 | 0.001835 | 0.041498 | 0.001359 | 0.951172 | 0.009534 | 0.001719 | 0.000100 | 0.001822 |
| large_m8_high | 160 | 8 | 1.2 | 1.2 | 512 / 512 / 0 | -0.007944 | 0.004270 | 0.096856 | 0.003125 | 0.953125 | 0.009351 | 0.009633 | 0.000583 | 0.010289 |
| large_m32_high | 160 | 32 | 1.2 | 1.2 | 512 / 512 / 0 | 0.003612 | 0.004269 | 0.096579 | 0.002863 | 0.960938 | 0.008571 | 0.009718 | 0.000584 | 0.010320 |

The executable gate additionally requires zero profile refusals, `|bias| <= 0.01 + 3 * bias_MCSE`, `RMSE <= 0.25 * sigma`, positive mean between-draw variance, and mean `T > Ubar` in every scenario. The draw-count and application matrix ceilings remain resource envelopes only; no scientific recommendation is inferred from `m <= 256` or the one-million-cell admission bound.

## Leakage-safe rolling origin

A separate 512-replication design generates 64 rows with `lambda = 0.8`, `sigma = 1.0`, 16 draws, and draw-specific uncertainty `0.5`. The first 48 rows are available before the early knowledge cutoff; the last 16 become available only later.

For every replication the early run over all 64 supplied rows must be bit-identical on `point_estimate_mean`, Rubin mean, and `T` to a run supplied only the first 48 rows. The early artifact must report 48 admitted and 16 excluded rows. A later cutoff admits all 64 rows. These three executions share one replication-specific immutable snapshot and carry three distinct run identities; no Analysis Run receipt is reused across different input views.

| rolling-origin state | attempted / recovered / failed | bias | RMSE | mean T |
| --- | --- | ---: | ---: | ---: |
| early cutoff, 48 eligible of 64 | 512 / 512 / 0 | 0.018218 | 0.155489 | 0.025215 |
| later cutoff, 64 eligible of 64 | 512 / 512 / 0 | 0.012167 | 0.131383 | 0.018653 |

The later-cutoff RMSE is lower in this predeclared design, but that is a design-specific empirical result, not a monotonicity contract. The actual temporal contract is the stronger historical replay condition: post-cutoff rows cannot change the earlier scientific result.

## Sampling-structure boundary

This evidence is intentionally single-level because `rubin_loading_uncertainty_v1` currently accepts one factor-score vector plus complete-data indicator draws. It does not flatten a multilevel, cross-classified, or multiple-membership design: those structures are absent from this profile contract. If the profile later gains such membership structure, this evidence must be extended before the corresponding scientific claim can be carried forward.

## Draw-generation activation boundary

This repeated-sampling study validates the behavior of the exact generator/analysis design declared above. It does **not** establish that an arbitrary caller-supplied finite draw matrix is a proper multiple-imputation or posterior draw set for Rubin-style frequentist inference. The current Analysis Run input identifies snapshot, availability, indicator kind, and the draw values, but it does not yet carry a versioned draw-generation/imputer contract or a validation-evidence identity for the generator/analysis pairing.

That distinction matters because Rubin combining-rule validity depends on more than the algebraic identity for `T`. Rubin's proper-imputation framework ties the repeated-imputation behavior to the estimand and imputation procedure; uncongenial imputer/analyst combinations can make Rubin variance biased even when each complete-data analysis is well defined. Therefore the coverage results in this document are scoped to this predeclared design and must not be projected as universal inferential authorization for arbitrary complete-data draws.

Issue #505 owns the missing product activation/projection contract. Until an approved versioned generator/analysis pairing is digest-bound to its validation evidence, TEPP may preserve the bounded descriptive `Qbar/Ubar/B/T` arithmetic but must keep any broader inferential claim fail closed. This does not change the protected numerical owner and does not invalidate the scoped #503 recovery study.

## Primary authority and traceability

Rubin, D. B. (1987). *Multiple Imputation for Nonresponse in Surveys*. Wiley. https://doi.org/10.1002/9780470316696

Rubin, D. B. (1996). Multiple imputation after 18+ years. *Journal of the American Statistical Association, 91*(434), 473–489. https://doi.org/10.1080/01621459.1996.10476908

Meng, X.-L. (1994). Multiple-imputation inferences with uncongenial sources of input. *Statistical Science, 9*(4), 538–558. https://doi.org/10.1214/ss/1177010269

Xie, X., & Meng, X.-L. (2017). Dissecting multiple imputation from a multi-phase inference perspective: What happens when God's, imputer's and analyst's models are uncongenial? *Statistica Sinica, 27*(4), 1485–1545. https://doi.org/10.5705/ss.2014.067

Repository formula authority remains `docs/research/rubin-total-variance.md`. Rubin (1996) supports `T = Ubar + (1 + 1/m)B`; it does not by itself authorize the normal interval diagnostic above, which is deliberately labeled and empirically checked rather than presented as Rubin's small-sample interval rule.

Issue #503 is satisfied only when the exact-head Rust acceptance test, documentation checks, security gates, coverage gates, and required review all pass on the surviving implementation/fold head. Issue #505 remains a separate activation/projection-policy gap. Predecessor receipts do not transfer.
