### Fixed

- Validation Evidence now rejects `MonteCarloSummary` payloads whose `standard_error` is incoherent with the represented `standard_deviation` and `replication_count`. Positive spread must agree with `SD / sqrt(n)` within a small binary64 relative tolerance, zero spread requires zero SE, and singleton summaries require zero spread/SE. This prevents finite serialized evidence from materially understating or overstating Monte Carlo uncertainty without imposing cross-language bit-for-bit equality.
