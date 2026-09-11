### Validation Evidence

`ValidationReport` now rejects a stored Wilson lower endpoint of exact zero, and the complement-symmetric exact-one upper endpoint, when the peer endpoint implies that the omitted Wilson root remains representable in binary64. This closes a boundary-admission hole where the eliminated endpoint-pair residual could cancel to zero even though the stored pair could not come from one Wilson interval, while preserving genuinely unrepresentable extreme boundary roots.
