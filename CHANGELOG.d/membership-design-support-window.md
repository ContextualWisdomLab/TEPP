## Changed

- Bind owner-derived longitudinal Membership design classifications to the exact classified support cardinality and its earliest/latest event times. These coordinates are derived from the same `MembershipObservation` slice used by canonical classification and are not caller-supplied.
- Keep parsed `MembershipDesignWire` values free of analytical authority. Support count and time bounds belong only to `MembershipDesignClassification` issued after canonical Membership classification.
- Treat count plus event-time bounds as provenance coordinates, not complete cohort reconstruction: different supports can share the same summary, so released analysis projections still require privacy-appropriate observation-support evidence and exact binding where scientific reconstruction depends on it.
