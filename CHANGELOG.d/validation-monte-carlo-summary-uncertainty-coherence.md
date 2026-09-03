### Fixed

- Validation Evidence now rejects `MonteCarloSummary` payloads whose standard error is impossible for the represented sample spread/count: nonzero sample SD with exact-zero SE, SE larger than SD, or nonzero singleton spread/SE. This prevents finite serialized evidence from claiming less or more Monte Carlo uncertainty than the summary contract can represent.
