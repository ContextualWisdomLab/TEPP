# Bias standard error exact-anchor conditioning

## Problem

GAP-106 made exact translated-residual admission permutation-invariant by considering every represented observation as an anchor and taking the first exact candidate in canonical `(high, low)` order. That removed transport-order drift, but canonical order is not a numerical conditioning criterion. When several anchors satisfy the same exactness proof, different exact translations can expose different binary64 magnitudes to the later square, compensated-sum, fused multiply-add, and square-root path.

The public counterexample uses three exactly represented residuals with `truth = [0, 0, 0]`:

- `middle = f64::from_bits(0x3ff7_c8a6_308f_7624)`,
- `low = f64::from_bits(0x3ff0_4284_fcf1_21a0)`,
- `high = f64::from_bits(0x3fff_659d_6d25_7410)`.

For this represented multiset, the exact squared standard error of the mean is

`850963186800334380866421373237 / 11408855402054064613470328848384`,

whose correctly rounded binary64 square root is `0x3fd1_7a99_c875_b980`.

All three observations are valid exact translation anchors. The canonical-low anchor produces translated magnitudes up to `0x1.e4630e068a4e0p-1` and the predecessor returns adjacent upper bits `0x3fd1_7a99_c875_b981`. The represented middle anchor produces translated magnitudes no larger than `0x1.e73dcf257f7b0p-2` and returns the correctly rounded `0x3fd1_7a99_c875_b980`. The estimand, represented residual multiset, and exact-admission predicate are unchanged; only the exact anchor's conditioning differs.

This is deterministic arithmetic error, not Monte Carlo uncertainty. GAP-106 solved which *set* of observations may admit exact translation, but it still chose among several admissible representations using an arbitrary lexical criterion before a numerically sensitive second-moment reconstruction.

## Constraints

- TEPP owns Validation Evidence performance-measure arithmetic; reusable static psychometric estimation remains in `fast-mlsirm`.
- The existing error-free high delta, low delta, and recombined delta proofs remain mandatory. The repair must not classify any previously inexact translation as exact.
- Anchor selection must remain a function of the represented multiset rather than incoming observation order.
- The repair must not sort or otherwise rewrite the scientific observation pairing outside this local translation choice.
- Production arithmetic remains deterministic Rust binary64. No arbitrary-precision runtime dependency is added.
- Existing two-level algebraic shortcuts, subnormal rational projection, and fail-closed fallback semantics remain unchanged.

## Decision

Every represented observation remains an anchor candidate. For each candidate that satisfies all existing exactness proofs, TEPP computes the maximum absolute translated residual. It selects the exact candidate with the smallest such maximum magnitude. Canonical `(high, low)` total order is retained only as a deterministic tie-breaker.

This criterion is local and causal: it does not claim a globally correctly rounded standard error for every `n > 2` sample. It removes an avoidably wide exact translation before the same power-of-two normalization and second-moment implementation. In the public counterexample, the centered represented observation halves the translated working radius and removes the one-ULP upward drift without changing the exact residual geometry.

The selection is permutation-invariant because every candidate is evaluated and the objective plus tie-breaker depends only on represented candidate values. It is also monotone with respect to the specific numerical risk being repaired: among exact translations of the same multiset, a smaller maximum magnitude cannot increase the exponent range subsequently exposed to squaring.

## Alternatives rejected

1. **Keep the first exact anchor in canonical `(high, low)` order.** Rejected because the public contract shows that lexical canonicalization can select a one-ULP-worse exact representation.
2. **Use the incoming first or first exact observation.** Rejected because it reintroduces the permutation defect closed by GAP-106.
3. **Always use the median high residual.** Rejected because a median high part is not necessarily an exact anchor once subtraction low terms are part of the represented residual decomposition.
4. **Choose whichever anchor happens to reproduce an external high-precision oracle.** Rejected because production path selection cannot depend on an unavailable oracle and would amount to test-payload fitting.
5. **Replace the general path with arbitrary-precision variance arithmetic.** Rejected as broader than this bounded conditioning defect and outside the current Rust binary64 reference boundary.

## Evidence and traceability

- Public RED: `49343ab6e7f1a4cadc0b9c71e0757edea0256add`
- Causal source repair: `0fc3ea97106f5156a6d68d4db8fd6f4a2ace0ac4`
- CHANGELOG fragment: `2793ff5927ed3b79886cd3d4daa2369357c1710b`
- Module/API: `crates/validation_core/src/bias.rs` → `bias_standard_error` → `exact_translated_residual_standard_error` → `canonical_exact_translated_residuals`
- Public contract: `crates/validation_core/tests/bias_standard_error_anchor_conditioning_contract.rs`
- Correct represented result: `0x3fd1_7a99_c875_b980`
- GAP-106 canonical-low predecessor result: `0x3fd1_7a99_c875_b981`
- The contract covers low/middle/high-first permutations and sign mirrors so the GAP-106 permutation guarantee is retained while the conditioned anchor changes the numerical result.

Hosted exact-head CI remains authoritative for GREEN after the surviving branch head is known. This repair is a bounded represented-input arithmetic acceptance claim, not a global correctly-rounded guarantee for all standard errors.

## Standards and methodological basis

IEEE 754-2019 remains the binary floating-point arithmetic basis for the Rust `f64` reference path. The active P754 project is a revision project that supersedes 754-2019 when completed; it is not yet a published replacement. ISO/IEC 60559:2020 remains the published international adoption of the 754-2019 arithmetic model.

Morris, White, and Crowther (2019) distinguish deterministic performance-measure computation from Monte Carlo uncertainty. A fixed represented sample must therefore not acquire extra numerical variation from an avoidable choice among algebraically equivalent exact translations.

The currently published AERA/APA/NCME *Standards for Educational and Psychological Testing* remains the 2014 edition while its announced revision is in progress. The published edition remains the normative testing reference until a replacement is issued.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. https://www.testingstandards.net/open-access-files.html

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE Standards Association. https://standards.ieee.org/ieee/754/6210/

Institute of Electrical and Electronics Engineers. (2024). *P754: Standard for floating-point arithmetic* [Active PAR]. IEEE Standards Association. https://standards.ieee.org/ieee/754/11684/

International Organization for Standardization, & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). https://www.iso.org/standard/80985.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
