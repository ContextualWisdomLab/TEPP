# Validation Evidence: reduce exact four-observation ratio before bounded sqrt proof

- Fix `validation_core::bias_standard_error` for exact four-observation samples whose pair-distance numerator exceeds the binary64 exact-integer admission before reduction even though the identical reduced rational ratio fits the bounded proof.
- Reduce `Σ_{i<j}(r_i-r_j)^2 / 48` by its integer greatest common divisor before the exact midpoint-square comparison. This preserves the same rational radicand while avoiding an unnecessary fallback to the rounded ratio-then-`sqrt` path.
- Add a public represented-input contract for `[0, 14_099_687, 16_729_100, 94_045_527]`, permutations, and sign mirrors. Its exact pair-square sum `21699306139092196` reduces by `4` to `5424826534773049 / 12`; the correctly rounded standard error is bits `0x4174_46e5_76f8_7445` rather than the fallback's adjacent lower `0x4174_46e5_76f8_7444`.
