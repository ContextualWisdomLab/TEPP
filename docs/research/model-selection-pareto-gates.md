# Candidate-K statistical and Pareto gates (doctoring)

## Scope

`model_selection` admits a topic count `K` only when it is statistically
supported (`K >= 2`, finite held-out log-likelihood, finite non-negative
complexity) and not Pareto-dominated on those two objectives. An LLM vote may
later recommend among admissible candidates. It cannot itself define the
numerical optimum or bypass diagnostics (ADR 0012).

`select_fitted_candidate_k` now fits each candidate with the CPU `f64` TRSL-TM
reference and builds `ModelCandidate::statistical` from the actual in-sample
mixture log-likelihood and Schwarz's (1978) large-sample penalty. A typed
non-convergence, non-finite, or invalid-input failure is a failed candidate,
not a fabricated diagnostic. This slice does not choose a neural architecture,
run GPU inference, or claim a unique true `K` for every corpus. Known-truth
recovery reports computed RMSE of the selected `K` against the generating `K`.

## Authority

### Normative TEPP contract

- `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md` —
  model selection uses statistical/recovery/stability/alignment/fairness gates
  and a Pareto-style comparison before any future blinded LLM review; the LLM
  never defines the numerical optimum. The fitted path copies the ADR-owned
  v1 reference hyperparameters from `topic_measurement`
  (`σ² = 1.0`, `λ = 0.25`, `ρ = 0.01`, topic-smoothing `0.05`, GEM step
  `0.2`). Those values are not a second heuristic set.

### Supporting model-selection literature

Schwarz (1978) is the primary source for the candidate score
`ℓ − (p ln N)/2`. The printed large-sample Bayes procedure chooses the model
that maximizes `log M_j − (1/2) k_j log n` (Schwarz, 1978, p. 461; Project
Euclid PDF opened 2026-08-25T06:08Z from
https://projecteuclid.org/journalArticle/Download?urlId=10.1214%2Faos%2F1176344136).
`N` is the total token mass. `p` is the free-parameter count
`K(V−1) + (D+F)(K−1)`. `N < 1` makes `ln N` negative and fails closed. This
is the maximizer form, not a second invented weight. Akaike (1974) and
Burnham and Anderson (2002) remain background for likelihood-and-complexity
comparison. Deb et al. (2002) remains background for non-dominated (Pareto)
filtering. Those sources do not by themselves validate the TEPP acceptance
criteria or orchestration boundary; ADR 0012 is normative for this
repository. They do **not** authorize an LLM vote as a statistical estimator.

Roberts et al. (2014, 2019) remain the STM-family authority for the
reference estimator itself. They do not license a different numeric
hyperparameter set than the ADR 0012 / `topic_measurement` copies.

Akaike, H. (1974). A new look at the statistical model identification. *IEEE
Transactions on Automatic Control, 19*(6), 716–723.
https://doi.org/10.1109/TAC.1974.1100705

Burnham, K. P., & Anderson, D. R. (2002). *Model selection and multimodel
inference: A practical information-theoretic approach* (2nd ed.). Springer.

Deb, K., Pratap, A., Agarwal, S., & Meyarivan, T. (2002). A fast and elitist
multiobjective genetic algorithm: NSGA-II. *IEEE Transactions on Evolutionary
Computation, 6*(2), 182–197. https://doi.org/10.1109/4235.996017

Roberts, M. E., Stewart, B. M., Tingley, D., Lucas, C., Leder-Luis, J.,
Gadarian, S. K., Albertson, B., & Rand, D. G. (2014). Structural topic models
for open-ended survey responses. *American Journal of Political Science,
58*(4), 1064–1082. https://doi.org/10.1111/ajps.12103

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for
structural topic models. *Journal of Statistical Software, 91*(2), 1–40.
https://doi.org/10.18637/jss.v091.i02

Schwarz, G. (1978). Estimating the dimension of a model. *The Annals of
Statistics, 6*(2), 461–464. https://doi.org/10.1214/aos/1176344136
