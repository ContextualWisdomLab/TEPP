### Validation

- Preserve Neumaier compensation through the scientific count division when exact mixed-sign cancellation leaves a same-sign remainder. This prevents a represented-input mean bias from moving by one ULP because `sum + correction` was rounded before division. The public regression contract covers the sign-mirrored boundary as well.
