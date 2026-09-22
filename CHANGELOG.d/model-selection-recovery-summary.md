### Scientific model-selection recovery summary

- Add `SelectedKRecoverySummary` / `selected_k_recovery_summary` so known-truth candidate-`K` recovery reports conditional bias and RMSE together with the attempted/successful/failed replication denominator.
- Report Monte Carlo standard errors for bias and RMSE across successful replications; RMSE uncertainty uses the delta method on mean squared error and is zero when every successful replication exactly recovers the truth.
- Require at least two successful replications for this acceptance summary. The existing RMSE-only helper remains compatibility/regression coverage and is not sufficient scientific acceptance evidence by itself.
- This change does not claim rolling-origin or held-out predictive validation. #680 still requires canonical `corpus_split` windows, leakage-safe train/evaluation admission, and a separately measured predictive diagnostic before candidate-`K` recovery can satisfy release acceptance.
