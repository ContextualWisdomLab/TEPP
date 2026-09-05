### Fixed

- `validation_core::bias_standard_error` now preserves an exact three-observation, three-level rational-square identity when every translated product/addition is proven error-free and the dispersion numerator is itself an exact represented square. The exact root is divided by three once instead of being reconstructed through rounded normalized moments and `sqrt`.
- The public represented-input contract `truth=[0,0,0]`, `recovered=[0,5/1024,21/1024]` now returns the correctly rounded binary64 standard error `0x3f79_5555_5555_5555` for all six permutations and their sign mirrors; the predecessor generic translated-moment path returned adjacent upper `0x3f79_5555_5555_5556`.
