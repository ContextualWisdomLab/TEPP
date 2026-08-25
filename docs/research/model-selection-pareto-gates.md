# Candidate-K statistical and Pareto gates (doctoring)

## Scope

`model_selection` admits a topic count `K` only when it is statistically
supported (`K >= 2`, finite held-out log-likelihood, finite non-negative
complexity) and not Pareto-dominated on those two objectives. An LLM vote may
later recommend among admissible candidates. It cannot itself define the
numerical optimum or bypass diagnostics (ADR 0012).

`select_fitted_candidate_k` now fits each candidate with the CPU `f64` TRSL-TM
reference and builds `ModelCandidate::statistical` from the actual in-sample
mixture log-likelihood and a BIC-style parameter-count penalty. A typed
non-convergence, non-finite, or invalid-input failure is a failed candidate,
not a fabricated diagnostic. This slice does not choose a neural architecture,
run GPU inference, or claim a unique true `K` for every corpus. Known-truth
recovery reports computed RMSE of the selected `K` against the generating `K`.

## Authority

### Normative TEPP contract

- `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md` —
  model selection uses statistical/recovery/stability/alignment/fairness gates
  and a Pareto-style comparison before any future blinded LLM review; the LLM
  never defines the numerical optimum.

### Supporting model-selection literature

Akaike (1974) and Burnham and Anderson (2002) provide background for
likelihood-and-complexity comparison of fitted candidates. Deb et al. (2002)
provides background for non-dominated (Pareto) filtering when two objectives
are compared simultaneously. Those sources do not by themselves validate the
exact TEPP thresholds, acceptance criteria, or orchestration boundary; ADR
0012 is normative for this repository. They do **not** authorize an LLM vote as
a statistical estimator.

Akaike, H. (1974). A new look at the statistical model identification. *IEEE
Transactions on Automatic Control, 19*(6), 716–723.
https://doi.org/10.1109/TAC.1974.1100705

Burnham, K. P., & Anderson, D. R. (2002). *Model selection and multimodel
inference: A practical information-theoretic approach* (2nd ed.). Springer.

Deb, K., Pratap, A., Agarwal, S., & Meyarivan, T. (2002). A fast and elitist
multiobjective genetic algorithm: NSGA-II. *IEEE Transactions on Evolutionary
Computation, 6*(2), 182–197. https://doi.org/10.1109/4235.996017
