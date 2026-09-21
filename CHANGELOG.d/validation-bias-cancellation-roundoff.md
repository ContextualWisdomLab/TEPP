### Fixed

- Validation Evidence mean-bias accumulation now retains error-free low terms from opposite-sign cancellation before scale reduction. Repeated sub-ULP residuals can therefore contribute when their combined represented mass changes the final bias, while exact cancellation and fail-closed unrepresentable results keep their existing semantics.
