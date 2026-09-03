### Longitudinal Modeling

- Known-truth component RMSE rejects duplicate `(unit, occasion, level)` identities instead of letting repeated rows silently reweight the scientific recovery denominator.
- Truth and recovered component rows are matched by `(unit, occasion, level)` identity rather than slice position, so a serialization-order permutation cannot turn scientifically identical recovery evidence into an invalid payload.
