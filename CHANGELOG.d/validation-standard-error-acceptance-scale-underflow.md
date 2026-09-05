### Fixed

- Preserved finite positive `k · SE` acceptance bounds before scale normalization in `accept_within_standard_errors`. A large estimate/target scale could previously make `SE / scale` underflow to zero even when the represented direct residual and represented `k · SE` bound were both finite, falsely rejecting a scientifically admissible recovery result. Direct finite residual/bound comparison now precedes the overflow-only normalized fallback; zero-SE and zero-multiplier exact-recovery semantics are unchanged.
