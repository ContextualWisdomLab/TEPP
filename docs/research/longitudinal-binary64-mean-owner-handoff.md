# Longitudinal binary64 mean owner handoff

## Decision status

**Proposed consumer boundary.** This record does not activate a new numerical algorithm. It records why TEPP Longitudinal Modeling must not repair the current mixed-sign mean defect with another local summation heuristic and what evidence is required before replacing it with a released reusable numerical contract.

## Consumer finding

TEPP PR #310 exposes a compact public RED through `crates/longitudinal_core/tests/mixed_sign_mean_rounding_contract.rs`.

The represented rate values are:

- `P = 0x1.8p+106` (`0x4698000000000000`),
- `-2^53`,
- `-1`.

The exact real sum is `P - 2^53 - 1`. The correctly rounded mean is `0x1.fffffffffffffp+104` (`0x467fffffffffffff`). The current `scaled_compensated_mean` path in `crates/longitudinal_core/src/irregular_residual.rs` returns `0x4680000000000000`, one ULP high, because the same-side coalescing step can lose the final `-1` before the original-count division.

This is an ordinary finite three-value input constructed through the public already-centered residual path. It is not a resource-extreme synthetic witness.

## Bounded-context ownership

TEPP owns the temporal estimand: event-time admission, Driver et al. log-rate construction, CWC semantics, longitudinal evidence composition, and the decision to publish or refuse a result.

Reusable finite binary64 sum/mean arithmetic is static numerical infrastructure and belongs to `ContextualWisdomLab/fast-mlsirm`. Canonical owner gap `fast-mlsirm#1814` therefore owns an exact or correctly rounded finite-sum/mean Published Language. TEPP may consume only an immutable released contract. Mutable owner PR heads, copied source, cross-repository SQL, or a second TEPP numerical implementation are not accepted dependencies.

Open fast-mlsirm PR #1536 contains relevant prior art at head `03d1e943b9f51d509977b63889a2052eae81717b`: private `add_fsum_partial` / `finish_fsum` helpers in `crates/mlsirm-core/src/multilevel_estimator.rs`, introduced by `5b856206e15ecc31236a4109571f1be58e77c54a`. Those helpers implement CPython-style partial accumulation and final half-even correction for membership-weight admission. They are design evidence only. The current caller is domain-private, uses non-negative totals expected near one, returns a rounded sum rather than a correctly rounded original-count mean, does not expose mathematical-zero versus nonzero-underflow identity, and does not establish an associative worker-reduction contract.

The current immutable fast-mlsirm release `v0.9.1` predates `fast-mlsirm#1814`; it is not authority for this repair.

## Canonical owner implementation now in review

`fast-mlsirm#1816` is now the canonical implementation vehicle for #1814. At the time of this record the exact owner head is `b5fec1d33c02e8ea969405e59747f0505e690d20` on protected base `493326f2de49ea1704da0ded19868ed05d2fe00f`. It publishes the proposed contract identity `fast_mlsirm.binary64_mean@1.0.0` and implements separate signed 34×`u64` exact magnitudes in `2^-1074` units, original-count integer quotient/remainder finalization, and a single round-to-nearest-ties-to-even projection.

That is a material owner-path advance, not a TEPP dependency yet. The owner PR remains open and mergeable, while its Draft/Ready presentation state is intentionally **not** consumer authority: during this review cycle it moved from Ready-for-check admission back to Draft without changing the cited source head. Repository CI, package, native CodeQL, Semgrep, Security Scan and ClusterFuzzLite evidence are GREEN on that head, while the organization delegated CodeQL receipt remains RED through the known producer/consumer settlement path. The owner also identified that central `coverage-evidence` can be GREEN without producing owned Rust line/branch measurement; `.github#2099` owns that missing immutable coverage evidence. No qualifying independent submitted review is currently present. The previously rate-limited CodeRabbit review was retried and is now processing the same exact head; an in-progress bot review is not a clean finding or merge authority.

TEPP therefore keeps the public longitudinal RED unchanged. Consumer migration begins only after #1816 lands through the normal protected path and a new immutable fast-mlsirm release exposes the contract with version/tag/package, SBOM/provenance, reproducibility, rollback and real owned source coverage evidence. A mutable PR SHA is not an allowed production dependency even if its source-level arithmetic appears correct.

## Candidate exact accumulator bound

A fixed-width exact accumulator is feasible without introducing a psychometric sample ceiling when the numerical API accepts an ordinary Rust slice and supported production targets have `usize <= 64` bits.

Every finite binary64 value is an integer multiple of `q = 2^-1074`. The largest finite value is

`MAX = (2^53 - 1) * 2^971`.

In `q` units one addend therefore has magnitude

`I_max = (2^53 - 1) * 2^2045`,

which requires 2,098 magnitude bits. For every 64-bit slice cardinality `n <= 2^64 - 1`,

`n * I_max < 2^2162`.

A sign plus 2,162-bit magnitude is therefore sufficient for the exact mathematical sum over every slice representable by a 64-bit `usize`. A 34×`u64` magnitude provides 2,176 bits. This representation is implemented by the current #1816 candidate but remains **Proposed** for TEPP until protected integration and immutable release acceptance complete.

The mean must be rounded from the exact rational `(S / n) * 2^-1074`, not from a binary64-rounded sum. Integer quotient/remainder finalization can distinguish exact mathematical zero (`S == 0`) from a nonzero value that rounds below binary64 range and can apply round-to-nearest-ties-to-even at the final mean boundary. It also admits same-sign cases where the exact sum exceeds binary64 while the mean remains representable.

## Rejected local repairs

The following are not causal acceptance:

- adding another swallowed-term or pair-order special case to `scaled_compensated_mean`;
- replacing the current path with plain Kahan or Neumaier summation without a final-rounding proof;
- pre-scaling all inputs by a large magnitude when cancellation can make a subnormal term scientifically material;
- computing a rounded sum first and then dividing by the sample count;
- adding a model-specific sample ceiling solely to simplify a numerical proof;
- copying the private fast-mlsirm #1536 helpers or #1816 implementation into TEPP;
- pinning a mutable fast-mlsirm PR/head as production authority;
- accepting faithful or approximate behavior while documenting it as correctly rounded.

## Release and consumer acceptance

Before TEPP changes production arithmetic, the fast-mlsirm owner contract must be protected-merged and published in an immutable versioned release with SBOM, provenance, reproducibility, rollback evidence, rustdoc, test and edge-case coverage. Its RED/GREEN suite must include at least the TEPP half-ULP case, mirrored sign, `[1e16, -1, -1]`, `[MAX, tiny, -MAX]`, subnormal residue, exact cancellation, same-sign near-MAX finite means, exact midpoint/ties-to-even division, mathematical zero versus nonzero-underflow, input permutation, and worker-count determinism.

TEPP then pins the released version through the repository's approved dependency/ACL boundary, removes the local generic mean heuristic rather than retaining two numerical authorities, reruns the public longitudinal RED to GREEN, and reacquires exact-head Rust, 100% owned line/branch coverage, security, SBOM/provenance, review, and release evidence. LLM review is supplementary and cannot activate the numerical change.

## Research basis

Ogita, T., Rump, S. M., & Oishi, S. (2005). Accurate sum and dot product. *SIAM Journal on Scientific Computing, 26*(6), 1955–1988. https://doi.org/10.1137/030601818

Rump, S. M., Ogita, T., & Oishi, S. (2008). Accurate floating-point summation part I: Faithful rounding. *SIAM Journal on Scientific Computing, 31*(1), 189–224. https://doi.org/10.1137/050645671

These publications motivate error-free transformation and faithful summation analysis. They do not prove the fast-mlsirm implementation or authorize TEPP production activation by themselves.
