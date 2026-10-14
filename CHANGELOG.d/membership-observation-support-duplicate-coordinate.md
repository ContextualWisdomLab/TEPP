# Membership support duplicate-coordinate determinism

`tepp.membership_observation_support_projection.v1` now rejects a parsed payload when the same projection-local `(member_ordinal, event_time)` coordinate is repeated with different active Membership assignments. Canonical owner state resolves one topology at one member/event-time coordinate, so contradictory duplicate rows are unreachable even when each row is individually well formed and the aggregate design label remains plausible.

Byte-identical duplicate rows remain valid and multiplicity-sensitive. This repair does not deduplicate observations, change the wire schema, make parsed wire authoritative, or alter Membership estimator arithmetic.
