#![forbid(unsafe_code)]

//! Architecture fitness for one Longitudinal Modeling irregular-rate estimand.
//!
//! The crate-public implementation already survives non-representable direct
//! residual ratios by evaluating the equivalent log-domain difference. The
//! private composition module must not retain the superseded direct-ratio
//! rejection path, because that would leave two scientifically different
//! implementations of the same named estimand in one bounded context.

#[test]
fn shadowed_centered_rate_does_not_reintroduce_direct_ratio_rejection() {
    let source = include_str!("../src/irregular_residual.rs");

    assert!(
        !source.contains("let ratio = pair.later_residual() / pair.earlier_residual();"),
        "the internal centered-rate path still rejects finite log-rates solely because the direct residual ratio overflows or underflows"
    );
    assert!(
        source.contains("driver_same_sign_log_rate("),
        "the internal composition must reuse the Longitudinal Modeling same-sign log-rate primitive"
    );
}
