# Component recovery denominator identity

## Decision

Known-truth component RMSE in the Longitudinal Modeling bounded context uses one contribution per unique `(unit_index, occasion_index, component_level)` identity. Duplicate identity multiplicity is invalid evidence and fails closed with `LongitudinalError::InvalidComponentPayload`; it is not an implicit observation weight. The order in which truth and recovered rows are serialized is not part of that scientific identity. Both slices are therefore admitted as identity maps and residuals are accumulated in canonical tuple order.

This boundary matters for two separate reasons. Repeating an already-matched component changes the RMSE denominator and can improve or worsen the reported recovery metric without adding a new known-truth target. Requiring pairwise row order can reject two scientifically identical recovery sets merely because an upstream repository, query, or transport emitted the same unique component identities in a different order. Even after identity lookup removes pairwise order coupling, iterating residuals in caller-provided truth order can still change the last bit of the scaled binary64 sum of squares. TEPP therefore treats tuple identity as authoritative for both alignment and deterministic accumulation while rejecting missing or duplicated identities.

Morris, White, and Crowther (2019) frame simulation evaluation around explicitly defined estimands and performance measures. For TEPP's known-truth recovery contract, the performance unit is the identified longitudinal component, so denominator membership and truth-to-recovery alignment must be determined by component identity rather than accidental row multiplicity or serialization order. The current published *Standards for Educational and Psychological Testing* remains the 2014 AERA/APA/NCME edition while a revision is in progress; TEPP treats reproducible evidence and clearly specified score/evidence interpretation as governing constraints rather than inferring validity from transport order.

## RED → repair trace

- Duplicate-identity RED `698f12f5b2f7a3c194e9d1d3f00c5aeaf10591f8`: `crates/longitudinal_core/tests/component_rmse_duplicate_identity_contract.rs` supplies the same `(unit, occasion, level)` twice in both truth and decided series. The predecessor implementation accepted it and therefore allowed duplicate multiplicity to change the recovery denominator.
- Duplicate-identity repair `2fae4cb2e7df2845270bd27192000ca370fb05ad`: `crates/longitudinal_core/src/component.rs` records admitted component identities before residual accumulation and rejects a duplicate.
- Cross-slice row-order RED `8ad72ac91cbddd5ce6432fc70630bad6ce7072ce`: `crates/longitudinal_core/tests/component_rmse_permutation_identity_contract.rs` supplies the same three unique known-truth components and recovered values in a different recovered-row order. The predecessor pairwise `zip` path rejects the scientifically identical perfect recovery solely because serialization order differs.
- Initial identity-alignment repair `2dd9537e04dd2048559ba707fecd2404db5a2a31`: recovered rows are indexed by `(unit, occasion, level)` so decided-row permutation no longer changes admission. Review of that repair found that residual accumulation still followed caller-provided truth order, leaving deterministic binary64 output order-sensitive.
- Truth-order rounding RED `5fb93c40eddbd9e7920196ef09594457b8ac72d3`: `crates/longitudinal_core/tests/component_rmse_truth_permutation_contract.rs` uses residual magnitudes `1`, `1e-100`, `3`, and the representable value immediately below `1`. Two truth-row permutations contain exactly the same identity-value pairs but drive the scaled sum-of-squares through different rounding paths and produce different RMSE bit patterns on the predecessor identity-aligned implementation.
- Causal deterministic repair `025dce7fd98cfb4f94ea790cacd555b744095377`: both truth and recovered slices are admitted into unique identity maps, truth identities are sorted by `(unit, occasion, level wire name)`, and residual accumulation follows that canonical order. Missing identities still fail closed. Existing overflow-safe residual scaling, nonzero-underflow refusal, and exact perfect-recovery zero remain unchanged.
- Edge-coverage reinforcement `976ce7d710125717ff2f8daeb943d54278c4acde`: a truth-side duplicate with unique recovered identities exercises the independent truth uniqueness gate rather than relying only on a payload where both slices contain duplicates.
- Public API: `component_root_mean_square_error`.
- Domain owner: `crates/longitudinal_core`; this is longitudinal Validation Evidence identity/admission policy, not reusable static psychometric arithmetic for `fast-mlsirm`.

## Invariant

For an admitted recovery vector of length `n`, there are exactly `n` unique component identities in each slice and the two identity sets are equal. Any permutation of either slice that preserves those identity-value pairs yields bit-identical deterministic CPU `f64` RMSE. A caller that needs weighted recovery must use a separately named, explicitly weighted contract with its own denominator and validation evidence; duplicate rows are not weights.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. https://www.testingstandards.net/open-access-files.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

### Standards currency note

The joint AERA/APA/NCME site still distributes the 2014 edition as the current published edition. A joint revision process is active; this note should be revisited when the successor edition is formally published rather than treating revision activity as a released standard.
