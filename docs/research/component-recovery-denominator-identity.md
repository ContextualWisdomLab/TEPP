# Component recovery denominator identity

## Decision

Known-truth component RMSE in the Longitudinal Modeling bounded context uses one contribution per unique `(unit_index, occasion_index, component_level)` identity. Duplicate identity multiplicity is invalid evidence and fails closed with `LongitudinalError::InvalidComponentPayload`; it is not an implicit observation weight.

This boundary matters because repeating an already-matched component changes the RMSE denominator and can improve or worsen the reported recovery metric without adding a new known-truth target. The metric therefore cannot distinguish a duplicated record from an intentional weight unless identity uniqueness is enforced before accumulation. TEPP does not assign such weights in this API.

Morris, White, and Crowther (2019) frame simulation evaluation around explicitly defined estimands and performance measures. For TEPP's known-truth recovery contract, the performance unit is the identified longitudinal component, so denominator membership must be determined by component identity rather than accidental row multiplicity. The current published *Standards for Educational and Psychological Testing* remains the 2014 AERA/APA/NCME edition while a revision is in progress; TEPP treats reproducible evidence and clearly specified score/evidence interpretation as governing constraints rather than inferring validity from duplicated records.

## RED → repair trace

- RED `698f12f5b2f7a3c194e9d1d3f00c5aeaf10591f8`: `crates/longitudinal_core/tests/component_rmse_duplicate_identity_contract.rs` supplies the same `(unit, occasion, level)` twice in both truth and decided series. The predecessor implementation accepted it and therefore allowed duplicate multiplicity to change the recovery denominator.
- Causal repair `2fae4cb2e7df2845270bd27192000ca370fb05ad`: `crates/longitudinal_core/src/component.rs` records admitted component identities before residual accumulation and rejects a duplicate. Existing pairwise identity matching, finite-input checks, overflow-safe residual scaling, nonzero-underflow refusal, and exact perfect-recovery zero remain unchanged.
- Public API: `component_root_mean_square_error`.
- Domain owner: `crates/longitudinal_core`; this is longitudinal Validation Evidence identity/admission policy, not reusable static psychometric arithmetic for `fast-mlsirm`.

## Invariant

For an admitted recovery vector of length `n`, there are exactly `n` unique component identities in both aligned slices. A caller that needs weighted recovery must use a separately named, explicitly weighted contract with its own denominator and validation evidence; duplicate rows are not weights.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. https://www.testingstandards.net/open-access-files.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

### Standards currency note

The joint AERA/APA/NCME site still distributes the 2014 edition as the current published edition. A joint revision process is active; this note should be revisited when the successor edition is formally published rather than treating revision activity as a released standard.
