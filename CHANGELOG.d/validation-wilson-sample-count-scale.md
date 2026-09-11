# Validation: preserve exact Wilson sample-count scale

- `WilsonCoverageEvidenceV1` no longer pre-rounds a fixed-width `u64` sample count before Wilson score projection when the count is not exactly representable in binary64.
- The inexact-large-count path evaluates Wilson scale terms through the correctly rounded reciprocal `1 / n`; exact binary64-representable counts keep the existing path.
- The all-covered large-count branch evaluates the complementary Wilson miss mass before subtracting from one, so representable uncertainty immediately below `1.0` is not erased.
- The prior non-boundary large-count fixture is corrected because its Wilson lower endpoint had inherited the rounded-denominator calculation even after empirical `covered_count / sample_count` was fixed.
