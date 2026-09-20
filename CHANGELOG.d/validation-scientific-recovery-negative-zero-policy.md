### Validation

- Versioned scientific recovery profiles now reject an IEEE 754 negative-zero Monte Carlo uncertainty multiplier. Canonical `+0.0` remains valid. Because the multiplier is hashed by its represented binary64 bits, accepting both signed zeros would create two profile SHA-256 identities for the same zero-uncertainty scientific policy.
