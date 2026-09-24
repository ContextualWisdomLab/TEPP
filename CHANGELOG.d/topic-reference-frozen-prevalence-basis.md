### Scientific validation

- Added an owner-issued training prevalence-design basis that freezes the reference estimator's training `EventTime` origin/location/scale and ordered `PrevalenceFeature` coordinates for later held-out projection. Evaluation rows reuse that basis without recomputing time statistics from the evaluation batch, and unseen covariate or membership coordinates fail closed instead of being silently reordered or dropped. This is numerical coordinate authority only; it does not authenticate Evidence, Membership, relation promotion, or source/event-time provenance. (#681)
