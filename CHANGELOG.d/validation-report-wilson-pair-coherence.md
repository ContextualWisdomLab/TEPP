### Fixed

- `ValidationReport` now rejects interval-coverage evidence whose stored Wilson lower and upper endpoints merely contain the empirical coverage but cannot be the two roots of one Wilson score interval for that same proportion. Admission uses the Wilson root identities after eliminating the unrecorded `z² / n` term, with the complementary uncovered-proportion identity below `p = 0.5` and a probability-scale binary64 tolerance.
- Canonical Wilson output remains admissible, including asymmetric interior coverage; JSON serialization, serde ingress, and human projection share the same fail-closed scientific boundary.
