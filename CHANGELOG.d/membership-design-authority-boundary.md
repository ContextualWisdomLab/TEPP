### Membership

- Separate deserialized `MembershipDesignWire` coordinates from owner-derived longitudinal classification authority. `classify_membership_observations_wire` now returns `MembershipDesignClassification`, whose private state can only be issued after canonical Membership classification at each observation event time; parsing a supported `{version, name}` pair remains a non-authoritative serialization boundary.
