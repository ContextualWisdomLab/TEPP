## Fixed

- Validation Evidence now treats `k = 0` in the standard-error acceptance gate as exact recovery before any binary64 scale reduction, preventing a huge standard error from erasing a nonzero residual into a false acceptance.
