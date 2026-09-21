## Changed

- Add a Membership-owned `tepp.membership_observation_support_projection.v1` wire projection for longitudinal support. The owner path resolves every observation at its own event time, preserves repeated-member/repeated-group linkage through projection-local ordinals, retains role and exact binary64 weight bits, and binds the source-support digest issued by canonical Membership classification.
- Canonical JSON omits raw `MemberId`/`GroupId` UUID text and rejects unsupported versions, unknown fields, timezone aliases, malformed weights, noncanonical ordering, and invalid local-ordinal spaces. This is privacy reduction for scientific reconstruction, not anonymity; parsed wire values remain non-authoritative serialization data.
