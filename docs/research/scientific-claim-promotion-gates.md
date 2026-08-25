# Scientific claim-promotion gates

## Scope

This note doctors the first ADR 0014 executable promotion slice in `validation_core`:

1. four claim authorities remain distinct (`decision_accepted`, `implemented_main`, `scientifically_supported`, `released`);
2. implementation, scientific, and release authorities bind to one exact protected-head SHA;
3. queued checks, predecessor-head results, skipped required tests, and LLM judgments cannot promote any authority;
4. scientific promotion uses computed RMSE and its standard error, not a hardcoded recovery threshold.

Full package/image release bundles remain accepted-target. No database migration is allocated.

## Authoritative sources

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Wasserstein, R. L., & Lazar, N. A. (2016). The ASA statement on *p*-values: Context, process, and purpose. *The American Statistician, 70*(2), 129–133. https://doi.org/10.1080/00031305.2016.1154108

## Application

The National Academies (2019) separate computational reproducibility from a scientific claim that a result is correct. Wasserstein and Lazar (2016) refuse to treat a passing statistical threshold as automatic scientific authority. TEPP therefore refuses to promote `implemented_main`, `scientifically_supported`, or `released` from queued, stale, skipped, or LLM evidence, and accepts scientific recovery only when computed RMSE lies within a configured number of its own standard errors (National Academies of Sciences, Engineering, and Medicine, 2019; Wasserstein & Lazar, 2016).

## Verification

- `DecisionAccepted` binds without implementation evidence;
- `ImplementedMain` requires the candidate SHA to equal the protected SHA and passing exact-head tests;
- queued, predecessor, skipped-required, and LLM evidence return dedicated fail-closed errors;
- `ScientificallySupported` and `Released` require their additional gates;
- a near-recovery vector promotes only when the computed RMSE/SE multiplier admits it;
- a large bias vector returns `ClaimRecoveryRejected`.
