## Fixed

- Validation Evidence now treats `k = 0` in the standard-error acceptance gate as exact recovery before any binary64 scale reduction, preventing a huge standard error from erasing a nonzero residual into a false acceptance.
- Exact-recovery acceptance now uses finite numeric equality rather than IEEE total-order identity, so `-0.0` and `+0.0` are one zero-valued recovery result while every nonzero finite residual remains distinct.
