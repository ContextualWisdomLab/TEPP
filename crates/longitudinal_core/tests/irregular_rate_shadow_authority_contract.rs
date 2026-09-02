#![forbid(unsafe_code)]

//! Architecture fitness for one Longitudinal Modeling irregular-rate estimand.
//!
//! The crate-public path and its internal temporal composition must share one
//! numerical authority. A compatibility/public facade may forward the named
//! operations, but it must not carry a second mean/log-rate implementation
//! whose floating-point edge semantics can drift independently.

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

#[test]
fn canonical_irregular_rate_implementation_is_not_a_second_public_api() {
    let source = include_str!("../src/irregular_residual.rs");

    assert!(
        !source.contains("pub fn recover_centered_irregular_residual_log_rate("),
        "the canonical implementation must stay crate-private; stable_irregular_rate is the only crate-public facade"
    );
    assert!(
        !source.contains("pub fn recover_within_unit_irregular_residual_log_rate("),
        "the canonical implementation must stay crate-private; stable_irregular_rate is the only crate-public facade"
    );
    assert!(
        source.contains("pub(crate) fn recover_centered_irregular_residual_log_rate("),
        "the public facade still needs crate-visible access to the canonical centered-rate implementation"
    );
    assert!(
        source.contains("pub(crate) fn recover_within_unit_irregular_residual_log_rate("),
        "the public facade still needs crate-visible access to the canonical CWC irregular-rate implementation"
    );
}

#[test]
fn public_irregular_rate_facade_has_no_second_numerical_implementation() {
    let facade = include_str!("../src/stable_irregular_rate.rs");

    assert!(
        !facade.contains("fn stable_mean("),
        "stable_irregular_rate still owns a second floating-point mean implementation"
    );
    assert!(
        facade.contains("irregular_residual::recover_centered_irregular_residual_log_rate"),
        "the public facade must delegate the centered-rate estimand to the canonical Longitudinal Modeling implementation"
    );
    assert!(
        facade.contains("irregular_residual::recover_within_unit_irregular_residual_log_rate"),
        "the public facade must delegate the CWC irregular-rate estimand to the canonical Longitudinal Modeling implementation"
    );
}
