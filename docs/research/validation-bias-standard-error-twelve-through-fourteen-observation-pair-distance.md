# Exact represented-input mean-bias standard error for 12–14 observations

## Decision scope

TEPP's Validation Evidence layer treats the represented binary64 inputs as the numerical observation contract. For small samples whose residual subtraction and every pairwise residual difference are proven exact, the identity

\[
SE(\bar r)^2 = \frac{\sum_{i<j}(r_i-r_j)^2}{n^2(n-1)}
\]

provides an exact dyadic/rational authority before the final square root. The existing bounded `u128` implementation reduces the scientific ratio by its integer GCD, uses binary64 only to seed a candidate, and decides the candidate versus its adjacent neighbor by exact dyadic-square/midpoint comparison. The path remains fail-closed when exact subtraction, checked integer construction, denominator bounds, or midpoint proof cannot be established.

GAP-121 through GAP-123 extend only the demonstrated small-sample admission boundary from 11 through 14 observations. They do not assert globally correctly rounded standard errors for arbitrary `n`, and they do not replace the general translated second-moment fallback.

## GAP-121 — twelve observations

The exact represented residuals are

`[18_775_780, 73_991_125, 198_689_967, 631_050_858, 778_682_730, 826_435_964, 853_584_967, 1_530_809_509, 1_562_270_376, 1_586_067_346, 1_682_017_356, 1_750_122_820]`.

The 66 exact squared pair distances sum to `55_761_699_077_165_681_660`. With `n = 12`, the scientific denominator is `12^2(12-1) = 1_584`. `gcd(N, 1_584) = 4`, so the reduced exact radicand is `13_940_424_769_291_420_415 / 396`. The translated floating moment/`sqrt` path returns `0x41a6_5ddb_5161_0460`; exact midpoint comparison returns the correctly rounded represented result `0x41a6_5ddb_5161_045f`.

Public RED: `b6ac111731a53b5d8d67d0222292542617432478` (`bias_standard_error_twelve_observation_pair_distance_contract.rs`). Causal repair: `035cf392b5e9f115c7b5a2589ebbeadb311d7a45`, extending only the existing pair-distance admission from `n <= 11` to `n <= 12` and moving the zero/fallback boundary to 13.

## GAP-122 — thirteen observations

The exact represented residuals are

`[13_412_968, 42_445_497, 117_340_356, 152_587_301, 309_740_336, 359_871_277, 717_207_453, 811_347_466, 1_016_388_094, 1_092_140_579, 1_412_658_032, 1_429_960_424, 1_525_741_984]`.

The 78 exact squared pair distances sum to `49_391_029_335_804_262_120`. With `n = 13`, the scientific denominator is `13^2(13-1) = 2_028`. `gcd(N, 2_028) = 4`, so the reduced exact radicand is `12_347_757_333_951_065_530 / 507`. The translated floating moment/`sqrt` path returns `0x41a2_9a8e_6db8_cb77`; exact midpoint comparison returns `0x41a2_9a8e_6db8_cb76`.

Public RED: `369af46de7e0719cfb4db04fedf4d2775e04f62c` (`bias_standard_error_thirteen_observation_pair_distance_contract.rs`). Causal repair: `0b8727e7550022ad5f89b2e5b48129f5b2f520eb`, extending the same bounded proof from `n <= 12` to `n <= 13` and retaining 14 as fallback until independent evidence exists.

## GAP-123 — fourteen observations

The exact represented residuals are

`[169_198_177, 170_212_614, 363_421_213, 482_119_205, 503_813_918, 556_586_639, 757_346_256, 811_004_051, 882_684_595, 948_393_523, 1_052_267_532, 1_523_536_361, 1_895_880_649, 1_922_535_250]`.

The 91 exact squared pair distances sum to `59_666_178_422_422_564_725`. With `n = 14`, the scientific denominator is `14^2(14-1) = 2_548`; the ratio is already reduced (`gcd = 1`). The translated floating moment/`sqrt` path returns adjacent-lower `0x41a2_3df9_5954_fb0a`; exact represented-input midpoint comparison returns `0x41a2_3df9_5954_fb0b`.

Public RED: `db40397603e2514cf3be25783dc65018aab64f10` (`bias_standard_error_fourteen_observation_pair_distance_contract.rs`). Causal repair: `58efe80da3c4a57e2b69860f5d4178894f769420`, extending only the existing exact pair-distance admission from `n <= 13` to `n <= 14`, updating its rustdoc, and preserving `n = 15` as explicit fallback.

The public contracts exercise original order, reverse order, an independent permutation, and sign mirrors. The expected bit pattern is invariant across those transformations because the scientific target depends on pair-distance geometry, not transport order or global sign.

## Alternatives rejected

An unconditional pair-distance implementation for all sample sizes is not adopted. It would make the O(n^2) reference path a general production cost without evidence that the same proof remains the right performance/overflow boundary. Payload-specific branches are also rejected because the defect is a class of exact represented-input double rounding, not a property of one fixture. Arbitrary-precision production arithmetic is unnecessary for these demonstrated cases: the existing checked `u128` proof admits them after exact dyadic construction and GCD reduction. Weakening subtraction exactness or midpoint proof would turn a reference path into an unverified approximation and is therefore rejected.

The next boundary, `n = 15`, remains on the established translated second-moment path. It should move only after a realistic represented-input counterexample and numerical/performance evidence demonstrate a causal need.

## Methodological and standards trace

IEEE 754-2019 / ISO/IEC 60559:2020 remain the published floating-point arithmetic authorities for binary64 behavior. The exact-midpoint decision is used so a rounded ratio followed by `sqrt` cannot silently become the scientific authority when the represented inputs admit an exact bounded rational proof. This is numerical validation of a performance measure, not statistical uncertainty.

Morris, White, and Crowther (2019) distinguish the performance measure being evaluated from Monte Carlo uncertainty caused by finite simulation replications. These GAPs are deterministic calculation defects for a fixed represented input and therefore belong to the former layer. AERA, APA, and NCME's *Standards for Educational and Psychological Testing* require validity evidence appropriate to the intended interpretation and use; TEPP records the numerical estimator contract and its edge-case evidence rather than allowing an LLM judgment to substitute for the arithmetic.

## Traceability

| Gap | Domain owner | Production source | Public contract | Exact evidence |
|---|---|---|---|---|
| GAP-121 | Validation Evidence | `crates/validation_core/src/bias_se.rs` | `crates/validation_core/tests/bias_standard_error_twelve_observation_pair_distance_contract.rs` | RED `b6ac1117...` -> repair `035cf392...` |
| GAP-122 | Validation Evidence | `crates/validation_core/src/bias_se.rs` | `crates/validation_core/tests/bias_standard_error_thirteen_observation_pair_distance_contract.rs` | RED `369af46d...` -> repair `0b8727e7...` |
| GAP-123 | Validation Evidence | `crates/validation_core/src/bias_se.rs` | `crates/validation_core/tests/bias_standard_error_fourteen_observation_pair_distance_contract.rs` | RED `db403976...` -> repair `58efe80d...` |

No ADR or PRD target changes are required: the estimator target remains standard error of mean signed bias under the existing represented-input contract. These changes repair numerical realization and evidence, not latent-variable meaning or service authority.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization, & International Electrotechnical Commission. (2020). *ISO/IEC 60559:2020 Information technology—Microprocessor systems—Floating-point arithmetic*.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
