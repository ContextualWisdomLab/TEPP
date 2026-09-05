### Fixed

- Preserve the exact three-observation rational-square bias standard-error identity when otherwise valid represented offsets are scaled high enough for their raw squares or cross-product to overflow. TEPP now retries the same error-free proof after an exactly reversible power-of-two normalization instead of falling back to a normalized moment path that can shift the correctly rounded result by one ULP.
