# CLAUDE.md

Read and follow `AGENTS.md` before changing this repository. The repository-wide contracts in that file are normative. Use `DOCUMENTATION.md` to locate the canonical PRD/TRD/Architecture/UML/ERD/API/security/privacy/operability/traceability authorities.

## Working method

- Use test-driven development for every behavior change.
- Keep one branch/PR internally coherent and independently reviewable, but do not stop the invocation merely because one bounded slice completed while another safe action exists.
- After every mutation, merge, proof, or defer decision, re-enumerate the executable queue; waiting on one branch is local, not a repository-wide blocker.
- Prefer explicit types and small modules with stable interfaces.
- Preserve source spans, temporal provenance, uncertainty, purpose/authorization, and model-version metadata end to end.
- Preserve standalone operation and integrate with other CWL services only through versioned contracts; never use hidden cross-service database coupling.
- Do not replace statistical estimation with an LLM judgment.
- Do not convert association, temporal precedence, or document links into causal language without identification evidence.
- Do not remove repeated report language with global stopword lists or use TF-IDF/BM25 as inferential weights. Model template, section, copied-text, style, modality, and corpus-background sources explicitly.
- Do not treat raw topic proportions as ordinary Euclidean indicators. Use logistic-normal coordinates or valid log-ratio coordinates and propagate posterior uncertainty into ESEM/DSEM.
- Do not treat metric/weak invariance as a latent-mean license. Strong (equal loading and intercept) or strict is required; `#84` `metric` licenses shared metric meaning only.
- Do not use the difference quotient as a continuous-time rate. The scalar map is `a = ln(φ) / Δt` on event time. Discrete lags from unequal event intervals are not one coefficient; remap them through that log-rate. Binary64 `exp(a Δt) = 0` is not a discrete lag. A constant predictor's discrete effect is Voelkle et al. (2012, Eq. 12), evaluated as `a_yx (expm1(z) / a_xx)` with `z = a_xx Δt` so a finite result is not lost when `z` overflows to `-∞` or when `a_yx Δt` overflows. When `expm1(z)` overflows at a finite `z`, rewrite in log space; a zero continuous effect is exactly zero; an overflowing `a_yx/a_xx` rewrite term fails closed. The first-order product is the underflow limit of that equation, not the general constant-predictor discrete effect. A time-varying predictor whose sampling interval equals its constancy interval uses Voelkle et al. (2012, Eq. 14): `b* = a_yx Δt`. Unmatched intervals fail closed (Oud & Jansen, 2000, unread). Discrete process noise is Driver et al. (2017, Eq. 3): `Q_Δt = 0.5 q (expm1(z) / a)` with `z = 2 (a Δt)` and `q = G G⊤ ≥ 0`; do not form `2 a` first; `a = 0` and `z → 0` recover `q Δt`; a zero diffusion is exactly zero; an overflowing rewrite scale `0.5 q / a` fails closed; this is not a Kalman filter. `Q_Δt` is `cov(η_t | η_{t-1})`, not `Var(η_t)`. The lagged covariance is `exp(a Δt) p` and the unconditional variance is `exp(2 a Δt) p + Q_Δt` (Driver et al., 2017, Eq. 3–4, pp. 4–5; JSS has no numbered §2.2). A zero diffusion whose `2 (a Δt)` overflows to `+∞` is not a finite `Var(η_t)`. The stationary within-subject variance is the `Δt → ∞` limit of Eq. 4: `-q / (2 a)` for stable `a < 0` (JSS p. 16 `asymDIFFUSION`; §4.3). Form `(q / a) * -0.5`. Do not form `2 a` first. Do not form `0.5 q` first (`q = from_bits(1)` underflows). `a ≥ 0` has no finite stationary variance. Finite-interval `Q_Δt` is not that limit. Trait-plus-state variance is `trait + state` and lagged covariance is `trait + exp(a Δt) p` (Driver et al., 2017, §4.3, p. 9). Trait variance is not process noise and not `asymDIFFUSION`. Evolving the summed variance as if it were all state is not that map. This is not RI-CLPM.
- Separate cluster means before within-unit lag. CWC plus an event-time lag is not DSEM. Subtracting the person-specific mean from a raw autoregressive series does not isolate the lagged within-person effect (Curran & Bauer, 2011, pp. 607–608); already-centered residuals with irregular event intervals use the exact scalar map.
- Do not treat the CWC cluster-mean coefficient as the between-cluster effect. It is the contextual effect `between − within` (Enders & Tofighi, 2007, Table 2, pp. 124–127).
- Never use future-available evidence in historical model fits.
- Do not blanket-mask PII when identity/role/linkage is scientifically required. Follow the purpose-bound separation, opaque-ID, encryption, retention, and audit contract in `docs/PRIVACY_DATA_GOVERNANCE.md`.
- Treat documents and LLM outputs as untrusted. Model routing/orchestration may vary reasoning effort, decomposition, recursion and roles, but deterministic/statistical gates remain authoritative.
- Treat CSAP/SOC 2/ISO/NIST mappings as readiness evidence, never certification or attestation.

## Verification before completion

Before stating that a task is complete, run the exact focused tests, complete test suite, line/branch coverage gate, docstring gate, formatter, linter, dependency/security checks, build/package checks, documentation contracts, and any required CPU/GPU parity or true-parameter study. Report actual evidence and unresolved external gates. If another safe executable repository action remains, continue rather than using the verification result as a stopping point.
