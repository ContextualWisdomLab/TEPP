# Validation: preserve standard-error ties below correction resolution

- Reject SE-aware recovery when an exact finite residual equals a rounded `k * SE` bound but the exact represented product is smaller and the FMA correction itself falls below binary64 resolution.
- Cover both the minimum-subnormal rounded bound and the minimum-normal boundary: when subtraction is exact and both finite-tie correction projections are zero, compare the represented residual magnitude with the exact dyadic product of represented `k` and `SE`.
- Keep exact minimum-subnormal/minimum-normal equality accepted and preserve the existing finite direct path, nonzero low-term tie discriminator, one-sided overflow behavior, and both-overflow exact comparator outside this boundary.
