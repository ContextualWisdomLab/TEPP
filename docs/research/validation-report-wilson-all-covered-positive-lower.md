# Wilson all-covered lower-endpoint support in durable Validation Evidence

## Decision

`ValidationReport` must reject exact all-covered (`interval_coverage == 1.0`) Wilson evidence whose stored lower endpoint is numeric zero.

The existing endpoint-pair identity is necessary for strict-interior coverage but degenerates at `p = 1`: with `U = 1`, the eliminated identity is satisfied for any `L`. That makes `[0, 1]` look algebraically coherent even though TEPP's canonical all-covered producer cannot emit it.

## Producer invariant

`wilson_coverage_interval` handles exact all-covered evidence as

`L = n / (n + z²)`, `U = 1`,

where the input slice is non-empty (`n >= 1`) and the represented `z²` must be finite. Therefore `n + z²` is finite and positive and the represented lower endpoint is strictly positive. If a positive `z` squares to binary64 zero, the producer yields `L = 1`, not `0`; that existing underflow behavior does not weaken this invariant.

An extreme finite fixture with `n = 1` and `z = 1e154` keeps `z²` finite and produces a positive lower endpoint near the smallest normal/subnormal transition, exercising the intended full-range support without inventing a denominator or critical value that the report does not store.

## RED → repair trace

- Public RED: `ce714f077fe1575b50f1b97131e1857ad0c69b1e`, `crates/validation_core/tests/validation_report_wilson_all_covered_positive_lower_contract.rs`.
- The RED proves the canonical extreme-`z` all-covered lower endpoint remains positive and requires `+0.0` and `-0.0` lower endpoints to fail explicit validation, JSON egress, human projection, and serde ingress.
- During the source edit, two intermediate commits introduced unrelated transcription defects. They were not treated as valid evidence and were fully neutralized by `184990522287e254a8e4c1995c02bce20aaa288e`, which restores the exact predecessor `report.rs` blob while preserving the RED file and branch ancestry. No force push or destructive rebase was used.
- Causal source repair: `72e9d9546e4bf98a63544d6e76a92116da5bf670`. The only surviving source delta splits the degredate `p = 1` case from `p = 0` and requires `coverage_wilson_lower > 0.0` for exact all-covered evidence.
- Changelog trace: `e3a2f4a27a9355c77c2aa9aad92793d254536439`.

## Scope and owner boundary

This is TEPP Validation Evidence artifact admission for the existing Wilson coverage producer. It does not redefine the Wilson estimator, does not add reusable static psychometric arithmetic, and does not move owner responsibility from `fast-mlsirm`. It also does not claim full Wilson provenance: `ValidationReport` still lacks the empirical denominator and critical-value/confidence-level semantics required for exact recomputation.

The rule is intentionally asymmetric at the boundaries. For `p = 0`, the current producer can emit an upper endpoint of zero when positive `z` squares to represented zero, so admission does not invent an `upper > 0` rule. For `p = 1`, however, the producer's `n / (n + z²)` path remains strictly positive for every accepted configuration.

## Standards and primary research trace

Wilson's score interval remains the primary statistical source. The current published *Standards for Educational and Psychological Testing* is the 2014 edition jointly issued by AERA, APA, and NCME; the sponsoring organizations have a Joint Committee revising that 2014 edition, so an unpublished revision is not treated as current normative authority.

### References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953
