# Validation: preserve subtraction-rounded standard-error decisions

- Reject SE-aware recovery when a finite subtraction rounds onto the same binary64 value as an exact finite `k * SE` bound even though the exact dyadic residual represented by the inputs is larger.
- Compare the error-free subtraction correction, sign-adjusted for the absolute residual, with the FMA product correction only on nonzero finite rounded ties; retain the prior rounded decision when those correction projections are equal.
- Keep the finite direct path, scale-underflow repair, and both-overflow exact significand/exponent comparator unchanged outside this tie boundary.
