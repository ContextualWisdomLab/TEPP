# Validation bias-SE exact-proof budget characterization

- Correct the pair-record resource evidence for 3,162 observations to 4,997,541 records and lock exact pair counts in a Rust characterization contract.
- Distinguish the current minimum-shifted O(n) `u128` intermediate ceiling (`n=2,047` at aligned diameter `2^53`) from the wider exact pair-square numerator envelope (`n=4,095` for the same diameter); neither is a production sample-count budget.
- Extend the release-mode characterization harness to compare the buffered O(n²) pair layout, an allocation-free two-pass O(n²) reference, and the O(n) algebraic accumulator while reporting target-specific pair-record size, scratch-record capacity, and scratch payload bytes.
- Keep production `bias_standard_error` admission unchanged at `n=4..=16` pending actual release-mode CPU/allocation results, admitted/refused-set comparison, exact-head CI, and applicable buyer-path p95 evidence.
