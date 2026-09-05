# Validation bias-SE exact-proof budget characterization

- Correct the pair-record resource evidence for 3,162 observations to 4,997,541 records and lock exact pair counts in a Rust characterization contract.
- Distinguish the current minimum-shifted O(n) `u128` intermediate envelope (`n=2,047` at aligned diameter `2^53`) from the wider exact pair-square numerator envelope (`n=4,095` for the same diameter); neither is a production sample-count budget.
- Prove the checked O(n) accumulator is a sufficient but not admission-equivalent replacement for the pair reference: with one zero and the remaining coefficients at `D=2^58`, both kernels fit at `n=64`, while at `n=65` the exact pair numerator still fits (`2^122`) but `n*sum(c_i^2)` overflows `u128` before cancellation.
- Extend the release-mode characterization harness to compare the buffered O(n²) pair layout, an allocation-free two-pass O(n²) reference, and the O(n) algebraic accumulator while reporting target-specific pair-record size, scratch-record capacity, and scratch payload bytes.
- Keep production `bias_standard_error` admission unchanged at `n=4..=16`; any O(n) production path must preserve current admission with pairwise fallback or wider checked integers and still requires actual release-mode CPU/allocation results, exact-head CI, and applicable buyer-path p95 evidence.
