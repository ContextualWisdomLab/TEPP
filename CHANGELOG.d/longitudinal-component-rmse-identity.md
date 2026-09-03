### Longitudinal Modeling

- Known-truth component RMSE rejects duplicate `(unit, occasion, level)` identities instead of letting repeated rows silently reweight the scientific recovery denominator.
- Truth and recovered component rows are matched by `(unit, occasion, level)` identity and accumulated in canonical identity order, so serialization-order permutations preserve the same deterministic CPU `f64` recovery result instead of changing admission or the last-bit RMSE rounding path.
