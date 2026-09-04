# Validation: preserve subnormal standard-error bound decisions

- Reject SE-aware recovery when an exact finite residual equals the rounded minimum-subnormal `k * SE` bound but the exact product represented by `k` and `SE` is smaller.
- On this subnormal finite-tie boundary, compare the exact represented residual magnitude with the exact dyadic product when both subtraction and FMA correction terms project to zero; keep exact minimum-subnormal equality accepted.
- Preserve the existing finite direct path, low-term tie discriminator, one-sided overflow behavior, and both-overflow exact comparator outside this boundary.
